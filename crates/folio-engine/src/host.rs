// What the books need from the app that hosts them
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::path::PathBuf;

use crate::wire::{BooksChanged, ExportProgress, PatchEvent};

/// How the books reach the outside: events out, and the person's folders in. The desktop app
/// answers with its windows and Tauri's paths; a command-line host answers with a log and the
/// environment.
pub trait Host: Send + Sync + 'static {
    /// The open books changed: one opened, closed, saved or became changed or clean.
    fn books_changed(&self, changed: &BooksChanged);
    /// A change was made to an open book.
    fn patched(&self, event: &PatchEvent);
    /// An export reached its next file.
    fn export_progress(&self, progress: &ExportProgress);
    /// The person's home folder.
    fn home_dir(&self) -> Option<PathBuf>;
    /// The person's Documents folder.
    fn document_dir(&self) -> Option<PathBuf>;
    /// The version of the app, for the generator line written into books and files.
    fn version(&self) -> String;
}
