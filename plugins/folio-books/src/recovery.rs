// Recovery copies: a copy of each book with changes, kept beside the real file, to come back to after a crash
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use folio_package::{Package, SaveOptions};
use serde::{Deserialize, Serialize};

use crate::error::Error;
use crate::wire::Recoverable;

/// The book itself, as a `.folio` file, inside a recovery folder.
const COPY: &str = "book.folio";
/// What is known of the copy: whose it is, and where the real file is.
const NOTE: &str = "recovery.json";

/// What is kept beside a copy to say whose it is.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Note {
    pub id: String,
    pub title: String,
    /// Where the real file is, or nothing for a book that was never saved.
    pub path: Option<String>,
    /// When the copy was written, in seconds since 1970.
    pub saved_at: u32,
}

/// The folder recovery copies are kept in, one folder for each book.
#[derive(Default)]
pub struct Recovery {
    folder: Mutex<Option<PathBuf>>,
}

fn locked<T>(mutex: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|e| e.into_inner())
}

fn now() -> u32 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs() as u32)
}

/// A name for a book's recovery folder that no other launch of Folio will use for another book.
pub fn new_id(counter: u64) -> String {
    format!("{:x}-{counter}", now())
}

/// Whether `id` is one this module made: letters, digits and dashes, so it cannot name another place.
fn is_safe(id: &str) -> bool {
    !id.is_empty() && id.len() <= 64 && id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
}

impl Recovery {
    /// Keeps recovery copies in `folder`.
    pub fn keep_in(&self, folder: PathBuf) {
        *locked(&self.folder) = Some(folder);
    }

    fn dir(&self, id: &str) -> Option<PathBuf> {
        if !is_safe(id) {
            return None;
        }
        locked(&self.folder).as_ref().map(|f| f.join(id))
    }

    /// Writes the book as it is now: the copy first, then the note that says it is complete. A copy
    /// without a note is never offered, so a crash in the middle of this leaves nothing half-way.
    pub fn write(
        &self,
        id: &str,
        title: &str,
        path: Option<&Path>,
        package: &mut Package,
        generator: &str,
    ) -> Result<(), Error> {
        let Some(dir) = self.dir(id) else {
            return Ok(());
        };
        fs::create_dir_all(&dir).map_err(|e| folio_package::PackageError::Io {
            path: dir.clone(),
            source: e,
        })?;
        let options = SaveOptions {
            generator: generator.to_owned(),
            keep_backup: false,
        };
        package.save_copy(dir.join(COPY), &options)?;
        let note = Note {
            id: id.to_owned(),
            title: title.to_owned(),
            path: path.map(|p| p.to_string_lossy().into_owned()),
            saved_at: now(),
        };
        let text = serde_json::to_vec_pretty(&note).map_err(|e| Error::Recovery(e.to_string()))?;
        let temporary = dir.join(format!("{NOTE}.tmp"));
        fs::write(&temporary, text)
            .and_then(|()| fs::rename(&temporary, dir.join(NOTE)))
            .map_err(|e| Error::Recovery(e.to_string()))?;
        Ok(())
    }

    /// Takes a book's copy away: it was saved, closed or given up.
    pub fn remove(&self, id: &str) {
        if let Some(dir) = self.dir(id) {
            if dir.is_dir() {
                if let Err(e) = fs::remove_dir_all(&dir) {
                    log::warn!("could not remove a recovery copy: {e}");
                }
            }
        }
    }

    fn note(dir: &Path) -> Option<Note> {
        let text = fs::read(dir.join(NOTE)).ok()?;
        let note: Note = serde_json::from_slice(&text).ok()?;
        (dir.join(COPY).is_file() && dir.file_name()?.to_str()? == note.id).then_some(note)
    }

    /// The copies there are, newest first, leaving out those of books open now.
    pub fn list(&self, open: &[String]) -> Vec<Recoverable> {
        let Some(folder) = locked(&self.folder).clone() else {
            return Vec::new();
        };
        let mut found: Vec<Recoverable> = fs::read_dir(folder)
            .into_iter()
            .flatten()
            .flatten()
            .filter_map(|entry| Self::note(&entry.path()))
            .filter(|note| !open.contains(&note.id))
            .map(|note| Recoverable {
                id: note.id,
                title: note.title,
                path: note.path,
                saved_at: note.saved_at,
            })
            .collect();
        found.sort_by(|a, b| b.saved_at.cmp(&a.saved_at).then_with(|| a.id.cmp(&b.id)));
        found
    }

    /// Opens a copy, with the note saying whose it is.
    pub fn open(&self, id: &str) -> Result<(Package, Note), Error> {
        let dir = self
            .dir(id)
            .ok_or_else(|| Error::Recovery(format!("there is no recovery copy `{id}`")))?;
        let note = Self::note(&dir)
            .ok_or_else(|| Error::Recovery(format!("there is no recovery copy `{id}`")))?;
        Ok((Package::open(dir.join(COPY))?, note))
    }
}
