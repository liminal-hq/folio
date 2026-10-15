// The layout of one spread from Rust, read again whenever the book it came from changes
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { SpreadLayout } from '@liminal-hq/folio-protocol/generated/SpreadLayout';
import { useEffect, useState } from 'react';
import type { BooksClient } from '../books/booksClient';

/**
 * The laid-out spread, or `null` until the first one arrives. The previous layout stays up while a
 * newer one is read, so the page does not flash empty on every edit, and an answer that comes back
 * after a still newer request has been made is dropped.
 */
export function useSpreadLayout(
	client: BooksClient,
	bookId: string,
	spread: string | null,
	revision: number,
	/** Whether `spread` is a master page's id, laid out with its tokens as written. */
	master = false,
): SpreadLayout | null {
	const [held, setHeld] = useState<{ key: string; layout: SpreadLayout } | null>(null);

	useEffect(() => {
		if (spread === null) return;
		let current = true;
		(master ? client.layoutMaster(bookId, spread) : client.layoutSpread(bookId, spread))
			.then((result) => {
				if (current) setHeld({ key: `${bookId}/${spread}`, layout: result.layout });
			})
			.catch((error: unknown) => {
				// The book was closed, or this is not Tauri: keep what is drawn, and say why in the log.
				console.warn(`Could not lay out spread ${spread}:`, error);
			});
		return () => {
			current = false;
		};
	}, [client, bookId, spread, revision, master]);

	return held !== null && held.key === `${bookId}/${spread}` ? held.layout : null;
}
