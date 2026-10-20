// The plugin's commands: new, open, save and close books, and change them with intents
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::path::PathBuf;

use folio_layout::wire::PreflightItem;
use folio_model::Intent;
use folio_presets::{KindInfo, NewBookSpec, PaintPresets};
use folio_protocol::PluginStatus;
use folio_session::Patch;
use tauri::{AppHandle, Emitter, Manager, Runtime, State, WebviewWindow};

use crate::error::Error;
use crate::state::Books;
use crate::wire::{
    BookSnapshot, BookSpelling, BookSummary, DictionaryProgress, DictionaryStatus, ExportColour,
    ExportReport, ExportTarget, FontChoice, FontUse, KeepFontsReport, ProofPage, ReaderPreview,
    Recoverable, SpellCheck, SpreadLayoutResult,
};
use crate::DICTIONARY_PROGRESS_EVENT;
use folio_import::ImportReport;

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

/// The cover of the book `spec` would make, as a `data:image/png` URI.
#[tauri::command]
pub async fn preview_new_book<R: Runtime>(
    _window: WebviewWindow<R>,
    books: State<'_, Books<R>>,
    spec: NewBookSpec,
) -> Result<String, Error> {
    books.preview_new_book(&spec)
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

/// Writes a recovery copy of each book with changes the last copy does not have, and answers with
/// how many were written. The window calls it every so often while it is open.
#[tauri::command]
pub async fn autosave_books<R: Runtime>(
    app: AppHandle<R>,
    _window: WebviewWindow<R>,
) -> Result<usize, Error> {
    // Writing files is work for a blocking thread, not the async runtime.
    tauri::async_runtime::spawn_blocking(move || Ok(app.state::<Books<R>>().autosave()))
        .await
        .expect("the autosave task does not panic")
}

/// The books a crash left changes in, that are not open.
#[tauri::command]
pub async fn recoverable_books<R: Runtime>(
    _window: WebviewWindow<R>,
    books: State<'_, Books<R>>,
) -> Result<Vec<Recoverable>, Error> {
    Ok(books.recoverable())
}

/// Opens a recovery copy as the book it was, still unsaved.
#[tauri::command]
pub async fn restore_book<R: Runtime>(
    _window: WebviewWindow<R>,
    books: State<'_, Books<R>>,
    id: String,
) -> Result<BookSnapshot, Error> {
    books.restore(&id)
}

/// Gives a recovery copy up.
#[tauri::command]
pub async fn discard_recovered<R: Runtime>(
    _window: WebviewWindow<R>,
    books: State<'_, Books<R>>,
    id: String,
) -> Result<(), Error> {
    books.discard(&id);
    Ok(())
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

/// How many steps can be undone now, to come back to with `rewind_history`.
#[tauri::command]
pub async fn history_mark<R: Runtime>(
    _window: WebviewWindow<R>,
    books: State<'_, Books<R>>,
    id: String,
) -> Result<usize, Error> {
    books.history_mark(&id)
}

/// Undoes steps until only `mark` can be undone; every window is sent each patch as an event.
#[tauri::command]
pub async fn rewind_history<R: Runtime>(
    _window: WebviewWindow<R>,
    books: State<'_, Books<R>>,
    id: String,
    mark: usize,
) -> Result<(), Error> {
    books.rewind(&id, mark)
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
    spelling: Option<Vec<String>>,
) -> Result<SpreadLayoutResult, Error> {
    tauri::async_runtime::spawn_blocking(move || {
        app.state::<Books<R>>()
            .layout_spread(&id, &spread, spelling.as_deref())
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

/// The book as an e-reader would show it, from the same chapters and CSS the EPUB ships.
#[tauri::command]
pub async fn reader_preview<R: Runtime>(
    app: AppHandle<R>,
    _window: WebviewWindow<R>,
    id: String,
) -> Result<ReaderPreview, Error> {
    tauri::async_runtime::spawn_blocking(move || app.state::<Books<R>>().reader_preview(&id))
        .await
        .expect("the preview task does not panic")
}

/// A spread as a press would print it, through the CMYK ICC profile at `profile`.
#[tauri::command]
pub async fn proof_spread<R: Runtime>(
    app: AppHandle<R>,
    _window: WebviewWindow<R>,
    id: String,
    spread: String,
    profile: String,
) -> Result<Vec<ProofPage>, Error> {
    // Drawing and converting every pixel is work for a blocking thread, not the async runtime.
    tauri::async_runtime::spawn_blocking(move || {
        app.state::<Books<R>>().proof_spread(&id, &spread, &profile)
    })
    .await
    .expect("the proof task does not panic")
}

/// The paperback cover sheet (back, spine and front) as the exporter would make it, as a
/// `data:image/png` URI; `None` for a book with no cover.
#[tauri::command]
pub async fn preview_cover_sheet<R: Runtime>(
    app: AppHandle<R>,
    _window: WebviewWindow<R>,
    id: String,
) -> Result<Option<String>, Error> {
    tauri::async_runtime::spawn_blocking(move || app.state::<Books<R>>().preview_cover_sheet(&id))
        .await
        .expect("the preview task does not panic")
}

/// Makes the files for `targets` and writes them into the book's `exports` folder.
#[tauri::command]
pub async fn export_book<R: Runtime>(
    app: AppHandle<R>,
    _window: WebviewWindow<R>,
    id: String,
    targets: Vec<ExportTarget>,
    colour: Option<ExportColour>,
) -> Result<ExportReport, Error> {
    // Laying out and writing a book is work for a blocking thread, not the async runtime.
    tauri::async_runtime::spawn_blocking(move || {
        app.state::<Books<R>>()
            .export(&id, &targets, colour.unwrap_or_default())
    })
    .await
    .expect("the export task does not panic")
}

/// Brings the words of a Markdown or plain text file in as the book's story, in one undo step.
#[tauri::command]
pub async fn import_text<R: Runtime>(
    app: AppHandle<R>,
    _window: WebviewWindow<R>,
    id: String,
    path: String,
) -> Result<ImportReport, Error> {
    tauri::async_runtime::spawn_blocking(move || app.state::<Books<R>>().import_text(&id, &path))
        .await
        .expect("the import task does not panic")
}

/// Stops the export in progress after the file it is making; what was made is kept.
#[tauri::command]
pub async fn cancel_export<R: Runtime>(
    _window: WebviewWindow<R>,
    books: State<'_, Books<R>>,
) -> Result<(), Error> {
    books.cancel_export();
    Ok(())
}

/// Shows a file in the system's file manager.
#[tauri::command]
pub async fn reveal_path<R: Runtime>(_window: WebviewWindow<R>, path: String) -> Result<(), Error> {
    crate::reveal::reveal(std::path::Path::new(&path)).map_err(Error::Export)
}

/// What preflight finds in the whole book, the issues that wait on the person first.
#[tauri::command]
pub async fn preflight_book<R: Runtime>(
    app: AppHandle<R>,
    _window: WebviewWindow<R>,
    id: String,
) -> Result<Vec<PreflightItem>, Error> {
    // Laying out every spread is work for a blocking thread, not the async runtime.
    tauri::async_runtime::spawn_blocking(move || app.state::<Books<R>>().preflight(&id))
        .await
        .expect("the preflight task does not panic")
}

/// Every family a book can draw in, and where each comes from.
#[tauri::command]
pub async fn list_fonts<R: Runtime>(
    _window: WebviewWindow<R>,
    books: State<'_, Books<R>>,
    id: String,
) -> Result<Vec<FontChoice>, Error> {
    books.font_choices(&id)
}

/// The typefaces a book uses, and where each comes from.
#[tauri::command]
pub async fn fonts_in_use<R: Runtime>(
    _window: WebviewWindow<R>,
    books: State<'_, Books<R>>,
    id: String,
) -> Result<Vec<FontUse>, Error> {
    books.fonts_in_use(&id)
}

/// Takes fonts out of a book, so it is laid out from the computer's again.
#[tauri::command]
pub async fn unkeep_fonts<R: Runtime>(
    _window: WebviewWindow<R>,
    books: State<'_, Books<R>>,
    id: String,
    families: Vec<String>,
) -> Result<(), Error> {
    books.unkeep_fonts(&id, &families)
}

/// Keeps installed fonts inside a book, so it opens the same on a computer without them, and says
/// which were kept and which could not be.
#[tauri::command]
pub async fn keep_fonts<R: Runtime>(
    app: AppHandle<R>,
    _window: WebviewWindow<R>,
    id: String,
    families: Vec<String>,
) -> Result<KeepFontsReport, Error> {
    // Reading font files is work for a blocking thread, not the async runtime.
    tauri::async_runtime::spawn_blocking(move || app.state::<Books<R>>().keep_fonts(&id, &families))
        .await
        .expect("the font task does not panic")
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
        let sent = id.clone();
        let tell = |received: u64, total: u64| {
            let event = DictionaryProgress {
                id: sent.clone(),
                received: received as u32,
                total: total as u32,
            };
            if let Err(e) = app.emit(DICTIONARY_PROGRESS_EVENT, event) {
                log::warn!("could not announce a download's progress: {e}");
            }
        };
        books.spelling.download(&id, &tell)?;
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

/// Every misspelt word in a book, and where it is.
#[tauri::command]
pub async fn spell_check_book<R: Runtime>(
    app: AppHandle<R>,
    _window: WebviewWindow<R>,
    id: String,
    personal: Option<Vec<String>>,
) -> Result<BookSpelling, Error> {
    tauri::async_runtime::spawn_blocking(move || {
        app.state::<Books<R>>()
            .spell_check_book(&id, &personal.unwrap_or_default())
    })
    .await
    .expect("the spelling task does not panic")
}
