// Reads and changes the open books through the books plugin's guest-js API
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import {
	applyIntent,
	closeBook,
	defaultSavePath,
	listBooks,
	newBook,
	onBooksChanged,
	onPatch,
	openBook,
	presets,
	redo,
	saveBook,
	snapshot,
	undo,
	type BookSnapshot,
	type BookSummary,
	type BooksChanged,
	type Intent,
	type KindInfo,
	type NewBookSpec,
	type Patch,
	type PatchEvent,
} from '@liminal-hq/folio-plugin-books';

/** Hears what happens to the open books, from any window. */
export interface BooksListener {
	onPatch(event: PatchEvent): void;
	onChanged(event: BooksChanged): void;
}

/** What the page needs from the books: Rust owns them, so every call goes through it. */
export interface BooksClient {
	presets(): Promise<KindInfo[]>;
	defaultSavePath(title: string): Promise<string>;
	newBook(spec: Partial<NewBookSpec>): Promise<BookSnapshot>;
	openBook(path: string): Promise<BookSnapshot>;
	saveBook(id: string, path?: string): Promise<BookSummary>;
	closeBook(id: string): Promise<void>;
	list(): Promise<BookSummary[]>;
	snapshot(id: string): Promise<BookSnapshot>;
	apply(id: string, intent: Intent): Promise<Patch>;
	undo(id: string): Promise<Patch | null>;
	redo(id: string): Promise<Patch | null>;
	/** Resolves to the function that stops listening. */
	subscribe(listener: BooksListener): Promise<() => void>;
}

export const tauriBooksClient: BooksClient = {
	presets,
	defaultSavePath,
	newBook,
	openBook,
	saveBook,
	closeBook,
	list: listBooks,
	snapshot,
	apply: applyIntent,
	undo,
	redo,
	async subscribe(listener) {
		const stops = await Promise.all([
			onPatch((event) => listener.onPatch(event)),
			onBooksChanged((event) => listener.onChanged(event)),
		]);
		return () => stops.forEach((stop) => stop());
	},
};
