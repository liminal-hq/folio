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
    /** Hears how far a dictionary download has got. */
    function onDictionaryProgress(listener) {
        return event.listen(DICTIONARY_PROGRESS_EVENT, (e) => listener(e.payload));
    }
    /** What preflight finds in the whole book, the issues that wait on the person first. */
    function preflightBook(id) {
        return cmd('preflight_book', { id });
    }
    /** Every family a book can draw in, and where each comes from (bundled, kept in the book or installed). */
    function listFonts(id) {
        return cmd('list_fonts', { id });
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

    exports.CHANGED_EVENT = CHANGED_EVENT;
    exports.DICTIONARY_PROGRESS_EVENT = DICTIONARY_PROGRESS_EVENT;
    exports.PATCH_EVENT = PATCH_EVENT;
    exports.applyIntent = applyIntent;
    exports.assetData = assetData;
    exports.autosaveBooks = autosaveBooks;
    exports.closeBook = closeBook;
    exports.defaultSavePath = defaultSavePath;
    exports.dictionaries = dictionaries;
    exports.discardRecovered = discardRecovered;
    exports.downloadDictionary = downloadDictionary;
    exports.getStatus = getStatus;
    exports.historyMark = historyMark;
    exports.importImage = importImage;
    exports.keepFonts = keepFonts;
    exports.layoutMaster = layoutMaster;
    exports.layoutSpread = layoutSpread;
    exports.listBooks = listBooks;
    exports.listFonts = listFonts;
    exports.newBook = newBook;
    exports.onBooksChanged = onBooksChanged;
    exports.onDictionaryProgress = onDictionaryProgress;
    exports.onPatch = onPatch;
    exports.openBook = openBook;
    exports.paintPresets = paintPresets;
    exports.preflightBook = preflightBook;
    exports.presets = presets;
    exports.recoverableBooks = recoverableBooks;
    exports.redo = redo;
    exports.removeDictionary = removeDictionary;
    exports.restoreBook = restoreBook;
    exports.rewindHistory = rewindHistory;
    exports.saveBook = saveBook;
    exports.snapshot = snapshot;
    exports.spellCheck = spellCheck;
    exports.spellCheckBook = spellCheckBook;
    exports.spellSuggest = spellSuggest;
    exports.undo = undo;

    return exports;

})({}, __TAURI__.core, __TAURI__.event);
Object.defineProperty(window.__TAURI__, 'folioBooks', { value: __TAURI_PLUGIN_FOLIO_BOOKS__ }) }
