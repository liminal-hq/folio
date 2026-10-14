// React access to the window's open books
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { BookSummary } from '@liminal-hq/folio-plugin-books';
import { createContext, useContext } from 'react';
import { useStore } from 'zustand';
import type { BookActions } from './bookActions';
import type { BooksClient } from './booksClient';
import { activeBook, type BooksState, type BooksStore, type OpenBookCopy } from './booksStore';

export interface BooksContextValue {
	store: BooksStore;
	client: BooksClient;
	actions: BookActions;
}

export const BooksContext = createContext<BooksContextValue | null>(null);

export function useBooksContext(): BooksContextValue {
	const value = useContext(BooksContext);
	if (value === null) throw new Error('useBooks needs a BooksProvider above it');
	return value;
}

/** A slice of the books state, re-rendering only when it changes. */
export function useBooksSelector<T>(select: (state: BooksState) => T): T {
	return useStore(useBooksContext().store.state, select);
}

/** The book being shown with its summary, or `null` when there is none. */
export function useActiveBook(): { summary: BookSummary; copy: OpenBookCopy } | null {
	const state = useBooksSelector((s) => s);
	return activeBook(state);
}

/** Every open book, from every window. */
export function useBookSummaries(): BookSummary[] {
	return useBooksSelector((s) => s.summaries);
}

/** What can be done with books: the store's calls, the file actions, and the client for presets. */
export function useBooks(): BooksContextValue {
	return useBooksContext();
}
