// The host of the books when no window is open: nobody to tell, and the folders from the environment
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::path::PathBuf;

use folio_engine::{BooksChanged, ExportProgress, Host, PatchEvent};

/// Hosts the books for an agent. There is no window to tell about changes, so events are only
/// logged, and the person's folders come from the environment.
pub struct CliHost;

impl Host for CliHost {
    fn books_changed(&self, changed: &BooksChanged) {
        log::debug!("{} books open", changed.books.len());
    }

    fn patched(&self, event: &PatchEvent) {
        log::debug!("book {} is at revision {}", event.id, event.patch.revision);
    }

    fn export_progress(&self, progress: &ExportProgress) {
        log::debug!("export {}/{}", progress.done, progress.total);
    }

    fn home_dir(&self) -> Option<PathBuf> {
        std::env::var_os("HOME")
            .or_else(|| std::env::var_os("USERPROFILE"))
            .map(PathBuf::from)
    }

    fn document_dir(&self) -> Option<PathBuf> {
        self.home_dir().map(|home| home.join("Documents"))
    }

    fn version(&self) -> String {
        env!("CARGO_PKG_VERSION").to_owned()
    }
}

/// The folder the desktop app keeps its dictionaries and recovery copies in, so the server finds
/// the dictionaries the person already downloaded.
pub fn app_data_dir() -> Option<PathBuf> {
    const IDENTIFIER: &str = "ca.liminalhq.folio";
    if cfg!(windows) {
        std::env::var_os("APPDATA").map(|d| PathBuf::from(d).join(IDENTIFIER))
    } else if cfg!(target_os = "macos") {
        CliHost
            .home_dir()
            .map(|h| h.join("Library/Application Support").join(IDENTIFIER))
    } else {
        std::env::var_os("XDG_DATA_HOME")
            .map(PathBuf::from)
            .or_else(|| CliHost.home_dir().map(|h| h.join(".local/share")))
            .map(|d| d.join(IDENTIFIER))
    }
}
