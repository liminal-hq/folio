// How wide the cover is: the back, the spine and the front, as the printed sheet
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { Book } from '@liminal-hq/folio-protocol/generated/Book';

/** KDP's spine factor for each paper, in inches a page; mirrors `spine_factor` in `folio-model`. */
const SPINE_FACTOR = { white: 0.002252, cream: 0.0025, colour: 0.002347 } as const;

/** The bleed KDP wants round the cover sheet, in points (0.125 in). */
export const COVER_EDGE = 9;

/** How many pages KDP counts for the spine: the interior, in twos, and at least 24. */
export function spinePages(book: Book): number {
	const interior = book.spreads.slice(1).reduce((sum, spread) => sum + spread.pages.length, 0);
	return Math.max(24, interior + (interior % 2));
}

/** The thickness of the spine, in points. */
export function spineWidth(book: Book): number {
	return spinePages(book) * SPINE_FACTOR[book.paper ?? 'white'] * 72;
}

/** The width of the cover spread, from the back's left trim edge to the front's right one. */
export function coverWidth(book: Book): number {
	return book.trim.w * 2 + spineWidth(book);
}
