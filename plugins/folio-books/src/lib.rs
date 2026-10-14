// Registers the books plugin: the open books, their commands and their events
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

mod commands;
mod error;
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

/// Registers the plugin.
pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("folio-books")
        .invoke_handler(tauri::generate_handler![
            commands::get_status,
            commands::presets,
            commands::default_save_path,
            commands::new_book,
            commands::open_book,
            commands::save_book,
            commands::close_book,
            commands::list_books,
            commands::snapshot,
            commands::apply_intent,
            commands::undo,
            commands::redo,
            commands::layout_spread,
            commands::layout_master,
            commands::import_image,
            commands::asset_data,
        ])
        .setup(|app, _api| {
            app.manage(Books::new(app.clone()));
            Ok(())
        })
        .build()
}
