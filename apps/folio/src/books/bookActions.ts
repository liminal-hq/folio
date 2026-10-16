// What the menus and keys do with books: save, save as, open, and close with a question
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { BookSummary, Recoverable } from '@liminal-hq/folio-plugin-books';
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

/** What the person chose when a book they open has unsaved work a crash left behind. */
export type RecoveredChoice = 'restore' | 'file' | 'cancel';

/** Whether two paths name the same file: `/` and `\` alike, and without case on a Windows drive. */
export function samePath(a: string, b: string): boolean {
	const plain = (p: string) => p.replace(/^\\\\\?\\/, '').replace(/\\/g, '/');
	const [x, y] = [plain(a), plain(b)];
	return /^[a-z]:\//i.test(x) ? x.toLowerCase() === y.toLowerCase() : x === y;
}

export interface BookActions {
	save(): Promise<ActionResult>;
	saveAs(): Promise<ActionResult>;
	/** Asks which book to open: its path, or `null` when the picker is cancelled. */
	pick(): Promise<string | null>;
	/**
	 * Opens a book from the picker, or from `path` when it is given (the recents list). When a crash
	 * left unsaved work on that file, `decideRecovered` asks whether to restore it or open the file as
	 * it was saved; without it the file is opened and the work is left for the start screen.
	 */
	open(
		path?: string,
		decideRecovered?: (found: Recoverable) => Promise<RecoveredChoice>,
	): Promise<ActionResult>;
	/** Brings back what a crash left, as the book it was, still unsaved. */
	restore(id: string): Promise<ActionResult>;
	/** Gives up what a crash left. */
	discard(id: string): Promise<void>;
	/** Closes the book being shown, asking `decide` first when it has changes worth keeping. */
	close(decide: (summary: BookSummary) => Promise<CloseChoice>): Promise<ActionResult>;
	/**
	 * Closes every open book, asking `decide` about each that has changes worth keeping, one after
	 * another. It stops at the first the person cancels or that cannot be saved, and then closes
	 * nothing: 'done' means the app may go.
	 */
	closeAll(decide: (summary: BookSummary) => Promise<CloseChoice>): Promise<ActionResult>;
}

export function createBookActions(
	store: BooksStore,
	client: BooksClient,
	dialogs: FileDialogs,
): BookActions {
	const shown = () => activeBook(store.state.getState());

	const find = (id: string) => {
		const state = store.state.getState();
		const summary = state.summaries.find((b) => b.id === id);
		const copy = state.copies[id];
		return summary && copy ? { summary, copy } : null;
	};

	const saveAsBook = async (id: string): Promise<ActionResult> => {
		const book = find(id);
		if (!book) return 'failed';
		const start = book.summary.path ?? (await client.defaultSavePath(book.summary.title));
		const picked = await dialogs.pickSavePath(start);
		if (picked === null) return 'cancelled';
		return (await store.save(book.summary.id, withExtension(picked))) ? 'done' : 'failed';
	};

	const saveBook = async (id: string): Promise<ActionResult> => {
		const book = find(id);
		if (!book) return 'failed';
		if (book.summary.path === null) return saveAsBook(id);
		return (await store.save(book.summary.id)) ? 'done' : 'failed';
	};

	const saveAs = async (): Promise<ActionResult> => {
		const book = shown();
		return book ? saveAsBook(book.summary.id) : 'failed';
	};

	const save = async (): Promise<ActionResult> => {
		const book = shown();
		return book ? saveBook(book.summary.id) : 'failed';
	};

	return {
		save,
		saveAs,
		pick: () => dialogs.pickBook(),
		async open(path, decideRecovered) {
			const target = path ?? (await dialogs.pickBook());
			if (target === null) return 'cancelled';
			if (decideRecovered) {
				const left = await client.recoverable().catch(() => [] as Recoverable[]);
				const found = left.find((r) => r.path !== null && samePath(r.path, target));
				if (found) {
					const choice = await decideRecovered(found);
					if (choice === 'cancel') return 'cancelled';
					if (choice === 'restore') return (await store.restoreBook(found.id)) ? 'done' : 'failed';
					// The file as saved was asked for, so the work that was left is given up.
					await client.discardRecovered(found.id).catch(() => undefined);
				}
			}
			return (await store.openBook(target)) ? 'done' : 'failed';
		},
		async restore(id) {
			return (await store.restoreBook(id)) ? 'done' : 'failed';
		},
		async discard(id) {
			await client.discardRecovered(id).catch(() => undefined);
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
		async closeAll(decide) {
			const open = [...store.state.getState().summaries];
			for (const { id } of open) {
				const book = find(id);
				if (!book || !needsAsking(book.summary, book.copy)) continue;
				const choice = await decide(book.summary);
				if (choice === 'cancel') return 'cancelled';
				if (choice === 'save') {
					const saved = await saveBook(id);
					if (saved !== 'done') return saved;
				}
			}
			// Everyone has been asked: the books go, so nothing is left behind for the next launch.
			for (const { id } of open) await store.close(id);
			return 'done';
		},
	};
}
