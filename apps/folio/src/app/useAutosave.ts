// Asks the books plugin, every so often, to keep a recovery copy of each book with changes
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { useEffect } from 'react';
import type { BooksClient } from '../books/booksClient';
import { useBooks } from '../books/useBooks';
import { useSettings } from '../settings/useSettings';

/**
 * While the window is open and the setting is on, a recovery copy is asked for every interval (D-27).
 * The plugin writes only for books with changes it has no copy of, so asking is cheap. A failed
 * write is the plugin's to log; there is nothing for the page to do but ask again next time.
 */
export function useAutosave(client?: Pick<BooksClient, 'autosave'>): void {
	const books = useBooks().client;
	const target = client ?? books;
	const { enabled, intervalSecs } = useSettings().autosave;
	useEffect(() => {
		if (!enabled) return;
		const timer = setInterval(
			() => void target.autosave().catch(() => undefined),
			intervalSecs * 1000,
		);
		return () => clearInterval(timer);
	}, [target, enabled, intervalSecs]);
}
