// Starts the window's books store and shares it, and the file pickers, with the page
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { useEffect, useMemo, type ReactNode } from 'react';
import { createBookActions } from './bookActions';
import type { BooksClient } from './booksClient';
import { createBooksStore } from './booksStore';
import type { FileDialogs } from './dialogs';
import { BooksContext } from './useBooks';

export function BooksProvider({
	client,
	dialogs,
	follow = false,
	pin = null,
	children,
}: {
	client: BooksClient;
	dialogs: FileDialogs;
	/** Show the book the main window shows, rather than choosing one (the Book tools window). */
	follow?: boolean;
	/** Show the one book this id names, and never say which book is shown (a dialog window). */
	pin?: string | null;
	children: ReactNode;
}) {
	const value = useMemo(() => {
		const store =
			pin !== null
				? createBooksStore(client, 'pinned', pin)
				: createBooksStore(client, follow ? 'follow' : 'main');
		return { store, client, dialogs, actions: createBookActions(store, client, dialogs) };
	}, [client, dialogs, follow, pin]);

	useEffect(() => {
		let active = true;
		let stop: (() => void) | undefined;
		value.store
			.start()
			.then((off) => {
				if (active) stop = off;
				else off();
			})
			.catch(() => {
				// Not in Tauri: there are no books.
			});
		return () => {
			active = false;
			stop?.();
		};
	}, [value]);

	return <BooksContext.Provider value={value}>{children}</BooksContext.Provider>;
}
