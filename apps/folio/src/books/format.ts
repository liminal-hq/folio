// How the status bar writes a book's size and length
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { Book } from '@liminal-hq/folio-protocol/generated/Book';

const POINTS_PER_INCH = 72;
const MM_PER_INCH = 25.4;

/** A number with at most `places` decimals and no trailing zeros: `8.5`, `8`, `215.9`. */
function plain(value: number, places: number): string {
	return String(Number(value.toFixed(places)));
}

/** A trim size in the units the person reads: `8.5 × 8.5 in`, or `216 × 216 mm`. */
export function formatTrim(trim: Book['trim'], units: 'inches' | 'millimetres'): string {
	if (units === 'millimetres') {
		const mm = (points: number) => plain((points / POINTS_PER_INCH) * MM_PER_INCH, 1);
		return `${mm(trim.w)} × ${mm(trim.h)} mm`;
	}
	const inches = (points: number) => plain(points / POINTS_PER_INCH, 2);
	return `${inches(trim.w)} × ${inches(trim.h)} in`;
}

/** A length in points as the person reads it: `0.125 in`, or `3.2 mm`. */
export function formatLength(points: number, units: 'inches' | 'millimetres'): string {
	if (units === 'millimetres') return `${plain((points / POINTS_PER_INCH) * MM_PER_INCH, 1)} mm`;
	return `${plain(points / POINTS_PER_INCH, 3)} in`;
}

/** The number of pages, not counting the cover (as `Book::page_count` in Rust). */
export function pageCount(book: Book): number {
	return book.spreads
		.filter((spread) => spread.kind !== 'cover')
		.reduce((total, spread) => total + spread.pages.length, 0);
}
