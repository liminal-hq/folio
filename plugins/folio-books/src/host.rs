// The books as Tauri hosts them: events to every window, and Tauri's folders
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::path::PathBuf;

use folio_engine::{
    BooksChanged, ExportProgress, Host, PatchEvent, CHANGED_EVENT, EXPORT_PROGRESS_EVENT,
    PATCH_EVENT,
};
use tauri::{AppHandle, Emitter, Manager, Runtime};

/// Tells every window what happens to the books, and answers for the folders.
pub struct TauriHost<R: Runtime>(pub AppHandle<R>);

impl<R: Runtime> Host for TauriHost<R> {
    fn books_changed(&self, changed: &BooksChanged) {
        if let Err(e) = self.0.emit(CHANGED_EVENT, changed) {
            log::warn!("could not announce the open books: {e}");
        }
    }

    fn patched(&self, event: &PatchEvent) {
        if let Err(e) = self.0.emit(PATCH_EVENT, event) {
            log::warn!("could not announce a change to a book: {e}");
        }
    }

    fn export_progress(&self, progress: &ExportProgress) {
        if let Err(e) = self.0.emit(EXPORT_PROGRESS_EVENT, progress) {
            log::warn!("could not send the export's progress: {e}");
        }
    }

    fn home_dir(&self) -> Option<PathBuf> {
        self.0.path().home_dir().ok()
    }

    fn document_dir(&self) -> Option<PathBuf> {
        self.0.path().document_dir().ok()
    }

    fn version(&self) -> String {
        self.0.package_info().version.to_string()
    }
}
