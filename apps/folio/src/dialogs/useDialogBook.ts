// The book a dialog window works on, and a way to change it that finishes before the window closes
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { Intent } from '@liminal-hq/folio-protocol/generated/Intent';
import type { Book } from '@liminal-hq/folio-protocol/generated/Book';
import { useCallback, useRef } from 'react';
import { useActiveBook, useBooks } from '../books/useBooks';
import type { BooksClient } from '../books/booksClient';
import type { DialogSession } from './DialogHost';

export interface DialogBook {
	id: string;
	book: Book;
	client: BooksClient;
	/** Changes the book through Rust, so the main window shows it at once (one undo step of the book). */
	apply(intent: Intent): void;
	/** Closes the window once the changes made so far have reached Rust. */
	close(): void;
}

/** The pinned book of a dialog window, or null until it has been read. */
export function useDialogBook(session: DialogSession): DialogBook | null {
	const { store, client } = useBooks();
	const active = useActiveBook();
	const pending = useRef<Promise<unknown>[]>([]);
	const apply = useCallback(
		(intent: Intent) => {
			pending.current.push(store.apply(intent));
		},
		[store],
	);
	const close = useCallback(() => {
		void Promise.allSettled(pending.current).then(() => session.close());
	}, [session]);
	if (!active) return null;
	return { id: active.summary.id, book: active.copy.book, client, apply, close };
}
