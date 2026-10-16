// Reads and changes the open books through the books plugin's guest-js API
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { BookSpelling } from '@liminal-hq/folio-protocol/generated/BookSpelling';
import type { Recoverable } from '@liminal-hq/folio-protocol/generated/Recoverable';
import type { DictionaryProgress } from '@liminal-hq/folio-protocol/generated/DictionaryProgress';
import type { DictionaryStatus } from '@liminal-hq/folio-protocol/generated/DictionaryStatus';
import type { SpellCheck } from '@liminal-hq/folio-protocol/generated/SpellCheck';
import type { SpreadLayoutResult } from '@liminal-hq/folio-protocol/generated/SpreadLayoutResult';
import {
	assetData,
	applyIntent,
	closeBook,
	defaultSavePath,
	dictionaries,
	downloadDictionary,
	removeDictionary,
	onDictionaryProgress,
	importImage,
	layoutMaster,
	layoutSpread,
	listBooks,
	newBook,
	onBooksChanged,
	onPatch,
	openBook,
	paintPresets,
	presets,
	redo,
	saveBook,
	spellCheck,
	spellCheckBook,
	spellSuggest,
	snapshot,
	undo,
	historyMark,
	rewindHistory,
	autosaveBooks,
	recoverableBooks,
	restoreBook,
	discardRecovered,
	type BookSnapshot,
	type BookSummary,
	type BooksChanged,
	type Intent,
	type KindInfo,
	type NewBookSpec,
	type PaintPresets,
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
	/** The colours and gradients the pickers offer. */
	paintPresets(): Promise<PaintPresets>;
	defaultSavePath(title: string): Promise<string>;
	newBook(spec: Partial<NewBookSpec>): Promise<BookSnapshot>;
	openBook(path: string): Promise<BookSnapshot>;
	saveBook(id: string, path?: string): Promise<BookSummary>;
	closeBook(id: string): Promise<void>;
	list(): Promise<BookSummary[]>;
	snapshot(id: string): Promise<BookSnapshot>;
	apply(id: string, intent: Intent): Promise<Patch>;
	undo(id: string): Promise<Patch | null>;
	/** Writes a recovery copy of each book with changes the last copy lacks; how many were written. */
	autosave(): Promise<number>;
	/** The books a crash left changes in, that are not open, newest first. */
	recoverable(): Promise<Recoverable[]>;
	/** Opens a recovery copy as the book it was, still unsaved. */
	restore(id: string): Promise<BookSnapshot>;
	/** Gives a recovery copy up. */
	discardRecovered(id: string): Promise<void>;
	/** How many steps can be undone now: a mark to rewind to. */
	historyMark(id: string): Promise<number>;
	/** Undoes steps until only `mark` can be undone; each is announced as an event. */
	rewindHistory(id: string, mark: number): Promise<void>;
	redo(id: string): Promise<Patch | null>;
	/** The spread's text shaped and broken into lines, from the book at the revision it carries. */
	layoutSpread(id: string, spread: string, spelling?: string[]): Promise<SpreadLayoutResult>;
	/** A master page's text laid out on a page of its own, with `{page}` and other tokens as written. */
	layoutMaster(id: string, master: string): Promise<SpreadLayoutResult>;
	/** Reads a picture file into the book; resolves to its path inside the book. */
	importImage(id: string, path: string): Promise<string>;
	/** One of the book's pictures as raw bytes. */
	assetData(id: string, asset: string): Promise<ArrayBuffer>;
	/** The words of `text` the book's dictionary does not know, by UTF-16 range, and whether it has a dictionary. */
	spellCheck(id: string, text: string, personal?: string[]): Promise<SpellCheck>;
	/** Every misspelt word in the book, and where it is. */
	spellCheckBook(id: string, personal?: string[]): Promise<BookSpelling>;
	/** Corrections for a word, best first. */
	spellSuggest(id: string, word: string): Promise<string[]>;
	/** Every dictionary there is, and whether it is in place. */
	dictionaries(): Promise<DictionaryStatus[]>;
	/** Downloads a dictionary, checked against what is pinned for it; resolves to the list again. */
	downloadDictionary(id: string): Promise<DictionaryStatus[]>;
	/** Takes a downloaded dictionary away; resolves to the list again. */
	removeDictionary(id: string): Promise<DictionaryStatus[]>;
	/** Hears how far a dictionary download has got; resolves to the function that stops listening. */
	onDictionaryProgress(listener: (progress: DictionaryProgress) => void): Promise<() => void>;
	/** Resolves to the function that stops listening. */
	subscribe(listener: BooksListener): Promise<() => void>;
}

export const tauriBooksClient: BooksClient = {
	presets,
	paintPresets,
	defaultSavePath,
	newBook,
	openBook,
	saveBook,
	closeBook,
	list: listBooks,
	snapshot,
	apply: applyIntent,
	undo,
	autosave: autosaveBooks,
	recoverable: recoverableBooks,
	restore: restoreBook,
	discardRecovered,
	historyMark,
	rewindHistory,
	redo,
	layoutSpread,
	layoutMaster,
	importImage,
	assetData,
	spellCheck,
	spellCheckBook,
	spellSuggest,
	dictionaries,
	downloadDictionary,
	removeDictionary,
	onDictionaryProgress,
	async subscribe(listener) {
		const stops = await Promise.all([
			onPatch((event) => listener.onPatch(event)),
			onBooksChanged((event) => listener.onChanged(event)),
		]);
		return () => stops.forEach((stop) => stop());
	},
};
