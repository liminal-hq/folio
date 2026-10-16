// The plugin's commands: new, open, save and close books, and change them with intents
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::path::PathBuf;

use folio_model::Intent;
use folio_presets::{KindInfo, NewBookSpec, PaintPresets};
use folio_protocol::PluginStatus;
use folio_session::Patch;
use tauri::{AppHandle, Manager, Runtime, State, WebviewWindow};

use crate::error::Error;
use crate::state::Books;
use crate::wire::{BookSnapshot, BookSummary, DictionaryStatus, SpellCheck, SpreadLayoutResult};

/// Reports that books work. They need nothing from the system, so this is always available.
#[tauri::command]
pub async fn get_status() -> PluginStatus {
    PluginStatus::available(vec!["books".to_owned(), "events".to_owned()])
}

/// The six kinds of book and the sizes each comes in.
#[tauri::command]
pub async fn presets<R: Runtime>(
    _window: WebviewWindow<R>,
    books: State<'_, Books<R>>,
) -> Result<Vec<KindInfo>, Error> {
    Ok(books.presets())
}

/// The colours and gradients the pickers offer.
#[tauri::command]
pub async fn paint_presets<R: Runtime>(_window: WebviewWindow<R>) -> Result<PaintPresets, Error> {
    Ok(folio_presets::paint_presets())
}

/// Where Save as… should start for a book with this title.
#[tauri::command]
pub async fn default_save_path<R: Runtime>(
    _window: WebviewWindow<R>,
    books: State<'_, Books<R>>,
    title: String,
) -> Result<String, Error> {
    Ok(books
        .default_save_path(&title)
        .to_string_lossy()
        .into_owned())
}

/// Starts a new book from the choices in `spec`. Nothing is written until it is saved.
#[tauri::command]
pub async fn new_book<R: Runtime>(
    _window: WebviewWindow<R>,
    books: State<'_, Books<R>>,
    spec: NewBookSpec,
) -> Result<BookSnapshot, Error> {
    books.new_book(&spec)
}

/// Opens a `.folio` file, or returns the book if it is already open.
#[tauri::command]
pub async fn open_book<R: Runtime>(
    _window: WebviewWindow<R>,
    books: State<'_, Books<R>>,
    path: String,
) -> Result<BookSnapshot, Error> {
    books.open(&PathBuf::from(path))
}

/// Saves a book to `path`, or to where it already is when `path` is left out.
#[tauri::command]
pub async fn save_book<R: Runtime>(
    _window: WebviewWindow<R>,
    books: State<'_, Books<R>>,
    id: String,
    path: Option<String>,
) -> Result<BookSummary, Error> {
    books.save(&id, path.map(PathBuf::from).as_deref())
}

/// Closes a book. It does not ask about unsaved changes; the window does that first.
#[tauri::command]
pub async fn close_book<R: Runtime>(
    _window: WebviewWindow<R>,
    books: State<'_, Books<R>>,
    id: String,
) -> Result<(), Error> {
    books.close(&id)
}

/// The open books, oldest first.
#[tauri::command]
pub async fn list_books<R: Runtime>(
    _window: WebviewWindow<R>,
    books: State<'_, Books<R>>,
) -> Result<Vec<BookSummary>, Error> {
    Ok(books.list())
}

/// An open book in full, with the revision it is at.
#[tauri::command]
pub async fn snapshot<R: Runtime>(
    _window: WebviewWindow<R>,
    books: State<'_, Books<R>>,
    id: String,
) -> Result<BookSnapshot, Error> {
    books.snapshot(&id)
}

/// Applies an intent and answers with the patch; every window is also sent it as an event.
#[tauri::command]
pub async fn apply_intent<R: Runtime>(
    _window: WebviewWindow<R>,
    books: State<'_, Books<R>>,
    id: String,
    intent: Intent,
) -> Result<Patch, Error> {
    books.apply(&id, intent)
}

/// Undoes the last step; `null` when there is nothing to undo.
#[tauri::command]
pub async fn undo<R: Runtime>(
    _window: WebviewWindow<R>,
    books: State<'_, Books<R>>,
    id: String,
) -> Result<Option<Patch>, Error> {
    books.undo(&id)
}

/// Redoes the last undone step; `null` when there is nothing to redo.
#[tauri::command]
pub async fn redo<R: Runtime>(
    _window: WebviewWindow<R>,
    books: State<'_, Books<R>>,
    id: String,
) -> Result<Option<Patch>, Error> {
    books.redo(&id)
}

/// Lays out one spread of a book: where every line of text goes, and the preflight checks on it. It
/// runs on a blocking thread, because the first call scans the system's fonts.
#[tauri::command]
pub async fn layout_spread<R: Runtime>(
    app: AppHandle<R>,
    _window: WebviewWindow<R>,
    id: String,
    spread: String,
) -> Result<SpreadLayoutResult, Error> {
    tauri::async_runtime::spawn_blocking(move || {
        app.state::<Books<R>>().layout_spread(&id, &spread)
    })
    .await
    .expect("the layout task does not panic")
}

/// Lays out the text of one master page of a book, with `{page}` and the other tokens as written.
#[tauri::command]
pub async fn layout_master<R: Runtime>(
    app: AppHandle<R>,
    _window: WebviewWindow<R>,
    id: String,
    master: String,
) -> Result<SpreadLayoutResult, Error> {
    tauri::async_runtime::spawn_blocking(move || {
        app.state::<Books<R>>().layout_master(&id, &master)
    })
    .await
    .expect("the layout task does not panic")
}

/// Reads a picture file into a book and answers with its path in the book's package.
#[tauri::command]
pub async fn import_image<R: Runtime>(
    app: AppHandle<R>,
    _window: WebviewWindow<R>,
    id: String,
    path: PathBuf,
) -> Result<String, Error> {
    // Reading and hashing a large picture is work for a blocking thread, not the async runtime.
    tauri::async_runtime::spawn_blocking(move || app.state::<Books<R>>().import_image(&id, &path))
        .await
        .expect("the import task does not panic")
}

/// One of a book's pictures, as raw bytes.
#[tauri::command]
pub async fn asset_data<R: Runtime>(
    app: AppHandle<R>,
    _window: WebviewWindow<R>,
    id: String,
    asset: String,
) -> Result<tauri::ipc::Response, Error> {
    let bytes = tauri::async_runtime::spawn_blocking(move || {
        app.state::<Books<R>>().asset_data(&id, &asset)
    })
    .await
    .expect("the read task does not panic")?;
    Ok(tauri::ipc::Response::new(bytes))
}

/// The words of `text` that the book's dictionary does not know and its ignore list does not hold.
#[tauri::command]
pub async fn spell_check<R: Runtime>(
    app: AppHandle<R>,
    _window: WebviewWindow<R>,
    id: String,
    text: String,
    personal: Option<Vec<String>>,
) -> Result<SpellCheck, Error> {
    // Loading the dictionary the first time is work for a blocking thread, not the async runtime.
    tauri::async_runtime::spawn_blocking(move || {
        app.state::<Books<R>>()
            .spell_check(&id, &text, &personal.unwrap_or_default())
    })
    .await
    .expect("the spelling task does not panic")
}

/// Corrections for a word, best first.
#[tauri::command]
pub async fn spell_suggest<R: Runtime>(
    app: AppHandle<R>,
    _window: WebviewWindow<R>,
    id: String,
    word: String,
) -> Result<Vec<String>, Error> {
    tauri::async_runtime::spawn_blocking(move || app.state::<Books<R>>().spell_suggest(&id, &word))
        .await
        .expect("the spelling task does not panic")
}

/// Every dictionary there is, and whether it is in place.
#[tauri::command]
pub async fn dictionaries<R: Runtime>(
    _window: WebviewWindow<R>,
    books: State<'_, Books<R>>,
) -> Result<Vec<DictionaryStatus>, Error> {
    Ok(books.spelling.status())
}

/// Downloads a dictionary, checks it against what is pinned for it, and answers with the list again.
#[tauri::command]
pub async fn download_dictionary<R: Runtime>(
    app: AppHandle<R>,
    _window: WebviewWindow<R>,
    id: String,
) -> Result<Vec<DictionaryStatus>, Error> {
    // The download is work for a blocking thread, not the async runtime.
    tauri::async_runtime::spawn_blocking(move || {
        let books = app.state::<Books<R>>();
        books.spelling.download(&id)?;
        Ok(books.spelling.status())
    })
    .await
    .expect("the download task does not panic")
}

/// Takes a downloaded dictionary away, and answers with the list again.
#[tauri::command]
pub async fn remove_dictionary<R: Runtime>(
    _window: WebviewWindow<R>,
    books: State<'_, Books<R>>,
    id: String,
) -> Result<Vec<DictionaryStatus>, Error> {
    books.spelling.remove(&id)?;
    Ok(books.spelling.status())
}
