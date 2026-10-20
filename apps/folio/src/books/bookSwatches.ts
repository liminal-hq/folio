// The book's swatches as the colour pickers and lists offer them, and ink as a screen shows it
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { Book } from '@liminal-hq/folio-protocol/generated/Book';
import type { Ink } from '@liminal-hq/folio-protocol/generated/Ink';
import type { PaintPresets } from '@liminal-hq/folio-protocol/generated/PaintPresets';

/** A colour a picker offers: its name, the value it sets, and what to draw it as when that differs. */
export interface PickerSwatch {
	name: string;
	colour: string;
	/** The colour to draw the chip in, for a swatch of the book (`colour` is its reference). */
	shown?: string;
}

/** The presets of the app, then the swatches of the book, as a picker lists them. */
export function pickerSwatches(
	book: Book,
	presets: PaintPresets | null | undefined,
): PickerSwatch[] {
	return [
		...(presets?.swatches ?? []),
		...(book.styles?.swatches ?? []).map((s) => ({
			name: s.name,
			colour: `swatch:${s.id}`,
			shown: s.display,
		})),
	];
}

const hex = (n: number) => Math.round(n).toString(16).padStart(2, '0');

/**
 * Ink as a screen shows it: the plain inverse, `(1 - plate)(1 - k)`. The twin of `Ink::to_rgb`
 * in `folio-model`, for the preview beside the fields before the swatch is added; the book keeps
 * the colour Rust works out.
 */
export function inkToHex(ink: Ink): string {
	const k = 1 - ink.k / 100;
	const plate = (percent: number) => 255 * (1 - percent / 100) * k;
	return `#${hex(plate(ink.c))}${hex(plate(ink.m))}${hex(plate(ink.y))}`;
}

/** An id for a new swatch named `name`, that no swatch of the book already has. */
export function newSwatchId(book: Book, name: string): string {
	const base =
		name
			.toLowerCase()
			.replace(/[^a-z0-9]+/g, '-')
			.replace(/^-+|-+$/g, '')
			.slice(0, 40) || 'swatch';
	const taken = new Set((book.styles?.swatches ?? []).map((s) => s.id));
	if (!taken.has(base)) return base;
	for (let n = 2; ; n += 1) {
		if (!taken.has(`${base}-${n}`)) return `${base}-${n}`;
	}
}
