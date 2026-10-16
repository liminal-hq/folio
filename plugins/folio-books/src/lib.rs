// Registers the books plugin: the open books, their commands and their events
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

mod commands;
mod error;
mod recovery;
mod spelling;
mod state;
#[cfg(test)]
mod tests;
mod wire;

use tauri::{
    plugin::{Builder, TauriPlugin},
    Manager, Runtime,
};

pub use error::Error;
pub use state::{BookEvent, BookHook, Books};
pub use wire::{BookSnapshot, BookSummary, BooksChanged, PatchEvent};

/// Sent to every window after a change to an open book, with a `PatchEvent`.
pub const PATCH_EVENT: &str = "folio-books://patch";

/// Sent to every window when books open, close, save or become changed or clean, with a
/// `BooksChanged`.
pub const CHANGED_EVENT: &str = "folio-books://changed";

/// Sent to every window while a dictionary downloads: how much has arrived.
pub const DICTIONARY_PROGRESS_EVENT: &str = "folio-books://dictionary-progress";

/// Registers the plugin.
pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("folio-books")
        .invoke_handler(tauri::generate_handler![
            commands::get_status,
            commands::presets,
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
            commands::dictionaries,
            commands::download_dictionary,
            commands::remove_dictionary,
        ])
        .setup(|app, _api| {
            let books = Books::new(app.clone());
            if let Ok(dir) = app.path().app_data_dir() {
                books.spelling.keep_in(dir.join("dictionaries"));
                books.recovery.keep_in(dir.join("recovery"));
            }
            app.manage(books);
            Ok(())
        })
        .build()
}
