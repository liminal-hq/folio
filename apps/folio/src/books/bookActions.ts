// What the menus and keys do with books: save, save as, open, and close with a question
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { BookSummary } from '@liminal-hq/folio-plugin-books';
import type { BooksClient } from './booksClient';
import { activeBook, type BooksStore, type OpenBookCopy } from './booksStore';
import type { FileDialogs } from './dialogs';

/** What was done: the person may cancel a picker, or a save may be refused. */
export type ActionResult = 'done' | 'cancelled' | 'failed';

/** What the person chose when asked about changes before closing. */
export type CloseChoice = 'save' | 'discard' | 'cancel';

/**
 * Whether closing a book would lose anything worth asking about. A saved book asks when it has
 * changes. A book never saved asks only once it has been touched: starting a book and closing it
 * again at once costs nothing.
 */
export function needsAsking(summary: BookSummary, copy: OpenBookCopy): boolean {
	return summary.path === null ? copy.revision > 0 : summary.dirty;
}

/** A path with `.folio` on the end, as the pickers on some systems do not add it. */
export function withExtension(path: string): string {
	return /\.folio$/i.test(path) ? path : `${path}.folio`;
}

export interface BookActions {
	save(): Promise<ActionResult>;
	saveAs(): Promise<ActionResult>;
	/** Opens a book from the picker, or from `path` when it is given (the recents list). */
	open(path?: string): Promise<ActionResult>;
	/** Closes the book being shown, asking `decide` first when it has changes worth keeping. */
	close(decide: (summary: BookSummary) => Promise<CloseChoice>): Promise<ActionResult>;
}

export function createBookActions(
	store: BooksStore,
	client: BooksClient,
	dialogs: FileDialogs,
): BookActions {
	const shown = () => activeBook(store.state.getState());

	const saveAs = async (): Promise<ActionResult> => {
		const book = shown();
		if (!book) return 'failed';
		const start = book.summary.path ?? (await client.defaultSavePath(book.summary.title));
		const picked = await dialogs.pickSavePath(start);
		if (picked === null) return 'cancelled';
		return (await store.save(book.summary.id, withExtension(picked))) ? 'done' : 'failed';
	};

	const save = async (): Promise<ActionResult> => {
		const book = shown();
		if (!book) return 'failed';
		if (book.summary.path === null) return saveAs();
		return (await store.save(book.summary.id)) ? 'done' : 'failed';
	};

	return {
		save,
		saveAs,
		async open(path) {
			const target = path ?? (await dialogs.pickBook());
			if (target === null) return 'cancelled';
			return (await store.openBook(target)) ? 'done' : 'failed';
		},
		async close(decide) {
			const book = shown();
			if (!book) return 'failed';
			if (needsAsking(book.summary, book.copy)) {
				const choice = await decide(book.summary);
				if (choice === 'cancel') return 'cancelled';
				if (choice === 'save') {
					const saved = await save();
					if (saved !== 'done') return saved;
				}
			}
			await store.close(book.summary.id);
			return 'done';
		},
	};
}
