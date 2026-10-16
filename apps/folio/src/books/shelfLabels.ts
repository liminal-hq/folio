// How the page shelf names a book's spreads, by their page numbers
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { Book } from '@liminal-hq/folio-protocol/generated/Book';
import { pageLabels } from './pageLabels';

/**
 * A label for each spread: "Cover", then "Page 1" for a page that opens on the right, and "Pages 2–3"
 * for a pair, in the numbering of the page's section (Roman for the front matter). A spread whose
 * section shows no numbers is called by the section's name.
 */
export function shelfLabels(book: Book): string[] {
	const labels = pageLabels(book.spreads);
	return book.spreads.map((spread, index) => {
		if (spread.kind === 'cover') return 'Cover';
		const pages = labels[index]!;
		const first = pages[0]?.label ?? '';
		if (first === '') return pages[0]?.section || 'Unnumbered';
		const last = pages[pages.length - 1]!.label;
		return pages.length === 1 ? `Page ${first}` : `Pages ${first}–${last}`;
	});
}
