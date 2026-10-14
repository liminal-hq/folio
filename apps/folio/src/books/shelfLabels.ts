// How the page shelf names a book's spreads, by their page numbers
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { Book } from '@liminal-hq/folio-protocol/generated/Book';

/**
 * A label for each spread: "Cover", then "Page 1" for a page that opens on the right, and "Pages 2–3"
 * for a pair. Pages are numbered from the first page after the cover.
 */
export function shelfLabels(book: Book): string[] {
	let page = 0;
	return book.spreads.map((spread) => {
		if (spread.kind === 'cover') return 'Cover';
		const first = page + 1;
		page += spread.pages.length;
		return spread.pages.length === 1 ? `Page ${first}` : `Pages ${first}–${page}`;
	});
}
