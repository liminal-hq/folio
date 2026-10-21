if ('__TAURI__' in window) {
var __TAURI_PLUGIN_FOLIO_BOOKS__ = (function (exports, core, event) {
    'use strict';

    // Exposes typed guest-side wrappers for the folio-books plugin
    //
    // (c) Copyright 2026 Liminal HQ, Scott Morris
    // SPDX-License-Identifier: Apache-2.0 OR MIT
    const PREFIX = 'plugin:folio-books|';
    /** Sent to every window after a change to an open book; the payload is a `PatchEvent`. */
    const PATCH_EVENT = 'folio-books://patch';
    /** Sent to every window when the main window shows another book; the payload is its id or `null`. */
    const ACTIVE_EVENT = 'folio-books://active';
    /** Sent to every window when books open, close, save or become changed or clean; a `BooksChanged`. */
    const CHANGED_EVENT = 'folio-books://changed';
    /** Sent to every window while a dictionary downloads. */
    const DICTIONARY_PROGRESS_EVENT = 'folio-books://dictionary-progress';
    function cmd(name, args) {
        return core.invoke(`${PREFIX}${name}`, args);
    }
    /** Reports whether the books plugin works. */
    function getStatus() {
        return cmd('get_status');
    }
    /** The six kinds of book and the sizes each comes in. */
    function presets() {
        return cmd('presets');
    }
    /** The colours and gradients the pickers offer. */
    function paintPresets() {
        return cmd('paint_presets');
    }
    /** Where Save as… should start for a book with this title. */
    function defaultSavePath(title) {
        return cmd('default_save_path', { title });
    }
    /** Starts a new book. Nothing is written until it is saved. */
    function newBook(spec) {
        return cmd('new_book', { spec });
    }
    /** The cover of the book `spec` would make, as a `data:image/png` URI. */
    function previewNewBook(spec) {
        return cmd('preview_new_book', { spec });
    }
    /** Opens a `.folio` file, or returns the book if it is already open. */
    function openBook(path) {
        return cmd('open_book', { path });
    }
    /** Saves a book to `path`, or to where it already is when `path` is left out. */
    function saveBook(id, path) {
        return cmd('save_book', { id, path: path ?? null });
    }
    /** Closes a book without asking about unsaved changes. */
    function closeBook(id) {
        return cmd('close_book', { id });
    }
    /** The open books, oldest first. */
    function listBooks() {
        return cmd('list_books');
    }
    /** An open book in full, with the revision it is at. */
    function snapshot(id) {
        return cmd('snapshot', { id });
    }
    /** Applies an intent. Every window also hears the patch as an event. */
    function applyIntent(id, intent) {
        return cmd('apply_intent', { id, intent });
    }
    /** Undoes the last step; `null` when there is nothing to undo. */
    function undo(id) {
        return cmd('undo', { id });
    }
    /** Writes a recovery copy of each book with changes the last copy does not have; how many were written. */
    function autosaveBooks() {
        return cmd('autosave_books');
    }
    /** The books a crash left changes in, that are not open, newest first. */
    function recoverableBooks() {
        return cmd('recoverable_books');
    }
    /** Opens a recovery copy as the book it was, still unsaved. */
    function restoreBook(id) {
        return cmd('restore_book', { id });
    }
    /** Gives a recovery copy up. */
    function discardRecovered(id) {
        return cmd('discard_recovered', { id });
    }
    /** How many steps can be undone now: a mark to give `rewindHistory`. */
    function historyMark(id) {
        return cmd('history_mark', { id });
    }
    /** Undoes steps until only `mark` can be undone. Every window hears each patch as an event. */
    function rewindHistory(id, mark) {
        return cmd('rewind_history', { id, mark });
    }
    /** Redoes the last undone step; `null` when there is nothing to redo. */
    function redo(id) {
        return cmd('redo', { id });
    }
    /** A master page's text laid out on a page of its own, with `{page}` and other tokens as written. */
    function layoutMaster(id, master) {
        return cmd('layout_master', { id, master });
    }
    /**
     * Lays out one spread of an open book: where every line of text goes, and the preflight checks on
     * it. Given `spelling` (the words added to the dictionary for every book), misspelt words are marked
     * in the lines; without it nothing is checked.
     */
    function layoutSpread(id, spread, spelling) {
        return cmd('layout_spread', { id, spread, spelling });
    }
    /** Hears every change to every open book. Read `snapshot` first and apply patches with a higher revision. */
    function onPatch(listener) {
        return event.listen(PATCH_EVENT, (e) => listener(e.payload));
    }
    /** Hears when books open, close, save or become changed or clean. */
    function onBooksChanged(listener) {
        return event.listen(CHANGED_EVENT, (e) => listener(e.payload));
    }
    const EXPORT_PROGRESS_EVENT = 'folio-books://export-progress';
    /** Hears which file an export has reached. */
    function onExportProgress(listener) {
        return event.listen(EXPORT_PROGRESS_EVENT, (e) => listener(e.payload));
    }
    /** Stops the export in progress after the file it is making; what was made is kept. */
    function cancelExport() {
        return cmd('cancel_export');
    }
    /** Hears how far a dictionary download has got. */
    function onDictionaryProgress(listener) {
        return event.listen(DICTIONARY_PROGRESS_EVENT, (e) => listener(e.payload));
    }
    /** The book as an e-reader would show it, from the same chapters and CSS the EPUB ships. */
    function readerPreview(id) {
        return cmd('reader_preview', { id });
    }
    /** A spread as a press would print it, through the CMYK ICC profile at `profile`. */
    function proofSpread(id, spread, profile) {
        return cmd('proof_spread', { id, spread, profile });
    }
    /** The paperback cover sheet as the exporter would make it, as a PNG data URI; `null` with no cover. */
    function previewCoverSheet(id) {
        return cmd('preview_cover_sheet', { id });
    }
    /** Makes the files for `targets` and writes them into the book's `exports` folder. */
    function exportBook(id, targets, colour) {
        return cmd('export_book', { id, targets, colour });
    }
    /** Brings the words of a Markdown or plain text file in as the book's story, in one undo step. */
    function importText(id, path, mapping) {
        return cmd('import_text', { id, path, mapping: mapping ?? null });
    }
    /** Asks a Book tool about a book; the answer is worked out in Rust. */
    function bookTool(id, request) {
        return cmd('book_tool', { id, request });
    }
    /** Makes a Book tool's change to a book as one undo step. */
    function bookToolFix(id, fix) {
        return cmd('book_tool_fix', { id, fix });
    }
    /** Says which book the main window is showing, so other windows can follow it. */
    function setActiveBook(id) {
        return cmd('set_active_book', { id });
    }
    /** The book the main window is showing; `null` when none is. */
    function activeBook() {
        return cmd('active_book');
    }
    /** Called when the main window shows another book. */
    function onActiveBook(listener) {
        return event.listen(ACTIVE_EVENT, (e) => listener(e.payload));
    }
    /** The spread templates a book can add, each drawn at the book's size. */
    function templateGallery(id) {
        return cmd('template_gallery', { id });
    }
    /** Adds a spread template after a spread, as one undo step. */
    function applyTemplate(id, template, after) {
        return cmd('apply_template', { id, template, after });
    }
    /** The cover sheet as a one-spread book with its layout, for the canvas; `null` with no cover. */
    function coverSheetCanvas(id) {
        return cmd('cover_sheet_canvas', { id });
    }
    /** The chapters of the book's stories and the spread each begins on. */
    function storyChapters(id) {
        return cmd('story_chapters', { id });
    }
    /** Shows a file in the system's file manager. */
    function revealPath(path) {
        return cmd('reveal_path', { path });
    }
    /** What preflight finds in the whole book, the issues that wait on the person first. */
    function preflightBook(id) {
        return cmd('preflight_book', { id });
    }
    /** Every family a book can draw in, and where each comes from (bundled, kept in the book or installed). */
    function listFonts(id) {
        return cmd('list_fonts', { id });
    }
    /** The typefaces the book uses, and where each comes from. */
    function fontsInUse(id) {
        return cmd('fonts_in_use', { id });
    }
    /** Takes fonts out of the book, so it is laid out from the computer's again. */
    function unkeepFonts(id, families) {
        return cmd('unkeep_fonts', { id, families });
    }
    /** Keeps installed fonts inside the book; says which were kept and which could not be, and why. */
    function keepFonts(id, families) {
        return cmd('keep_fonts', { id, families });
    }
    /** Reads a picture file into the book; resolves to its path inside the book (`assets/images/…`). */
    function importImage(id, path) {
        return cmd('import_image', { id, path });
    }
    /** One of the book's pictures as raw bytes. */
    function assetData(id, asset) {
        return cmd('asset_data', { id, asset });
    }
    /** The words of `text` the book's dictionary does not know (and neither the book nor `personal` ignores), by UTF-16 range, and whether there is a dictionary. */
    function spellCheck(id, text, personal = []) {
        return cmd('spell_check', { id, text, personal });
    }
    /** Corrections for a word, best first. */
    function spellSuggest(id, word) {
        return cmd('spell_suggest', { id, word });
    }
    /** Every dictionary there is, and whether it is in place. */
    function dictionaries() {
        return cmd('dictionaries');
    }
    /** Downloads a dictionary, checked against what is pinned for it; resolves to the list again. */
    function downloadDictionary(id) {
        return cmd('download_dictionary', { id });
    }
    /** Takes a downloaded dictionary away; resolves to the list again. */
    function removeDictionary(id) {
        return cmd('remove_dictionary', { id });
    }
    /** Every misspelt word in the book, and where it is. */
    function spellCheckBook(id, personal = []) {
        return cmd('spell_check_book', { id, personal });
    }

    exports.ACTIVE_EVENT = ACTIVE_EVENT;
    exports.CHANGED_EVENT = CHANGED_EVENT;
    exports.DICTIONARY_PROGRESS_EVENT = DICTIONARY_PROGRESS_EVENT;
    exports.EXPORT_PROGRESS_EVENT = EXPORT_PROGRESS_EVENT;
    exports.PATCH_EVENT = PATCH_EVENT;
    exports.activeBook = activeBook;
    exports.applyIntent = applyIntent;
    exports.applyTemplate = applyTemplate;
    exports.assetData = assetData;
    exports.autosaveBooks = autosaveBooks;
    exports.bookTool = bookTool;
    exports.bookToolFix = bookToolFix;
    exports.cancelExport = cancelExport;
    exports.closeBook = closeBook;
    exports.coverSheetCanvas = coverSheetCanvas;
    exports.defaultSavePath = defaultSavePath;
    exports.dictionaries = dictionaries;
    exports.discardRecovered = discardRecovered;
    exports.downloadDictionary = downloadDictionary;
    exports.exportBook = exportBook;
    exports.fontsInUse = fontsInUse;
    exports.getStatus = getStatus;
    exports.historyMark = historyMark;
    exports.importImage = importImage;
    exports.importText = importText;
    exports.keepFonts = keepFonts;
    exports.layoutMaster = layoutMaster;
    exports.layoutSpread = layoutSpread;
    exports.listBooks = listBooks;
    exports.listFonts = listFonts;
    exports.newBook = newBook;
    exports.onActiveBook = onActiveBook;
    exports.onBooksChanged = onBooksChanged;
    exports.onDictionaryProgress = onDictionaryProgress;
    exports.onExportProgress = onExportProgress;
    exports.onPatch = onPatch;
    exports.openBook = openBook;
    exports.paintPresets = paintPresets;
    exports.preflightBook = preflightBook;
    exports.presets = presets;
    exports.previewCoverSheet = previewCoverSheet;
    exports.previewNewBook = previewNewBook;
    exports.proofSpread = proofSpread;
    exports.readerPreview = readerPreview;
    exports.recoverableBooks = recoverableBooks;
    exports.redo = redo;
    exports.removeDictionary = removeDictionary;
    exports.restoreBook = restoreBook;
    exports.revealPath = revealPath;
    exports.rewindHistory = rewindHistory;
    exports.saveBook = saveBook;
    exports.setActiveBook = setActiveBook;
    exports.snapshot = snapshot;
    exports.spellCheck = spellCheck;
    exports.spellCheckBook = spellCheckBook;
    exports.spellSuggest = spellSuggest;
    exports.storyChapters = storyChapters;
    exports.templateGallery = templateGallery;
    exports.undo = undo;
    exports.unkeepFonts = unkeepFonts;

    return exports;

})({}, __TAURI__.core, __TAURI__.event);
Object.defineProperty(window.__TAURI__, 'folioBooks', { value: __TAURI_PLUGIN_FOLIO_BOOKS__ }) }
