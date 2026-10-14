// Exposes typed guest-side wrappers for the folio-books plugin
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type { BookSnapshot } from '@liminal-hq/folio-protocol/generated/BookSnapshot';
import type { BookSummary } from '@liminal-hq/folio-protocol/generated/BookSummary';
import type { BooksChanged } from '@liminal-hq/folio-protocol/generated/BooksChanged';
import type { Intent } from '@liminal-hq/folio-protocol/generated/Intent';
import type { KindInfo } from '@liminal-hq/folio-protocol/generated/KindInfo';
import type { NewBookSpec } from '@liminal-hq/folio-protocol/generated/NewBookSpec';
import type { Patch } from '@liminal-hq/folio-protocol/generated/Patch';
import type { PatchEvent } from '@liminal-hq/folio-protocol/generated/PatchEvent';
import type { PluginStatus } from '@liminal-hq/folio-protocol/generated/PluginStatus';
import type { SpreadLayoutResult } from '@liminal-hq/folio-protocol/generated/SpreadLayoutResult';

const PREFIX = 'plugin:folio-books|';

/** Sent to every window after a change to an open book; the payload is a `PatchEvent`. */
export const PATCH_EVENT = 'folio-books://patch';

/** Sent to every window when books open, close, save or become changed or clean; a `BooksChanged`. */
export const CHANGED_EVENT = 'folio-books://changed';

/**
 * What a rejected command carries. `kind` is a stable name: `noSuchBook`, `needsPath`, `preset`
 * (a choice that cannot make a book), `notFolio` (not a Folio book), `newerVersion` (saved by a
 * newer Folio), `package` (the file could not be read or written) and `session` (an intent that
 * cannot be applied).
 */
export interface BooksCommandError {
	kind:
		| 'noSuchBook'
		| 'needsPath'
		| 'preset'
		| 'notFolio'
		| 'newerVersion'
		| 'package'
		| 'session'
		| 'layout'
		| 'noSuchSpread'
		| 'image';
	message: string;
}

function cmd<T>(name: string, args?: Record<string, unknown>): Promise<T> {
	return invoke<T>(`${PREFIX}${name}`, args);
}

/** Reports whether the books plugin works. */
export function getStatus(): Promise<PluginStatus> {
	return cmd<PluginStatus>('get_status');
}

/** The six kinds of book and the sizes each comes in. */
export function presets(): Promise<KindInfo[]> {
	return cmd<KindInfo[]>('presets');
}

/** Where Save as… should start for a book with this title. */
export function defaultSavePath(title: string): Promise<string> {
	return cmd<string>('default_save_path', { title });
}

/** Starts a new book. Nothing is written until it is saved. */
export function newBook(spec: Partial<NewBookSpec>): Promise<BookSnapshot> {
	return cmd<BookSnapshot>('new_book', { spec });
}

/** Opens a `.folio` file, or returns the book if it is already open. */
export function openBook(path: string): Promise<BookSnapshot> {
	return cmd<BookSnapshot>('open_book', { path });
}

/** Saves a book to `path`, or to where it already is when `path` is left out. */
export function saveBook(id: string, path?: string): Promise<BookSummary> {
	return cmd<BookSummary>('save_book', { id, path: path ?? null });
}

/** Closes a book without asking about unsaved changes. */
export function closeBook(id: string): Promise<void> {
	return cmd<void>('close_book', { id });
}

/** The open books, oldest first. */
export function listBooks(): Promise<BookSummary[]> {
	return cmd<BookSummary[]>('list_books');
}

/** An open book in full, with the revision it is at. */
export function snapshot(id: string): Promise<BookSnapshot> {
	return cmd<BookSnapshot>('snapshot', { id });
}

/** Applies an intent. Every window also hears the patch as an event. */
export function applyIntent(id: string, intent: Intent): Promise<Patch> {
	return cmd<Patch>('apply_intent', { id, intent });
}

/** Undoes the last step; `null` when there is nothing to undo. */
export function undo(id: string): Promise<Patch | null> {
	return cmd<Patch | null>('undo', { id });
}

/** Redoes the last undone step; `null` when there is nothing to redo. */
export function redo(id: string): Promise<Patch | null> {
	return cmd<Patch | null>('redo', { id });
}

/** Lays out one spread of an open book: where every line of text goes, and the preflight checks on it. */
export function layoutSpread(id: string, spread: string): Promise<SpreadLayoutResult> {
	return cmd<SpreadLayoutResult>('layout_spread', { id, spread });
}

/** Hears every change to every open book. Read `snapshot` first and apply patches with a higher revision. */
export function onPatch(listener: (event: PatchEvent) => void): Promise<UnlistenFn> {
	return listen<PatchEvent>(PATCH_EVENT, (e) => listener(e.payload));
}

/** Hears when books open, close, save or become changed or clean. */
export function onBooksChanged(listener: (event: BooksChanged) => void): Promise<UnlistenFn> {
	return listen<BooksChanged>(CHANGED_EVENT, (e) => listener(e.payload));
}

export type {
	BookSnapshot,
	BookSummary,
	BooksChanged,
	Intent,
	KindInfo,
	NewBookSpec,
	Patch,
	PatchEvent,
	PluginStatus,
};

/** Reads a picture file into the book; resolves to its path inside the book (`assets/images/…`). */
export function importImage(id: string, path: string): Promise<string> {
	return cmd<string>('import_image', { id, path });
}

/** One of the book's pictures as raw bytes. */
export function assetData(id: string, asset: string): Promise<ArrayBuffer> {
	return cmd<ArrayBuffer>('asset_data', { id, asset });
}
