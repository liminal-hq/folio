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
	children,
}: {
	client: BooksClient;
	dialogs: FileDialogs;
	children: ReactNode;
}) {
	const value = useMemo(() => {
		const store = createBooksStore(client);
		return { store, client, actions: createBookActions(store, client, dialogs) };
	}, [client, dialogs]);

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
