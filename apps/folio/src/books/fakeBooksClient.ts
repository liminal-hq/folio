// An in-memory books client for tests and for the page outside Tauri
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type {
	BookSnapshot,
	BookSummary,
	Intent,
	KindInfo,
	NewBookSpec,
	PaintPresets,
	Patch,
	PatchEvent,
} from '@liminal-hq/folio-plugin-books';
import type { Book } from '@liminal-hq/folio-protocol/generated/Book';
import type { BookSpelling } from '@liminal-hq/folio-protocol/generated/BookSpelling';
import type { Recoverable } from '@liminal-hq/folio-protocol/generated/Recoverable';
import type { DictionaryProgress } from '@liminal-hq/folio-protocol/generated/DictionaryProgress';
import type { DictionaryStatus } from '@liminal-hq/folio-protocol/generated/DictionaryStatus';
import type { ReaderPreview } from '@liminal-hq/folio-protocol/generated/ReaderPreview';
import type { ExportReport } from '@liminal-hq/folio-protocol/generated/ExportReport';
import type { PreflightItem } from '@liminal-hq/folio-protocol/generated/PreflightItem';
import type { KeepFontsReport } from '@liminal-hq/folio-protocol/generated/KeepFontsReport';
import { applyPatch } from './applyPatch';
import type { BooksClient, BooksListener } from './booksClient';
import cases from './fixtures/patchCases.json';

/** A real book from the Rust session's own fixtures, so the fake holds the shapes Rust makes. */
const BASE_BOOK = (cases as unknown as Array<{ before: Book }>)[0]!.before;

/** A palette like the plugin's, trimmed to what the tests read. */
export const FAKE_PAINTS: PaintPresets = {
	swatches: [
		{ name: 'Paper white', colour: '#ffffff' },
		{ name: 'Cream', colour: '#f6efdc' },
		{ name: 'Slate', colour: '#1f3a5f' },
	],
	textures: [
		{ id: 'linen', name: 'Linen' },
		{ id: 'dot-grid', name: 'Dot grid' },
	],
	gradients: [
		{
			name: 'Dawn',
			angle: 180,
			stops: [
				{ at: 0, colour: '#f7d9b5' },
				{ at: 1, colour: '#f2b8c6' },
			],
		},
	],
};

/** The six kinds as the plugin lists them, trimmed to what the screens read. */
export const FAKE_KINDS: KindInfo[] = [
	[
		'picture',
		'Picture book',
		'Fixed layout · pictures lead',
		'fixed',
		['8.5 × 8.5 in', '8 × 10 in'],
		32,
	],
	[
		'novel',
		'Novel',
		'Flowing text · reflows on e-readers',
		'flowing',
		['5 × 8 in', '6 × 9 in'],
		248,
	],
	['textbook', 'Textbook', 'Grid, figures, callouts', 'fixed', ['8.5 × 11 in', '7 × 10 in'], 96],
	[
		'illustrated',
		'Illustrated',
		'Cookbook, garden, craft',
		'flowing',
		['7 × 10 in', '8 × 10 in'],
		144,
	],
	['poetry', 'Poetry', 'Generous white space', 'flowing', ['5.5 × 8.5 in', '5 × 8 in'], 80],
	[
		'memoir',
		'Memoir with photos',
		'Text with photo spreads',
		'flowing',
		['6 × 9 in', '5.5 × 8.5 in'],
		212,
	],
].map(([kind, label, sub, layout, trims, pages]) => ({
	kind: kind as KindInfo['kind'],
	label: label as string,
	sub: sub as string,
	layout: layout as KindInfo['layout'],
	trims: (trims as string[]).map((t) => ({ label: t, width: 612, height: 612 })),
	defaultPages: pages as number,
	sampleTitle: `Sample ${label as string}`,
}));

interface Held {
	snapshot: BookSnapshot;
	past: number;
	future: number;
	saved: boolean;
}

/** What a test can drive besides the client calls. */
export interface FakeBooks {
	client: BooksClient;
	/** Everything the page called, in order, as `name(args)`. */
	calls: string[];
	/** Sends an event as if another window had changed a book. */
	emit: { patch(event: PatchEvent): void; changed(): void };
	/** Makes the next call named `name` fail with a refusal. */
	refuseNext(name: string, kind: string, message: string): void;
	/** Books a crash left changes in, which a test puts here for the client to offer. */
	recoveries: Recoverable[];
	/** The words the fake speller flags, whatever the case; none to begin with. */
	misspelt: string[];
	/** The families installed on the fake computer; a test puts some here. */
	installedFonts: string[];
	/** What the fake preflight finds; a test puts items here. */
	preflightItems: PreflightItem[];
}

export function createFakeBooksClient(): FakeBooks {
	const held = new Map<string, Held>();
	const listeners = new Set<BooksListener>();
	const calls: string[] = [];
	const refusals = new Map<string, { kind: string; message: string }>();
	let counter = 0;
	const misspelt: string[] = [];
	const installedFonts: string[] = [];
	const preflightItems: PreflightItem[] = [];
	const downloaded = new Set<string>();
	/** Books a crash left changes in; a test puts some here. */
	const recoveries: Recoverable[] = [];
	/** The revision each book was last copied at, as the plugin keeps it. */
	const copied = new Map<string, number>();
	const progress = new Set<(event: DictionaryProgress) => void>();
	const listing = (): DictionaryStatus[] => [
		{
			id: 'en-CA',
			name: 'English (Canada)',
			languages: ['en-CA'],
			licence: '(MIT AND BSD)',
			bytes: 0,
			installed: true,
			bundled: true,
		},
		...[
			{
				id: 'fr',
				name: 'French',
				languages: ['fr', 'fr-CA'],
				licence: 'MPL-2.0',
				bytes: 1_400_000,
			},
			{
				id: 'nl',
				name: 'Dutch',
				languages: ['nl'],
				licence: '(BSD-3-Clause OR CC-BY-3.0)',
				bytes: 2_500_000,
			},
		].map((d) => ({ ...d, installed: downloaded.has(d.id), bundled: false })),
	];

	const summaries = (): BookSummary[] => [...held.values()].map((h) => summary(h));
	const summary = (h: Held): BookSummary => ({
		...h.snapshot.summary,
		dirty: !h.saved || (h.snapshot.revision !== 0 && h.past > 0),
	});
	const changed = () => listeners.forEach((l) => l.onChanged({ books: summaries() }));
	const call = (name: string, args: unknown[] = []) => {
		calls.push(`${name}(${args.map((a) => JSON.stringify(a)).join(', ')})`);
		const refusal = refusals.get(name);
		if (refusal) {
			refusals.delete(name);
			return Promise.reject(refusal);
		}
		return null;
	};
	const find = (id: string): Held => {
		const book = held.get(id);
		if (!book) throw { kind: 'noSuchBook', message: `there is no open book \`${id}\`` };
		return book;
	};
	const patch = (id: string, held: Held, bleed: number): Patch => {
		const revision = held.snapshot.revision + 1;
		const result: Patch = {
			revision,
			ops: [
				{
					op: 'setGeometry',
					trim: held.snapshot.book.trim,
					margins: held.snapshot.book.margins,
					bleed,
					grid: held.snapshot.book.grid,
				},
			],
		};
		held.snapshot = {
			...held.snapshot,
			revision,
			book: applyPatch(held.snapshot.book, result),
			canUndo: held.past > 0,
			canRedo: held.future > 0,
		};
		const event: PatchEvent = {
			id,
			patch: result,
			canUndo: held.past > 0,
			canRedo: held.future > 0,
		};
		listeners.forEach((l) => l.onPatch(event));
		return result;
	};

	const client: BooksClient = {
		async presets() {
			await call('presets');
			return FAKE_KINDS;
		},
		async paintPresets() {
			await call('paintPresets');
			return FAKE_PAINTS;
		},
		async defaultSavePath(title) {
			await call('defaultSavePath', [title]);
			return `/home/mom/Books/${title}/${title}.folio`;
		},
		async newBook(spec: Partial<NewBookSpec>) {
			await call('newBook', [spec]);
			counter += 1;
			const id = `bs_${counter}`;
			const book: Book = {
				...BASE_BOOK,
				title: spec.title || 'Untitled',
				kind: spec.kind ?? 'picture',
			};
			const snapshot: BookSnapshot = {
				summary: { id, title: book.title, kind: book.kind, path: null, dirty: true },
				revision: 0,
				book,
				styles: {},
				settings: {},
				canUndo: false,
				canRedo: false,
			};
			held.set(id, { snapshot, past: 0, future: 0, saved: false });
			changed();
			return snapshot;
		},
		async openBook(path) {
			await call('openBook', [path]);
			const existing = [...held.values()].find((h) => h.snapshot.summary.path === path);
			if (existing) return existing.snapshot;
			counter += 1;
			const id = `bs_${counter}`;
			const snapshot: BookSnapshot = {
				summary: { id, title: path.split('/').pop() ?? path, kind: 'picture', path, dirty: false },
				revision: 0,
				book: { ...BASE_BOOK, title: path.split('/').pop() ?? path },
				styles: {},
				settings: {},
				canUndo: false,
				canRedo: false,
			};
			held.set(id, { snapshot, past: 0, future: 0, saved: true });
			changed();
			return snapshot;
		},
		async saveBook(id, path) {
			await call('saveBook', [id, path ?? null]);
			const h = find(id);
			if (!path && !h.snapshot.summary.path) throw { kind: 'needsPath', message: 'needs a place' };
			h.snapshot = {
				...h.snapshot,
				summary: { ...h.snapshot.summary, path: path ?? h.snapshot.summary.path },
			};
			h.saved = true;
			h.past = 0;
			changed();
			return summary(h);
		},
		async closeBook(id) {
			await call('closeBook', [id]);
			find(id);
			held.delete(id);
			changed();
		},
		async list() {
			await call('list');
			return summaries();
		},
		async snapshot(id) {
			await call('snapshot', [id]);
			return find(id).snapshot;
		},
		async apply(id, intent: Intent) {
			await call('apply', [id, intent.type]);
			const h = find(id);
			h.past += 1;
			h.future = 0;
			const result = patch(id, h, h.snapshot.book.bleed + 1);
			changed();
			return result;
		},
		async undo(id) {
			await call('undo', [id]);
			const h = find(id);
			if (h.past === 0) return null;
			h.past -= 1;
			h.future += 1;
			const result = patch(id, h, h.snapshot.book.bleed - 1);
			changed();
			return result;
		},
		async autosave() {
			await call('autosave');
			let written = 0;
			for (const [id, h] of held) {
				const touched = h.snapshot.summary.path === null ? h.past > 0 : !h.saved || h.past > 0;
				if (touched && copied.get(id) !== h.snapshot.revision) {
					copied.set(id, h.snapshot.revision);
					written += 1;
				}
			}
			return written;
		},
		async recoverable() {
			await call('recoverable');
			return [...recoveries];
		},
		async restore(id) {
			await call('restore', [id]);
			const at = recoveries.findIndex((r) => r.id === id);
			if (at < 0) throw { kind: 'recovery', message: `there is no recovery copy ${id}` };
			const [found] = recoveries.splice(at, 1);
			counter += 1;
			const bookId = `bs_${counter}`;
			const snapshot: BookSnapshot = {
				summary: {
					id: bookId,
					title: found!.title,
					kind: 'picture',
					path: found!.path,
					dirty: true,
				},
				revision: 1,
				book: { ...BASE_BOOK, title: found!.title },
				styles: {},
				settings: {},
				canUndo: false,
				canRedo: false,
			};
			held.set(bookId, { snapshot, past: 0, future: 0, saved: false });
			changed();
			return snapshot;
		},
		async discardRecovered(id) {
			await call('discardRecovered', [id]);
			const at = recoveries.findIndex((r) => r.id === id);
			if (at >= 0) recoveries.splice(at, 1);
		},
		async historyMark(id) {
			await call('historyMark', [id]);
			return find(id).past;
		},
		async rewindHistory(id, mark) {
			await call('rewindHistory', [id, mark]);
			const h = find(id);
			while (h.past > mark) {
				h.past -= 1;
				h.future += 1;
				patch(id, h, h.snapshot.book.bleed - 1);
			}
			changed();
		},
		async redo(id) {
			await call('redo', [id]);
			const h = find(id);
			if (h.future === 0) return null;
			h.future -= 1;
			h.past += 1;
			const result = patch(id, h, h.snapshot.book.bleed + 1);
			changed();
			return result;
		},
		async layoutSpread(id, spread, spelling) {
			await call('layoutSpread', spelling ? [id, spread, spelling] : [id, spread]);
			const h = find(id);
			const found = h.snapshot.book.spreads.find((s) => s.id === spread);
			if (!found) throw { kind: 'noSuchSpread', message: `there is no spread \`${spread}\`` };
			// Every text frame is laid out as empty, which is enough to draw the spread's furniture.
			const frames = found.frames
				.filter((f) => f.type === 'text')
				.map((f) => ({ frame: f.id, lines: [], overset: false }));
			return { revision: h.snapshot.revision, layout: { spread, frames, preflight: [] } };
		},
		async layoutMaster(id, master) {
			await call('layoutMaster', [id, master]);
			const h = find(id);
			const found = h.snapshot.book.masterPages?.find((m) => m.id === master);
			if (!found) throw { kind: 'noSuchMaster', message: `there is no master page \`${master}\`` };
			const frames = found.frames
				.filter((f) => f.type === 'text')
				.map((f) => ({ frame: f.id, lines: [], overset: false }));
			return { revision: h.snapshot.revision, layout: { spread: master, frames, preflight: [] } };
		},
		async importImage(id, path) {
			await call('importImage', [id, path]);
			find(id);
			return `assets/images/${path.split('/').pop() ?? 'picture'}`;
		},
		async assetData(id, asset) {
			await call('assetData', [id, asset]);
			find(id);
			return new Uint8Array([137, 80, 78, 71]).buffer;
		},
		async spellCheck(id, text, personal = []) {
			await call('spellCheck', [id, text, personal]);
			find(id);
			const ignored = new Set([...(find(id).snapshot.book.proofing?.ignore ?? []), ...personal]);
			const flagged = new Set(misspelt.map((w) => w.toLowerCase()));
			const found = [...text.matchAll(/[\p{L}'’]+/gu)]
				.filter((m) => flagged.has(m[0].toLowerCase()) && !ignored.has(m[0]))
				.map((m) => ({ start: m.index!, end: m.index! + m[0].length, word: m[0] }));
			return { supported: true, dictionary: 'en-CA', offer: null, misspellings: found };
		},
		async spellCheckBook(id, personal = []) {
			await call('spellCheckBook', [id, personal]);
			const book = find(id).snapshot.book;
			const ignored = new Set([...(book.proofing?.ignore ?? []), ...personal]);
			const flagged = new Set(misspelt.map((w) => w.toLowerCase()));
			const words: BookSpelling['words'] = [];
			const look = (
				spread: string,
				frame: string,
				text: string,
				cell?: [number, number],
				replaceable = true,
			) => {
				for (const m of text.matchAll(/[\p{L}'’]+/gu)) {
					if (!flagged.has(m[0].toLowerCase()) || ignored.has(m[0])) continue;
					words.push({
						spread,
						frame,
						row: cell?.[0] ?? null,
						col: cell?.[1] ?? null,
						word: m[0],
						start: m.index!,
						end: m.index! + m[0].length,
						context: text.slice(Math.max(0, m.index! - 24), m.index! + m[0].length + 24).trim(),
						replaceable,
					});
				}
			};
			const plain = (c?: { inline: { text: string }[] } | null) =>
				c?.inline.map((r) => r.text).join('') ?? '';
			for (const spread of book.spreads) {
				for (const f of spread.frames) {
					if (f.type === 'text' || f.type === 'shape') look(spread.id, f.id, plain(f.content));
					if (f.type === 'table') {
						f.rows.forEach((row, r) =>
							row.forEach((cell, c) => look(spread.id, f.id, plain(cell.content), [r, c])),
						);
					}
				}
			}
			return { supported: true, dictionary: 'en-CA', offer: null, words, truncated: false };
		},
		async spellSuggest(id, word) {
			await call('spellSuggest', [id, word]);
			find(id);
			return [word.slice(0, -1), `${word}s`];
		},
		async dictionaries() {
			await call('dictionaries');
			return listing();
		},
		async downloadDictionary(id) {
			await call('downloadDictionary', [id]);
			// A download reports a quarter at a time, as the real one reports as it goes.
			for (const received of [250, 500, 750, 1000]) {
				for (const hear of progress) hear({ id, received, total: 1000 });
			}
			downloaded.add(id);
			return listing();
		},
		async onDictionaryProgress(listener) {
			progress.add(listener);
			return () => progress.delete(listener);
		},
		async removeDictionary(id) {
			await call('removeDictionary', [id]);
			downloaded.delete(id);
			return listing();
		},
		async readerPreview(id) {
			await call('readerPreview', [id]);
			find(id);
			const preview: ReaderPreview = {
				title: 'Pip',
				language: 'en-CA',
				css: 'p { margin: 0 }',
				chapters: [{ title: 'One', body: '<h1>One</h1><p class="body">Down by the reeds.</p>' }],
				images: [],
			};
			return preview;
		},
		async exportBook(id, targets) {
			await call('exportBook', [id, targets]);
			find(id);
			const names: Record<string, string[]> = {
				sharingPdf: ['pip-share.pdf'],
				kdpInterior: ['pip-paperback-interior.pdf'],
				printShopPdf: ['pip-print-pdfx4.pdf'],
				kindleEpub: ['pip-kindle.epub'],
				koboEpub: ['pip.epub', 'pip.kepub.epub'],
			};
			const report: ExportReport = {
				folder: '/home/mom/Books/Pip/exports',
				files: targets.flatMap((target) =>
					(names[target] ?? []).map((name) => ({
						target,
						path: `/home/mom/Books/Pip/exports/${name}`,
						bytes: 123456,
					})),
				),
			};
			return report;
		},
		async revealPath(path) {
			await call('revealPath', [path]);
		},
		async preflightBook(id) {
			await call('preflightBook', [id]);
			find(id);
			return [...preflightItems];
		},
		async listFonts(id) {
			await call('listFonts', [id]);
			const kept = new Set(find(id).snapshot.book.fonts?.map((f) => f.family) ?? []);
			return [
				...['Literata', 'Source Serif 4', 'Inter', 'Atkinson Hyperlegible'].map((family) => ({
					family,
					origin: 'bundled' as const,
				})),
				...[...kept].map((family) => ({ family, origin: 'book' as const })),
				...installedFonts
					.filter((family) => !kept.has(family))
					.map((family) => ({ family, origin: 'system' as const })),
			];
		},
		async keepFonts(id, families) {
			await call('keepFonts', [id, families]);
			const book = find(id);
			const report: KeepFontsReport = { kept: [], skipped: [] };
			const fonts = [...(book.snapshot.book.fonts ?? [])];
			for (const family of families) {
				if (fonts.some((f) => f.family === family)) report.kept.push(family);
				else if (installedFonts.includes(family)) {
					fonts.push({
						family,
						weight: 400,
						italic: false,
						path: `assets/fonts/${family.toLowerCase().replace(/\W+/g, '')}.ttf`,
						index: 0,
					});
					report.kept.push(family);
				} else if (
					!['Literata', 'Source Serif 4', 'Inter', 'Atkinson Hyperlegible'].includes(family)
				) {
					report.skipped.push({ family, reason: 'notInstalled' });
				}
			}
			if (fonts.length !== (book.snapshot.book.fonts?.length ?? 0)) {
				const revision = book.snapshot.revision + 1;
				const result: Patch = { revision, ops: [{ op: 'setFonts', fonts }] };
				book.past += 1;
				book.snapshot = {
					...book.snapshot,
					revision,
					book: applyPatch(book.snapshot.book, result),
					canUndo: true,
					canRedo: false,
				};
				const event: PatchEvent = { id, patch: result, canUndo: true, canRedo: false };
				listeners.forEach((l) => l.onPatch(event));
			}
			return report;
		},
		async subscribe(listener) {
			await call('subscribe');
			listeners.add(listener);
			return () => listeners.delete(listener);
		},
	};

	return {
		client,
		calls,
		emit: {
			patch: (event) => listeners.forEach((l) => l.onPatch(event)),
			changed,
		},
		refuseNext: (name, kind, message) => refusals.set(name, { kind, message }),
		misspelt,
		installedFonts,
		preflightItems,
		recoveries,
	};
}
