// Export choices as the page handles them: which layer says what, and putting them together
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { ExportPrefs } from '@liminal-hq/folio-protocol/generated/ExportPrefs';
import type { ExportTarget } from '@liminal-hq/folio-protocol/generated/ExportTarget';
import type { PdfPrefs } from '@liminal-hq/folio-protocol/generated/PdfPrefs';
import type { PicturePreset } from '@liminal-hq/folio-protocol/generated/PicturePreset';

/** The three kinds of PDF whose choices are kept apart. */
export type PdfKind = 'sharing' | 'kdp' | 'printShop';

/** Which kind of PDF a target makes, or `null` for a file that is not a PDF. */
export function kindOf(target: ExportTarget): PdfKind | null {
	switch (target) {
		case 'sharingPdf':
			return 'sharing';
		case 'kdpInterior':
		case 'kdpCover':
			return 'kdp';
		case 'printShopPdf':
			return 'printShop';
		default:
			return null;
	}
}

/** What nothing chosen means for the pictures of each kind; the twin of Rust's built-in settings. */
export const BUILTIN_PRESET: Record<PdfKind, PicturePreset> = {
	sharing: 'standard',
	kdp: 'high',
	printShop: 'high',
};

/** The presets a kind of PDF offers: a press file keeps at least 300 pixels an inch. */
export const PRESETS: Record<PdfKind, readonly PicturePreset[]> = {
	sharing: ['small', 'standard', 'high', 'original'],
	kdp: ['high', 'original'],
	printShop: ['high', 'original'],
};

export function emptyPrefs(): ExportPrefs {
	return { pdf: { sharing: {}, kdp: {}, printShop: {} } };
}

/** `value` without the keys that hold nothing. */
function defined<T extends object>(value: T): T {
	return Object.fromEntries(Object.entries(value).filter(([, v]) => v !== undefined)) as T;
}

/** `prefs` with `change` laid over the choices of one kind of PDF; an `undefined` takes one back. */
export function withPdf(prefs: ExportPrefs, kind: PdfKind, change: Partial<PdfPrefs>): ExportPrefs {
	return {
		...prefs,
		pdf: { ...prefs.pdf, [kind]: defined({ ...prefs.pdf[kind], ...change }) },
	};
}

/** `over`'s choices laid over `base`'s: each one `over` makes wins, the rest are `base`'s. */
export function overlay(base: ExportPrefs | undefined, over: ExportPrefs | undefined): ExportPrefs {
	const low = base ?? emptyPrefs();
	const high = over ?? emptyPrefs();
	const pdf = (kind: PdfKind): PdfPrefs => defined({ ...low.pdf?.[kind], ...high.pdf?.[kind] });
	const result: ExportPrefs = {
		...low,
		...defined(high),
		pdf: { sharing: pdf('sharing'), kdp: pdf('kdp'), printShop: pdf('printShop') },
	};
	return result;
}

/** The choices that count for `kind`, from layers given lowest first. */
export function effectivePdf(kind: PdfKind, ...layers: (ExportPrefs | undefined)[]): PdfPrefs {
	return layers.reduce<PdfPrefs>((now, layer) => defined({ ...now, ...layer?.pdf?.[kind] }), {});
}

/** Whether `prefs` hold no choice at all. */
export function isBlank(prefs: ExportPrefs): boolean {
	const blank = (value: unknown): boolean =>
		value === undefined ||
		(typeof value === 'object' &&
			value !== null &&
			!Array.isArray(value) &&
			Object.values(value).every(blank));
	return blank(prefs);
}

/** What nothing chosen means for each choice of a kind of PDF; the twin of Rust's built-in settings. */
export function builtinPdf(kind: PdfKind): PdfPrefs {
	return {
		preset: BUILTIN_PRESET[kind],
		compression: 'auto',
		keepJpeg: false,
		bookmarks: kind === 'sharing' ? 'auto' : 'none',
		pageLabels: kind === 'sharing',
		links: kind === 'sharing',
		viewerLayout: 'single',
		sheets: 'pages',
		cover: kind === 'sharing' ? 'frontAndBackEnds' : 'front',
		cropMarks: kind === 'printShop',
		bleed: { kind: 'book' },
		ink: false,
	};
}

/** The ppi and quality a preset means, for showing what an empty field stands for. */
export const PRESET_FACTS: Record<PicturePreset, { ppi: number | null; quality: number }> = {
	small: { ppi: 100, quality: 60 },
	standard: { ppi: 150, quality: 80 },
	high: { ppi: 300, quality: 90 },
	original: { ppi: null, quality: 95 },
};

/** The choices of a kind of PDF that its destination fixes, each with why. */
export type LockedField = keyof PdfPrefs;

const NO_EXTRAS: LockedField[] = [
	'bookmarks',
	'pageLabels',
	'links',
	'viewerLayout',
	'subject',
	'keywords',
];
const WHOLE_BOOK: LockedField[] = ['sheets', 'range', 'cover'];
const NO_MARKS: LockedField[] = ['cropMarks', 'bleed', 'slug'];
const SHOP_ONLY: LockedField[] = ['cropMarks', 'bleed', 'slug'];

export type LockReason =
	| 'export.locked.kdp.info'
	| 'export.locked.pages'
	| 'export.locked.marks'
	| 'export.locked.links'
	| 'export.locked.kdpInk';

/** The message that says why `field` is fixed for `kind`, or `null` when it is free. */
export function lockedBecause(kind: PdfKind, field: LockedField): LockReason | null {
	if (kind === 'kdp') {
		if (NO_EXTRAS.includes(field)) return 'export.locked.kdp.info';
		if (WHOLE_BOOK.includes(field)) return 'export.locked.pages';
		if (NO_MARKS.includes(field)) return 'export.locked.marks';
		if (field === 'ink' || field === 'profile') return 'export.locked.kdpInk';
	}
	if (kind === 'printShop') {
		if (field === 'links') return 'export.locked.links';
		if (WHOLE_BOOK.includes(field)) return 'export.locked.pages';
	}
	if (kind === 'sharing' && (SHOP_ONLY.includes(field) || field === 'profile' || field === 'ink')) {
		return 'export.locked.marks';
	}
	return null;
}
