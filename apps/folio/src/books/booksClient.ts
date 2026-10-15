// Reads and changes the open books through the books plugin's guest-js API
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

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
	spellSuggest,
	snapshot,
	undo,
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
	/** Corrections for a word, best first. */
	spellSuggest(id: string, word: string): Promise<string[]>;
	/** Every dictionary there is, and whether it is in place. */
	dictionaries(): Promise<DictionaryStatus[]>;
	/** Downloads a dictionary, checked against what is pinned for it; resolves to the list again. */
	downloadDictionary(id: string): Promise<DictionaryStatus[]>;
	/** Takes a downloaded dictionary away; resolves to the list again. */
	removeDictionary(id: string): Promise<DictionaryStatus[]>;
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
	redo,
	layoutSpread,
	layoutMaster,
	importImage,
	assetData,
	spellCheck,
	spellSuggest,
	dictionaries,
	downloadDictionary,
	removeDictionary,
	async subscribe(listener) {
		const stops = await Promise.all([
			onPatch((event) => listener.onPatch(event)),
			onBooksChanged((event) => listener.onChanged(event)),
		]);
		return () => stops.forEach((stop) => stop());
	},
};
