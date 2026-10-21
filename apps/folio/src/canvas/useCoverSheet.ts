// The cover as the printed sheet: the generated barcode, spine and guides, and how its words lay out
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { Frame } from '@liminal-hq/folio-protocol/generated/Frame';
import type { Guide } from '@liminal-hq/folio-protocol/generated/Guide';
import type { SpreadLayout } from '@liminal-hq/folio-protocol/generated/SpreadLayout';
import { useEffect, useState } from 'react';
import type { BooksClient } from '../books/booksClient';

/** What Folio draws on the cover itself, and the layout of the whole sheet. */
export interface CoverSheet {
	/** The barcode box and bars and the spine's title: drawn, but never the person's to move. */
	frames: Frame[];
	/** The folds and the safe areas. */
	guides: Guide[];
	layout: SpreadLayout;
}

/**
 * The cover sheet of the book, drawn again each time the book changes, while `on`. The one on show
 * stays until the next arrives, so nothing blinks as the person edits.
 */
export function useCoverSheet(
	client: BooksClient,
	bookId: string,
	revision: number,
	on: boolean,
	/** The ids of the frames the book itself holds on the cover. */
	own: ReadonlySet<string>,
): CoverSheet | null {
	const [held, setHeld] = useState<{ book: string; sheet: CoverSheet } | null>(null);

	useEffect(() => {
		if (!on) return;
		let current = true;
		client
			.coverSheetCanvas(bookId)
			.then((found) => {
				const spread = found?.book.spreads[0];
				if (!current || !found || !spread) return;
				setHeld({
					book: bookId,
					sheet: {
						frames: spread.frames.filter((f) => !own.has(f.id)),
						guides: spread.guides,
						layout: found.layout,
					},
				});
			})
			.catch((error: unknown) => {
				console.warn('Could not draw the cover sheet:', error);
			});
		return () => {
			current = false;
		};
		// `own` is the cover's own frames, which change with the revision.
		// eslint-disable-next-line react-hooks/exhaustive-deps
	}, [client, bookId, revision, on]);

	return on && held !== null && held.book === bookId ? held.sheet : null;
}
