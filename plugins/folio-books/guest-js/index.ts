// Exposes typed guest-side wrappers for the folio-books plugin
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type { BookSnapshot } from '@liminal-hq/folio-protocol/generated/BookSnapshot';
import type { BookSummary } from '@liminal-hq/folio-protocol/generated/BookSummary';
import type { BooksChanged } from '@liminal-hq/folio-protocol/generated/BooksChanged';
import type { BookSpelling } from '@liminal-hq/folio-protocol/generated/BookSpelling';
import type { DictionaryOffer } from '@liminal-hq/folio-protocol/generated/DictionaryOffer';
import type { DictionaryProgress } from '@liminal-hq/folio-protocol/generated/DictionaryProgress';
import type { DictionaryStatus } from '@liminal-hq/folio-protocol/generated/DictionaryStatus';
import type { ExportColour } from '@liminal-hq/folio-protocol/generated/ExportColour';
import type { ExportProgress } from '@liminal-hq/folio-protocol/generated/ExportProgress';
import type { CoverSheetPreview } from '@liminal-hq/folio-protocol/generated/CoverSheetPreview';
import type { ImportReport } from '@liminal-hq/folio-protocol/generated/ImportReport';
import type { ExportReport } from '@liminal-hq/folio-protocol/generated/ExportReport';
import type { ExportTarget } from '@liminal-hq/folio-protocol/generated/ExportTarget';
import type { FontUse } from '@liminal-hq/folio-protocol/generated/FontUse';
import type { FontChoice } from '@liminal-hq/folio-protocol/generated/FontChoice';
import type { KeepFontsReport } from '@liminal-hq/folio-protocol/generated/KeepFontsReport';
import type { ToolFix } from '@liminal-hq/folio-protocol/generated/ToolFix';
import type { ToolFixed } from '@liminal-hq/folio-protocol/generated/ToolFixed';
import type { ToolReport } from '@liminal-hq/folio-protocol/generated/ToolReport';
import type { ToolRequest } from '@liminal-hq/folio-protocol/generated/ToolRequest';
import type { TemplateGallery } from '@liminal-hq/folio-protocol/generated/TemplateGallery';
import type { CoverCanvas } from '@liminal-hq/folio-protocol/generated/CoverCanvas';
import type { StoryChapter } from '@liminal-hq/folio-protocol/generated/StoryChapter';
import type { ProofSpread } from '@liminal-hq/folio-protocol/generated/ProofSpread';
import type { ReaderPreview } from '@liminal-hq/folio-protocol/generated/ReaderPreview';
import type { PreflightItem } from '@liminal-hq/folio-protocol/generated/PreflightItem';
import type { Misspelling } from '@liminal-hq/folio-protocol/generated/Misspelling';
import type { SpellCheck } from '@liminal-hq/folio-protocol/generated/SpellCheck';
import type { Intent } from '@liminal-hq/folio-protocol/generated/Intent';
import type { PaintPresets } from '@liminal-hq/folio-protocol/generated/PaintPresets';
import type { KindInfo } from '@liminal-hq/folio-protocol/generated/KindInfo';
import type { NewBookSpec } from '@liminal-hq/folio-protocol/generated/NewBookSpec';
import type { Patch } from '@liminal-hq/folio-protocol/generated/Patch';
import type { Recoverable } from '@liminal-hq/folio-protocol/generated/Recoverable';
import type { PatchEvent } from '@liminal-hq/folio-protocol/generated/PatchEvent';
import type { PluginStatus } from '@liminal-hq/folio-protocol/generated/PluginStatus';
import type { SpreadLayoutResult } from '@liminal-hq/folio-protocol/generated/SpreadLayoutResult';

const PREFIX = 'plugin:folio-books|';

/** Sent to every window after a change to an open book; the payload is a `PatchEvent`. */
export const PATCH_EVENT = 'folio-books://patch';

/** Sent to every window when the main window shows another book; the payload is its id or `null`. */
export const ACTIVE_EVENT = 'folio-books://active';

/** Sent to every window when books open, close, save or become changed or clean; a `BooksChanged`. */
export const CHANGED_EVENT = 'folio-books://changed';

/** Sent to every window while a dictionary downloads. */
export const DICTIONARY_PROGRESS_EVENT = 'folio-books://dictionary-progress';

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

/** The colours and gradients the pickers offer. */
export function paintPresets(): Promise<PaintPresets> {
	return cmd<PaintPresets>('paint_presets');
}

/** Where Save as… should start for a book with this title. */
export function defaultSavePath(title: string): Promise<string> {
	return cmd<string>('default_save_path', { title });
}

/** Starts a new book. Nothing is written until it is saved. */
export function newBook(spec: Partial<NewBookSpec>): Promise<BookSnapshot> {
	return cmd<BookSnapshot>('new_book', { spec });
}

/** The cover of the book `spec` would make, as a `data:image/png` URI. */
export function previewNewBook(spec: NewBookSpec): Promise<string> {
	return cmd<string>('preview_new_book', { spec });
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

/** Writes a recovery copy of each book with changes the last copy does not have; how many were written. */
export function autosaveBooks(): Promise<number> {
	return cmd<number>('autosave_books');
}

/** The books a crash left changes in, that are not open, newest first. */
export function recoverableBooks(): Promise<Recoverable[]> {
	return cmd<Recoverable[]>('recoverable_books');
}

/** Opens a recovery copy as the book it was, still unsaved. */
export function restoreBook(id: string): Promise<BookSnapshot> {
	return cmd<BookSnapshot>('restore_book', { id });
}

/** Gives a recovery copy up. */
export function discardRecovered(id: string): Promise<void> {
	return cmd<void>('discard_recovered', { id });
}

/** How many steps can be undone now: a mark to give `rewindHistory`. */
export function historyMark(id: string): Promise<number> {
	return cmd<number>('history_mark', { id });
}

/** Undoes steps until only `mark` can be undone. Every window hears each patch as an event. */
export function rewindHistory(id: string, mark: number): Promise<void> {
	return cmd<void>('rewind_history', { id, mark });
}

/** Redoes the last undone step; `null` when there is nothing to redo. */
export function redo(id: string): Promise<Patch | null> {
	return cmd<Patch | null>('redo', { id });
}

/** A master page's text laid out on a page of its own, with `{page}` and other tokens as written. */
export function layoutMaster(id: string, master: string): Promise<SpreadLayoutResult> {
	return cmd<SpreadLayoutResult>('layout_master', { id, master });
}

/**
 * Lays out one spread of an open book: where every line of text goes, and the preflight checks on
 * it. Given `spelling` (the words added to the dictionary for every book), misspelt words are marked
 * in the lines; without it nothing is checked.
 */
export function layoutSpread(
	id: string,
	spread: string,
	spelling?: string[],
): Promise<SpreadLayoutResult> {
	return cmd<SpreadLayoutResult>('layout_spread', { id, spread, spelling });
}

/** Hears every change to every open book. Read `snapshot` first and apply patches with a higher revision. */
export function onPatch(listener: (event: PatchEvent) => void): Promise<UnlistenFn> {
	return listen<PatchEvent>(PATCH_EVENT, (e) => listener(e.payload));
}

/** Hears when books open, close, save or become changed or clean. */
export function onBooksChanged(listener: (event: BooksChanged) => void): Promise<UnlistenFn> {
	return listen<BooksChanged>(CHANGED_EVENT, (e) => listener(e.payload));
}

export const EXPORT_PROGRESS_EVENT = 'folio-books://export-progress';

/** Hears which file an export has reached. */
export function onExportProgress(listener: (event: ExportProgress) => void): Promise<UnlistenFn> {
	return listen<ExportProgress>(EXPORT_PROGRESS_EVENT, (e) => listener(e.payload));
}

/** Stops the export in progress after the file it is making; what was made is kept. */
export function cancelExport(): Promise<void> {
	return cmd<void>('cancel_export');
}

/** Hears how far a dictionary download has got. */
export function onDictionaryProgress(
	listener: (event: DictionaryProgress) => void,
): Promise<UnlistenFn> {
	return listen<DictionaryProgress>(DICTIONARY_PROGRESS_EVENT, (e) => listener(e.payload));
}

export type {
	BookSnapshot,
	BookSpelling,
	DictionaryOffer,
	DictionaryStatus,
	ExportReport,
	ExportTarget,
	FontChoice,
	FontUse,
	KeepFontsReport,
	Misspelling,
	PreflightItem,
	ReaderPreview,
	SpellCheck,
	BookSummary,
	BooksChanged,
	Intent,
	KindInfo,
	PaintPresets,
	NewBookSpec,
	Patch,
	PatchEvent,
	PluginStatus,
	Recoverable,
};

/** The book as an e-reader would show it, from the same chapters and CSS the EPUB ships. */
export function readerPreview(id: string): Promise<ReaderPreview> {
	return cmd<ReaderPreview>('reader_preview', { id });
}

/** A spread as a press would print it, through the CMYK ICC profile at `profile`. */
export function proofSpread(id: string, spread: string, profile: string): Promise<ProofSpread> {
	return cmd<ProofSpread>('proof_spread', { id, spread, profile });
}

/** The paperback cover sheet as the exporter would make it, as a PNG data URI; `null` with no cover. */
export function previewCoverSheet(id: string): Promise<CoverSheetPreview | null> {
	return cmd<CoverSheetPreview | null>('preview_cover_sheet', { id });
}

/** Makes the files for `targets` and writes them into the book's `exports` folder. */
export function exportBook(
	id: string,
	targets: ExportTarget[],
	colour?: ExportColour,
): Promise<ExportReport> {
	return cmd<ExportReport>('export_book', { id, targets, colour });
}

/** Brings the words of a Markdown or plain text file in as the book's story, in one undo step. */
export function importText(
	id: string,
	path: string,
	mapping?: Record<string, string>,
): Promise<ImportReport> {
	return cmd<ImportReport>('import_text', { id, path, mapping: mapping ?? null });
}

/** Asks a Book tool about a book; the answer is worked out in Rust. */
export function bookTool(id: string, request: ToolRequest): Promise<ToolReport> {
	return cmd<ToolReport>('book_tool', { id, request });
}

/** Makes a Book tool's change to a book as one undo step. */
export function bookToolFix(id: string, fix: ToolFix): Promise<ToolFixed> {
	return cmd<ToolFixed>('book_tool_fix', { id, fix });
}

/** Says which book the main window is showing, so other windows can follow it. */
export function setActiveBook(id: string | null): Promise<void> {
	return cmd<void>('set_active_book', { id });
}

/** The book the main window is showing; `null` when none is. */
export function activeBook(): Promise<string | null> {
	return cmd<string | null>('active_book');
}

/** Called when the main window shows another book. */
export function onActiveBook(listener: (id: string | null) => void): Promise<UnlistenFn> {
	return listen<string | null>(ACTIVE_EVENT, (e) => listener(e.payload));
}

/** The spread templates a book can add, each drawn at the book's size. */
export function templateGallery(id: string): Promise<TemplateGallery> {
	return cmd<TemplateGallery>('template_gallery', { id });
}

/** Adds a spread template after a spread, as one undo step. */
export function applyTemplate(id: string, template: string, after: string): Promise<Patch> {
	return cmd<Patch>('apply_template', { id, template, after });
}

/** The cover sheet as a one-spread book with its layout, for the canvas; `null` with no cover. */
export function coverSheetCanvas(id: string): Promise<CoverCanvas | null> {
	return cmd<CoverCanvas | null>('cover_sheet_canvas', { id });
}

/** The chapters of the book's stories and the spread each begins on. */
export function storyChapters(id: string): Promise<StoryChapter[]> {
	return cmd<StoryChapter[]>('story_chapters', { id });
}

/** Shows a file in the system's file manager. */
export function revealPath(path: string): Promise<void> {
	return cmd<void>('reveal_path', { path });
}

/** What preflight finds in the whole book, the issues that wait on the person first. */
export function preflightBook(id: string): Promise<PreflightItem[]> {
	return cmd<PreflightItem[]>('preflight_book', { id });
}

/** Every family a book can draw in, and where each comes from (bundled, kept in the book or installed). */
export function listFonts(id: string): Promise<FontChoice[]> {
	return cmd<FontChoice[]>('list_fonts', { id });
}

/** The typefaces the book uses, and where each comes from. */
export function fontsInUse(id: string): Promise<FontUse[]> {
	return cmd<FontUse[]>('fonts_in_use', { id });
}

/** Takes fonts out of the book, so it is laid out from the computer's again. */
export function unkeepFonts(id: string, families: string[]): Promise<void> {
	return cmd<void>('unkeep_fonts', { id, families });
}

/** Keeps installed fonts inside the book; says which were kept and which could not be, and why. */
export function keepFonts(id: string, families: string[]): Promise<KeepFontsReport> {
	return cmd<KeepFontsReport>('keep_fonts', { id, families });
}

/** Reads a picture file into the book; resolves to its path inside the book (`assets/images/…`). */
export function importImage(id: string, path: string): Promise<string> {
	return cmd<string>('import_image', { id, path });
}

/** One of the book's pictures as raw bytes. */
export function assetData(id: string, asset: string): Promise<ArrayBuffer> {
	return cmd<ArrayBuffer>('asset_data', { id, asset });
}

/** The words of `text` the book's dictionary does not know (and neither the book nor `personal` ignores), by UTF-16 range, and whether there is a dictionary. */
export function spellCheck(id: string, text: string, personal: string[] = []): Promise<SpellCheck> {
	return cmd<SpellCheck>('spell_check', { id, text, personal });
}

/** Corrections for a word, best first. */
export function spellSuggest(id: string, word: string): Promise<string[]> {
	return cmd<string[]>('spell_suggest', { id, word });
}

/** Every dictionary there is, and whether it is in place. */
export function dictionaries(): Promise<DictionaryStatus[]> {
	return cmd<DictionaryStatus[]>('dictionaries');
}

/** Downloads a dictionary, checked against what is pinned for it; resolves to the list again. */
export function downloadDictionary(id: string): Promise<DictionaryStatus[]> {
	return cmd<DictionaryStatus[]>('download_dictionary', { id });
}

/** Takes a downloaded dictionary away; resolves to the list again. */
export function removeDictionary(id: string): Promise<DictionaryStatus[]> {
	return cmd<DictionaryStatus[]>('remove_dictionary', { id });
}

/** Every misspelt word in the book, and where it is. */
export function spellCheckBook(id: string, personal: string[] = []): Promise<BookSpelling> {
	return cmd<BookSpelling>('spell_check_book', { id, personal });
}
