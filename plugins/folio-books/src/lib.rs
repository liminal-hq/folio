// Registers the books plugin: the open books, their commands and their events
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

mod commands;
mod host;
mod reveal;
#[cfg(test)]
mod tests;

use std::sync::Arc;

use tauri::{
    plugin::{Builder, TauriPlugin},
    Manager, Runtime,
};

pub use folio_engine::{
    BookEvent, BookHook, BookSnapshot, BookSummary, Books, BooksChanged, Error, PatchEvent,
    CHANGED_EVENT, DICTIONARY_PROGRESS_EVENT, EXPORT_PROGRESS_EVENT, PATCH_EVENT,
};
pub use host::TauriHost;

/// Sent to every window when the main window shows another book; the payload is the book's id, or
/// `null` when none is shown.
pub const ACTIVE_EVENT: &str = "folio-books://active";

/// The book the main window is showing, so another window (Book tools) can follow it.
#[derive(Default)]
pub struct ActiveBook(pub std::sync::Mutex<Option<String>>);

/// Registers the plugin.
pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("folio-books")
        .invoke_handler(tauri::generate_handler![
            commands::get_status,
            commands::presets,
            commands::preview_new_book,
            commands::template_gallery,
            commands::book_tool,
            commands::set_active_book,
            commands::active_book,
            commands::apply_template,
            commands::paint_presets,
            commands::default_save_path,
            commands::new_book,
            commands::open_book,
            commands::save_book,
            commands::close_book,
            commands::list_books,
            commands::snapshot,
            commands::apply_intent,
            commands::autosave_books,
            commands::recoverable_books,
            commands::restore_book,
            commands::discard_recovered,
            commands::undo,
            commands::history_mark,
            commands::rewind_history,
            commands::redo,
            commands::layout_spread,
            commands::layout_master,
            commands::import_image,
            commands::asset_data,
            commands::spell_check,
            commands::spell_suggest,
            commands::spell_check_book,
            commands::reader_preview,
            commands::preview_cover_sheet,
            commands::cover_sheet_canvas,
            commands::proof_spread,
            commands::export_book,
            commands::import_text,
            commands::cancel_export,
            commands::reveal_path,
            commands::preflight_book,
            commands::story_chapters,
            commands::fonts_in_use,
            commands::unkeep_fonts,
            commands::list_fonts,
            commands::keep_fonts,
            commands::dictionaries,
            commands::download_dictionary,
            commands::remove_dictionary,
        ])
        .setup(|app, _api| {
            // Shared: the commands use it, and the app lends it to the agents' server.
            let books = Arc::new(Books::new(Arc::new(TauriHost(app.clone()))));
            if let Ok(dir) = app.path().app_data_dir() {
                books.spelling.keep_in(dir.join("dictionaries"));
                books.recovery.keep_in(dir.join("recovery"));
            }
            app.manage(books);
            app.manage(ActiveBook::default());
            Ok(())
        })
        .build()
}
