// The open books: one session and package each, one writer, events to every window
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard};

use folio_model::Intent;
use folio_package::{Package, SaveOptions};
use folio_presets::{kind_infos, new_book, suggest_save_path, KindInfo, NewBookSpec};
use folio_session::{Patch, Session};
use tauri::{AppHandle, Emitter, Manager, Runtime};

use crate::error::Error;
use crate::wire::{BookSnapshot, BookSummary, BooksChanged, PatchEvent};
use crate::{CHANGED_EVENT, PATCH_EVENT};

/// What happened to the open books, for the app to react to (the recents list, for one).
#[derive(Debug, Clone)]
pub enum BookEvent {
    /// A book was opened from a file, or was already open.
    Opened(BookSummary),
    /// A book was written to disk.
    Saved(BookSummary),
    Closed(String),
}

/// Runs after a book opens, saves or closes, outside the plugin's locks.
pub type BookHook = Arc<dyn Fn(&BookEvent) + Send + Sync>;

struct OpenBook {
    id: String,
    session: Session,
    package: Package,
    path: Option<PathBuf>,
}

impl OpenBook {
    fn summary(&self) -> BookSummary {
        let book = self.session.book();
        BookSummary {
            id: self.id.clone(),
            title: book.title.clone(),
            kind: book.kind,
            path: self.path.as_ref().map(|p| p.to_string_lossy().into_owned()),
            dirty: self.path.is_none() || self.session.is_dirty(),
        }
    }

    fn snapshot(&self) -> BookSnapshot {
        BookSnapshot {
            summary: self.summary(),
            revision: self.session.revision(),
            book: self.session.book().clone(),
            styles: self.package.styles().clone(),
            settings: self.package.settings().clone(),
            can_undo: self.session.can_undo(),
            can_redo: self.session.can_redo(),
        }
    }
}

#[derive(Default)]
struct Inner {
    books: Vec<OpenBook>,
    next: u64,
}

fn locked<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    // A poisoned lock only means a hook panicked; the books are still consistent.
    mutex.lock().unwrap_or_else(|e| e.into_inner())
}

/// The books every window can see. Rust owns them: a window asks for a change and hears the answer
/// as an event.
pub struct Books<R: Runtime> {
    app: AppHandle<R>,
    inner: Mutex<Inner>,
    hooks: Mutex<Vec<BookHook>>,
    generator: String,
}

impl<R: Runtime> Books<R> {
    pub fn new(app: AppHandle<R>) -> Self {
        let generator = format!("Folio {} (Liminal HQ)", app.package_info().version);
        Self {
            app,
            inner: Mutex::new(Inner::default()),
            hooks: Mutex::new(Vec::new()),
            generator,
        }
    }

    /// Runs `hook` after a book opens, saves or closes.
    pub fn on_book(&self, hook: impl Fn(&BookEvent) + Send + Sync + 'static) {
        locked(&self.hooks).push(Arc::new(hook));
    }

    fn fire(&self, event: BookEvent) {
        let hooks: Vec<BookHook> = locked(&self.hooks).clone();
        for hook in hooks {
            hook(&event);
        }
    }

    fn announce(&self, inner: &Inner) {
        let books = inner.books.iter().map(OpenBook::summary).collect();
        if let Err(e) = self.app.emit(CHANGED_EVENT, BooksChanged { books }) {
            log::warn!("could not announce the open books: {e}");
        }
    }

    fn announce_patch(&self, book: &OpenBook, patch: &Patch) {
        let event = PatchEvent {
            id: book.id.clone(),
            patch: patch.clone(),
            can_undo: book.session.can_undo(),
            can_redo: book.session.can_redo(),
        };
        if let Err(e) = self.app.emit(PATCH_EVENT, event) {
            log::warn!("could not announce a change to a book: {e}");
        }
    }

    pub fn presets(&self) -> Vec<KindInfo> {
        kind_infos()
    }

    /// Where Save as… starts for a book called `title`: `<Books>/<Title>/<Title>.folio`, with
    /// `Books` in the home folder (in Documents on Windows).
    pub fn default_save_path(&self, title: &str) -> PathBuf {
        let paths = self.app.path();
        let base = if cfg!(windows) {
            paths.document_dir()
        } else {
            paths.home_dir()
        };
        let books = base.unwrap_or_default().join("Books");
        suggest_save_path(title, &books)
    }

    fn add(&self, session: Session, package: Package, path: Option<PathBuf>) -> BookSnapshot {
        let mut inner = locked(&self.inner);
        inner.next += 1;
        let book = OpenBook {
            id: format!("bs_{}", inner.next),
            session,
            package,
            path,
        };
        let snapshot = book.snapshot();
        inner.books.push(book);
        self.announce(&inner);
        snapshot
    }

    /// Starts a new book, unsaved: nothing is written until the first save.
    pub fn new_book(&self, spec: &NewBookSpec) -> Result<BookSnapshot, Error> {
        let made = new_book(spec)?;
        let mut package = Package::new(made.book.clone());
        *package.styles_mut() = made.styles;
        *package.settings_mut() = made.settings;
        Ok(self.add(Session::new(made.book), package, None))
    }

    /// Opens a `.folio` file, or returns the session it is already open in.
    pub fn open(&self, path: &Path) -> Result<BookSnapshot, Error> {
        let canonical = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
        {
            let inner = locked(&self.inner);
            if let Some(open) = inner
                .books
                .iter()
                .find(|b| b.path.as_deref() == Some(&canonical))
            {
                return Ok(open.snapshot());
            }
        }
        let package = Package::open(&canonical)?;
        let session = Session::new(package.book().clone());
        let snapshot = self.add(session, package, Some(canonical));
        self.fire(BookEvent::Opened(snapshot.summary.clone()));
        Ok(snapshot)
    }

    /// Saves a book to `path`, or to where it already is.
    pub fn save(&self, id: &str, path: Option<&Path>) -> Result<BookSummary, Error> {
        let summary = {
            let mut inner = locked(&self.inner);
            let open = inner
                .books
                .iter_mut()
                .find(|b| b.id == id)
                .ok_or_else(|| Error::NoSuchBook(id.into()))?;
            let target = match (path, &open.path) {
                (Some(path), _) => path.to_path_buf(),
                (None, Some(existing)) => existing.clone(),
                (None, None) => return Err(Error::NeedsPath),
            };
            if let Some(parent) = target.parent() {
                std::fs::create_dir_all(parent).map_err(|e| folio_package::PackageError::Io {
                    path: parent.to_path_buf(),
                    source: e,
                })?;
            }
            *open.package.book_mut() = open.session.book().clone();
            let options = SaveOptions {
                generator: self.generator.clone(),
                ..SaveOptions::default()
            };
            open.package.save(&target, &options)?;
            open.session.mark_saved();
            open.path = Some(std::fs::canonicalize(&target).unwrap_or(target));
            let summary = open.summary();
            self.announce(&inner);
            summary
        };
        self.fire(BookEvent::Saved(summary.clone()));
        Ok(summary)
    }

    /// Closes a book, whether or not it has been saved: asking first is the window's job.
    pub fn close(&self, id: &str) -> Result<(), Error> {
        {
            let mut inner = locked(&self.inner);
            let at = inner
                .books
                .iter()
                .position(|b| b.id == id)
                .ok_or_else(|| Error::NoSuchBook(id.into()))?;
            inner.books.remove(at);
            self.announce(&inner);
        }
        self.fire(BookEvent::Closed(id.into()));
        Ok(())
    }

    pub fn list(&self) -> Vec<BookSummary> {
        locked(&self.inner)
            .books
            .iter()
            .map(OpenBook::summary)
            .collect()
    }

    pub fn snapshot(&self, id: &str) -> Result<BookSnapshot, Error> {
        locked(&self.inner)
            .books
            .iter()
            .find(|b| b.id == id)
            .map(OpenBook::snapshot)
            .ok_or_else(|| Error::NoSuchBook(id.into()))
    }

    /// Applies an intent to a book, tells every window, and answers with what changed.
    pub fn apply(&self, id: &str, intent: Intent) -> Result<Patch, Error> {
        self.change(
            id,
            |session| session.apply(intent).map_err(Error::from),
            |patch: &Patch| Some(patch),
        )
    }

    /// Undoes the last step, or `None` when there is nothing to undo.
    pub fn undo(&self, id: &str) -> Result<Option<Patch>, Error> {
        self.change(
            id,
            |session| Ok(session.undo()),
            |patch: &Option<Patch>| patch.as_ref(),
        )
    }

    /// Redoes the last undone step, or `None` when there is nothing to redo.
    pub fn redo(&self, id: &str) -> Result<Option<Patch>, Error> {
        self.change(
            id,
            |session| Ok(session.redo()),
            |patch: &Option<Patch>| patch.as_ref(),
        )
    }

    /// Runs a change on a book's session and tells every window what happened: the patch when
    /// there is one, and the list of books when this one became changed or clean.
    fn change<T>(
        &self,
        id: &str,
        run: impl FnOnce(&mut Session) -> Result<T, Error>,
        patch_of: impl FnOnce(&T) -> Option<&Patch>,
    ) -> Result<T, Error> {
        let mut inner = locked(&self.inner);
        let open = inner
            .books
            .iter_mut()
            .find(|b| b.id == id)
            .ok_or_else(|| Error::NoSuchBook(id.into()))?;
        let was = open.summary();
        let result = run(&mut open.session)?;
        if let Some(patch) = patch_of(&result).filter(|p| !p.ops.is_empty()) {
            self.announce_patch(open, patch);
        }
        let now = open.summary();
        if now != was {
            self.announce(&inner);
        }
        Ok(result)
    }
}
