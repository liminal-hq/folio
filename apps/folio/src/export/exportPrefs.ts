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
