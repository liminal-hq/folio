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
    /** Redoes the last undone step; `null` when there is nothing to redo. */
    function redo(id) {
        return cmd('redo', { id });
    }
    /** A master page's text laid out on a page of its own, with `{page}` and other tokens as written. */
    function layoutMaster(id, master) {
        return cmd('layout_master', { id, master });
    }
    /** Lays out one spread of an open book: where every line of text goes, and the preflight checks on it. */
    function layoutSpread(id, spread) {
        return cmd('layout_spread', { id, spread });
    }
    /** Hears every change to every open book. Read `snapshot` first and apply patches with a higher revision. */
    function onPatch(listener) {
        return event.listen(PATCH_EVENT, (e) => listener(e.payload));
    }
    /** Hears when books open, close, save or become changed or clean. */
    function onBooksChanged(listener) {
        return event.listen(CHANGED_EVENT, (e) => listener(e.payload));
    }
    /** Reads a picture file into the book; resolves to its path inside the book (`assets/images/…`). */
    function importImage(id, path) {
        return cmd('import_image', { id, path });
    }
    /** One of the book's pictures as raw bytes. */
    function assetData(id, asset) {
        return cmd('asset_data', { id, asset });
    }

    exports.CHANGED_EVENT = CHANGED_EVENT;
    exports.PATCH_EVENT = PATCH_EVENT;
    exports.applyIntent = applyIntent;
    exports.assetData = assetData;
    exports.closeBook = closeBook;
    exports.defaultSavePath = defaultSavePath;
    exports.getStatus = getStatus;
    exports.importImage = importImage;
    exports.layoutMaster = layoutMaster;
    exports.layoutSpread = layoutSpread;
    exports.listBooks = listBooks;
    exports.newBook = newBook;
    exports.onBooksChanged = onBooksChanged;
    exports.onPatch = onPatch;
    exports.openBook = openBook;
    exports.presets = presets;
    exports.redo = redo;
    exports.saveBook = saveBook;
    exports.snapshot = snapshot;
    exports.undo = undo;

    return exports;

})({}, __TAURI__.core, __TAURI__.event);
Object.defineProperty(window.__TAURI__, 'folioBooks', { value: __TAURI_PLUGIN_FOLIO_BOOKS__ }) }
