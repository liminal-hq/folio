// What the ribbon's document controls do: which are live for the selection, and what each one asks for
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { TableEdit } from '@liminal-hq/folio-protocol/generated/TableEdit';
import type { FontChoice } from '@liminal-hq/folio-protocol/generated/FontChoice';
import type { Frame } from '@liminal-hq/folio-protocol/generated/Frame';
import type { Intent } from '@liminal-hq/folio-protocol/generated/Intent';
import type { Spread } from '@liminal-hq/folio-protocol/generated/Spread';
import type { SpreadLayout } from '@liminal-hq/folio-protocol/generated/SpreadLayout';
import type { TextSelection } from '../canvas/textSelection';
import type { Marks } from '../canvas/marksModel';
import { surfaceOf } from '../canvas/masterView';
import type { NewFrameKind } from '../canvas/newFrames';
import { t, type MessageKey } from '../i18n/messages';
import type { StyleChoice } from '../inspector/styleChoices';
import { effectiveSize, withOverrides } from '../inspector/textFacts';

export interface EditorInput {
	spread: Spread;
	/** Where the spread is on the shelf, and how many there are. */
	spreadIndex: number;
	spreadCount: number;
	selected: Frame[];
	layout: SpreadLayout | null;
	choices: StyleChoice[];
	/** The typefaces the book can draw in, for the Font list. */
	fonts?: FontChoice[];
	marks: Marks;
	snap: boolean;
	/** Whether the rulers show. */
	rulers: boolean;
	/** What the stage shows: the pages as printed, or as a Kindle or Kobo shows them. */
	preview?: 'print' | 'kindle' | 'kobo';
	/** The cell being edited in the selected table, which says where a row or column goes. */
	cell?: { row: number; col: number } | null;
	/** Whether `spread` is a master page being edited, not one of the book's spreads. */
	master?: boolean;
	/** The swatches of the book, offered after the colours of the Text colour list. */
	swatches?: { id: string; name: string }[];
	/** The words selected in the open text editor: bold, italic and the font apply to just those. */
	selection?: TextSelection | null;
}

/** What the controls show: which can be used now, which are pressed, the numbers, the extra tabs. */
export interface EditorState {
	available: ReadonlySet<string>;
	active: Readonly<Record<string, boolean>>;
	values: Readonly<Record<string, number>>;
	/** The text a list shows as chosen, by control: the font of the selected words. */
	texts: Readonly<Record<string, string>>;
	/** The choices of a list, by control, as value and label. */
	options: Readonly<Record<string, ReadonlyArray<readonly [string, string]>>>;
	contextual: readonly string[];
}

/** What a control asks the book view to do. */
export type EditorAction =
	| { kind: 'intents'; intents: Intent[] }
	| { kind: 'font'; intents: Intent[]; keep: string | null }
	| { kind: 'add'; shape: NewFrameKind }
	| { kind: 'marks'; marks: Partial<Marks> }
	| { kind: 'snap' }
	| { kind: 'rulers' }
	| { kind: 'spelling' }
	| { kind: 'zoom'; zoom: 'in' | 'out' | 'fit' }
	| { kind: 'page'; op: 'add' | 'duplicate' | 'remove' }
	| { kind: 'choosePicture'; frame: string }
	| { kind: 'stretch'; frame: string }
	| { kind: 'fit'; frame: string; fit: 'fill' | 'fit' }
	| { kind: 'preview'; mode: 'print' | 'kindle' | 'kobo' }
	| { kind: 'checkBook' }
	| { kind: 'templates' }
	| { kind: 'softProof' }
	| { kind: 'coverSheet' }
	| { kind: 'barcode' }
	| { kind: 'bookDetails' }
	| { kind: 'exportBook'; quick: boolean }
	| { kind: 'editMaster' }
	| { kind: 'pageBackground' };

const ALIGNS: Record<string, 'left' | 'center' | 'right' | 'justify'> = {
	alignL: 'left',
	alignC: 'center',
	alignR: 'right',
	justify: 'justify',
};

const texts = (frames: Frame[]) =>
	frames.flatMap((f) => (f.type === 'text' && !f.locked && !f.fromMaster ? [f] : []));
/** The selected stretch of words, when it is in one of the selected text frames or shapes. */
function wordsSelected(input: EditorInput) {
	const { selection } = input;
	if (!selection || selection.end <= selection.start) return null;
	const frame = input.selected.find((f) => f.id === selection.frame);
	return frame && (frame.type === 'text' || frame.type === 'shape') && !frame.locked
		? { frame, selection }
		: null;
}

/** Whether every character of `start..end` is in a run where `has` is true. */
function allRuns(
	frame: Extract<Frame, { type: 'text' | 'shape' }>,
	start: number,
	end: number,
	has: (run: { bold: boolean; italic: boolean }) => boolean,
): boolean {
	let at = 0;
	for (const run of frame.content?.inline ?? []) {
		const length = [...run.text].length;
		const overlaps = at < end && at + length > start;
		if (overlaps && !has(run)) return false;
		at += length;
	}
	return true;
}

/** The colours the Text colour list offers: ink first, then colours that read well on paper. */
const TEXT_COLOURS: ReadonlyArray<readonly [hex: string, name: MessageKey]> = [
	['#1c1b20', 'colour.ink'],
	['#5b2a4a', 'colour.plum'],
	['#1f4e5f', 'colour.teal'],
	['#2f5d3a', 'colour.forest'],
	['#a23b2a', 'colour.brick'],
	['#c98b2b', 'colour.marigold'],
	['#2b3a67', 'colour.navy'],
	['#4a4a4a', 'colour.slate'],
	['#ffffff', 'colour.white'],
];

const free = (frames: Frame[]) => frames.filter((f) => !f.locked && !f.fromMaster);

export function editorState(input: EditorInput): EditorState {
	const { spread, selected, marks, snap, layout, choices } = input;
	const available = new Set<string>([
		'text',
		'image',
		'table',
		'bubble',
		'rect',
		'round',
		'ellipse',
		'line',
		'speech',
		'thought',
		'shout',
		'addPage',
		'margins',
		'grid',
		'bleed',
		'guides',
		'rulers',
		'spell',
		'rguides',
		'snap',
		'zoomIn',
		'zoomOut',
		'fit',
	]);
	const active: Record<string, boolean> = {
		margins: marks.margins,
		grid: marks.columns || marks.baseline,
		bleed: marks.bleed,
		guides: marks.frameEdges,
		rulers: input.rulers,
		rguides: marks.guides,
		snap,
	};
	const values: Record<string, number> = {};
	const shown: Record<string, string> = {};
	const contextual: string[] = [];

	if (input.master) {
		available.delete('addPage');
	} else {
		if (spread.kind !== 'cover') {
			available.add('dupPage');
			if (input.spreadCount > 1) available.add('delPage');
		}
		if (spread.guides.some((g) => !g.locked)) available.add('rgClear');
	}
	if (input.master) active.master = true;
	available.add('master');
	available.add('preflight');
	if (!input.master) available.add('templates');
	available.add('softProof');
	available.add('coverSheet');
	available.add('barcode');
	available.add('bookDetails');
	for (const id of ['print', 'kindle', 'kobo']) available.add(id);
	active[input.preview ?? 'print'] = true;
	available.add('export');
	available.add('quickPdf');
	// A master page has no spread to paint; its background is set from the pages that use it.
	if (!input.master) available.add('pageBg');
	if (selected.length > 0) available.add('lock');
	if (selected.length > 0 && selected.every((f) => f.locked)) active.lock = true;
	if (free(selected).length > 0) {
		available.add('front');
		available.add('back');
	}
	const words = texts(selected);
	if (words.length > 0) {
		for (const id of ['bold', 'italic', ...Object.keys(ALIGNS), 'size']) available.add(id);
		const first = words[0]!;
		active.bold = first.overrides?.bold === true;
		active.italic = first.overrides?.italic === true;
		const align = first.overrides?.align ?? 'left';
		for (const [id, value] of Object.entries(ALIGNS)) active[id] = align === value;
		values.size = effectiveSize(first, layout, choices);
		available.add('font');
		available.add('colour');
		shown.font = first.overrides?.font ?? '';
	}
	const only = selected.length === 1 ? selected[0]! : null;
	if (only?.type === 'image') {
		contextual.push('picture');
		for (const id of ['replace', 'toBleed', 'fitFill']) available.add(id);
		// Bleed is square to the page, so a turned picture cannot be stretched to it.
		if (only.rotation) available.delete('toBleed');
		active.fitFill = only.image.fit === 'fit';
	}
	if (only?.type === 'shape') contextual.push('shape');
	if (only?.type === 'table') {
		contextual.push('table');
		if (!only.locked && !only.fromMaster) {
			for (const id of ['rowAdd', 'colAdd', 'headToggle', 'tableLined', 'tableGrid', 'tableRecipe'])
				available.add(id);
			if (only.rows.length > 1) available.add('rowDel');
			if (only.cols.length > 1) available.add('colDel');
		}
		active.headToggle = only.head > 0;
		active.tableLined = only.look === 'lined';
		active.tableGrid = only.look === 'grid';
		active.tableRecipe = only.look === 'recipe';
	}
	// Any one frame that is free to change can be turned.
	if (only && !only.locked && !only.fromMaster) {
		available.add('rotL');
		available.add('rotR');
	}
	const options: Record<string, ReadonlyArray<readonly [string, string]>> = {};
	options.colour = [
		...TEXT_COLOURS.map(([hex, name]) => [hex, t(name)] as const),
		...(input.swatches ?? []).map((s) => [`swatch:${s.id}`, s.name] as const),
	];
	if (input.fonts) options.font = input.fonts.map((f) => [f.family, f.family] as const);
	return { available, active, values, texts: shown, options, contextual };
}

/** The action a control asks for, or `null` when it is not one of the document's controls. */
export function editorAction(
	id: string,
	value: string | number | boolean | undefined,
	input: EditorInput,
): EditorAction | null {
	const { spread, selected } = input;
	const on = { on: surfaceOf(spread) };
	switch (id) {
		case 'print':
		case 'kindle':
		case 'kobo':
			return { kind: 'preview', mode: id };
		case 'preflight':
			return { kind: 'checkBook' };
		case 'templates':
			return { kind: 'templates' };
		case 'softProof':
			return { kind: 'softProof' };
		case 'coverSheet':
			return { kind: 'coverSheet' };
		case 'barcode':
			return { kind: 'barcode' };
		case 'bookDetails':
			return { kind: 'bookDetails' };
		case 'export':
			return { kind: 'exportBook', quick: false };
		case 'quickPdf':
			return { kind: 'exportBook', quick: true };
		case 'text':
			return { kind: 'add', shape: 'text' };
		case 'image':
			return { kind: 'add', shape: 'picture' };
		case 'table':
			return { kind: 'add', shape: 'table' };
		case 'rect':
		case 'round':
		case 'ellipse':
		case 'line':
		case 'speech':
		case 'thought':
		case 'shout':
			return { kind: 'add', shape: id };
		case 'bubble':
			return { kind: 'add', shape: 'speech' };
		case 'addPage':
			return { kind: 'page', op: 'add' };
		case 'dupPage':
			return { kind: 'page', op: 'duplicate' };
		case 'delPage':
			return { kind: 'page', op: 'remove' };
		case 'margins':
			return { kind: 'marks', marks: { margins: !input.marks.margins } };
		case 'bleed':
			return { kind: 'marks', marks: { bleed: !input.marks.bleed } };
		case 'guides':
			return { kind: 'marks', marks: { frameEdges: !input.marks.frameEdges } };
		case 'grid': {
			const next = !(input.marks.columns || input.marks.baseline);
			return { kind: 'marks', marks: { columns: next, baseline: next } };
		}
		case 'snap':
			return { kind: 'snap' };
		case 'rulers':
			return { kind: 'rulers' };
		case 'spell':
			return { kind: 'spelling' };
		case 'rguides':
			return { kind: 'marks', marks: { guides: !input.marks.guides } };
		case 'master':
			return { kind: 'editMaster' };
		case 'pageBg':
			return { kind: 'pageBackground' };
		case 'rgClear':
			return { kind: 'intents', intents: [{ type: 'ClearGuides', spread: spread.id }] };
		case 'zoomIn':
			return { kind: 'zoom', zoom: 'in' };
		case 'zoomOut':
			return { kind: 'zoom', zoom: 'out' };
		case 'fit':
			return { kind: 'zoom', zoom: 'fit' };
		case 'front':
		case 'back':
			return {
				kind: 'intents',
				intents: free(selected).map((f) => ({
					type: 'Reorder',
					...on,
					frame: f.id,
					to: id === 'front' ? 'front' : 'back',
				})),
			};
		case 'lock': {
			const lock = !selected.every((f) => f.locked);
			return {
				kind: 'intents',
				intents: selected
					.filter((f) => f.locked !== lock)
					.map((f) => ({ type: 'LockFrame', ...on, frame: f.id, locked: lock })),
			};
		}
		case 'bold':
		case 'italic': {
			const picked = wordsSelected(input);
			if (picked) {
				const { frame, selection } = picked;
				const on = allRuns(frame, selection.start, selection.end, (r) => r[id]);
				return {
					kind: 'intents',
					intents: [
						{
							type: 'StyleRange',
							frame: frame.id,
							start: selection.start,
							end: selection.end,
							change: { [id]: !on },
						},
					],
				};
			}
			const words = texts(selected);
			const turnOn = words[0]?.overrides?.[id] !== true;
			return {
				kind: 'intents',
				intents: words.map((f) => ({
					type: 'OverrideStyle',
					frame: f.id,
					overrides: withOverrides(f, { [id]: turnOn }),
				})),
			};
		}
		case 'alignL':
		case 'alignC':
		case 'alignR':
		case 'justify':
			return {
				kind: 'intents',
				intents: texts(selected).map((f) => ({
					type: 'OverrideStyle',
					frame: f.id,
					overrides: withOverrides(f, { align: ALIGNS[id]! }),
				})),
			};
		case 'colour': {
			// The selected words, or all of them when none are selected.
			const hex = typeof value === 'string' ? value : '';
			const picked = wordsSelected(input);
			const targets = picked
				? [{ id: picked.frame.id, start: picked.selection.start, end: picked.selection.end }]
				: texts(selected).map((f) => ({
						id: f.id,
						start: 0,
						end: [...(f.content?.inline ?? []).map((r) => r.text).join('')].length,
					}));
			return {
				kind: 'intents',
				intents: targets
					.filter((r) => r.end > r.start)
					.map((r) => ({
						type: 'StyleRange',
						frame: r.id,
						start: r.start,
						end: r.end,
						change: { colour: hex },
					})),
			};
		}
		case 'font': {
			const family = typeof value === 'string' ? value : '';
			const origin = input.fonts?.find((f) => f.family === family)?.origin;
			const picked = wordsSelected(input);
			if (picked) {
				const { frame, selection } = picked;
				return {
					kind: 'font',
					keep: origin === 'system' ? family : null,
					intents: [
						{
							type: 'StyleRange',
							frame: frame.id,
							start: selection.start,
							end: selection.end,
							change: { font: family },
						},
					],
				};
			}
			return {
				kind: 'font',
				// An installed font is kept with the book, so it opens the same elsewhere.
				keep: origin === 'system' ? family : null,
				intents: texts(selected).map((f) => {
					const { font: _was, ...rest } = withOverrides(f, {});
					return {
						type: 'OverrideStyle',
						frame: f.id,
						overrides: family === '' ? rest : { ...rest, font: family },
					};
				}),
			};
		}
		case 'size': {
			const size = typeof value === 'number' ? value : Number(value);
			if (!Number.isFinite(size)) return null;
			return {
				kind: 'intents',
				intents: texts(selected).map((f) => ({
					type: 'OverrideStyle',
					frame: f.id,
					overrides: withOverrides(f, { size: Math.min(200, Math.max(4, size)) }),
				})),
			};
		}
		case 'rowAdd':
		case 'rowDel':
		case 'colAdd':
		case 'colDel':
		case 'headToggle':
		case 'tableLined':
		case 'tableGrid':
		case 'tableRecipe': {
			const only = selected.length === 1 ? selected[0]! : null;
			if (only?.type !== 'table' || only.locked || only.fromMaster) return null;
			// Rows and columns go on and come off the end until a cell is chosen to say where.
			const edit: TableEdit | null =
				id === 'rowAdd'
					? { type: 'addRow', at: input.cell?.row ?? only.rows.length }
					: id === 'rowDel'
						? { type: 'removeRow', at: input.cell?.row ?? only.rows.length - 1 }
						: id === 'colAdd'
							? { type: 'addColumn', at: input.cell?.col ?? only.cols.length }
							: id === 'colDel'
								? { type: 'removeColumn', at: input.cell?.col ?? only.cols.length - 1 }
								: id === 'headToggle'
									? { type: 'setHead', head: only.head > 0 ? 0 : 1 }
									: {
											type: 'setLook',
											look: id === 'tableGrid' ? 'grid' : id === 'tableRecipe' ? 'recipe' : 'lined',
										};
			return {
				kind: 'intents',
				intents: [{ type: 'EditTable', frame: only.id, edit }],
			};
		}
		case 'rotL':
		case 'rotR': {
			const only = selected.length === 1 ? selected[0]! : null;
			if (!only || only.locked || only.fromMaster) return null;
			const turn = id === 'rotR' ? 90 : -90;
			return {
				kind: 'intents',
				intents: [{ type: 'RotateFrame', ...on, frame: only.id, degrees: only.rotation + turn }],
			};
		}
		case 'replace':
		case 'toBleed':
		case 'fitFill': {
			const only = selected.length === 1 ? selected[0]! : null;
			if (only?.type !== 'image' || only.locked) return null;
			if (id === 'replace') return { kind: 'choosePicture', frame: only.id };
			if (id === 'toBleed' && only.rotation) return null;
			if (id === 'toBleed') return { kind: 'stretch', frame: only.id };
			return { kind: 'fit', frame: only.id, fit: only.image.fit === 'fit' ? 'fill' : 'fit' };
		}
		default:
			return null;
	}
}
