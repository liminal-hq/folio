// The open books: one session and package each, one writer, events to every window
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard, OnceLock};

use folio_layout::assets::sniff;
use folio_layout::preflight::spread_assets;
use folio_layout::wire::PreflightItem;
use folio_layout::{AssetFact, AssetFacts, BookFace, FontBook, FontSource, Layouter, StyleSheet};
use folio_model::{BookFont, Intent, SpreadId};
use folio_package::{Package, SaveOptions};
use folio_presets::{kind_infos, new_book, suggest_save_path, KindInfo, NewBookSpec};
use folio_session::{Patch, Session};
use folio_spell::{find_misspellings, Checker};
use tauri::{AppHandle, Emitter, Manager, Runtime};

use crate::error::Error;
use crate::recovery::{self, Recovery};
use crate::spelling::Spelling;
use crate::wire::{
    BookSnapshot, BookSpelling, BookSummary, BooksChanged, FontChoice, FontOrigin, FontSkip,
    KeepFontsReport, Misspelling, MisspeltWord, PatchEvent, Recoverable, SkippedFont, SpellCheck,
    SpreadLayoutResult,
};
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

/// The kinds of file a picture frame can show, which the webview can draw.
const IMAGE_EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "gif", "webp", "svg"];

/// Whether the start of a file is what its extension says, so a renamed file is not taken.
fn looks_like(extension: &str, bytes: &[u8]) -> bool {
    match extension {
        "png" => bytes.starts_with(b"\x89PNG\r\n\x1a\n"),
        "jpg" | "jpeg" => bytes.starts_with(&[0xff, 0xd8, 0xff]),
        "gif" => bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a"),
        "webp" => bytes.len() > 12 && &bytes[..4] == b"RIFF" && &bytes[8..12] == b"WEBP",
        "svg" => {
            let head = String::from_utf8_lossy(&bytes[..bytes.len().min(2048)]);
            head.contains("<svg")
        }
        _ => false,
    }
}

/// The most misspelt words the proofing list holds.
const MAX_LISTED_WORDS: usize = 500;

/// The largest picture file taken in whole.
const MAX_IMAGE_BYTES: u64 = 100 * 1024 * 1024;

/// The largest font file a book will keep: a family with a bigger one is left to the computer.
const MAX_KEPT_FONT_BYTES: usize = 32 * 1024 * 1024;

/// The extension a font file should be stored under, from its first bytes.
fn font_extension(data: &[u8]) -> &'static str {
    match data.get(..4) {
        Some(b"OTTO") => "otf",
        Some(b"ttcf") => "ttc",
        _ => "ttf",
    }
}

struct OpenBook {
    id: String,
    session: Session,
    package: Package,
    path: Option<PathBuf>,
    /// Where the copy of this book that a crash would be recovered from is kept.
    recovery_id: String,
    /// The revision the copy was last written at.
    autosaved: Option<u64>,
    /// What is known of each picture, read from its header the first time layout needs it. A
    /// picture's path is its content hash, so what is known of it never goes stale.
    facts: HashMap<String, AssetFact>,
    /// The fonts the book keeps, laid over the app's, made again only when the list changes.
    fonts: Option<(Vec<BookFont>, Arc<FontBook>)>,
}

impl OpenBook {
    /// The font book to lay this book out with: `base` with the fonts the book keeps added, or
    /// `None` when it keeps none. A file that cannot be read is left out, so the family falls back
    /// as it would if it were missing.
    fn font_book(&mut self, base: &FontBook) -> Option<Arc<FontBook>> {
        let kept = &self.session.book().fonts;
        if kept.is_empty() {
            self.fonts = None;
            return None;
        }
        if let Some((list, book)) = &self.fonts {
            if list == kept {
                return Some(Arc::clone(book));
            }
        }
        let list = kept.clone();
        let faces: Vec<BookFace> = list
            .iter()
            .filter_map(|font| {
                let data = self.package.asset_bytes(&font.path).ok()?;
                Some(BookFace {
                    family: font.family.clone(),
                    weight: font.weight,
                    italic: font.italic,
                    index: font.index,
                    data: Arc::from(data),
                })
            })
            .collect();
        let book = Arc::new(base.layered(faces));
        self.fonts = Some((list, Arc::clone(&book)));
        Some(book)
    }

    /// What is known of the pictures spread `spread` uses. A picture that is not in the package is
    /// left out, which is how layout knows it is missing; one that cannot be read is unreadable.
    fn facts_for(&mut self, book: &folio_model::Book, spread: &str) -> AssetFacts {
        let mut known = HashMap::new();
        let Some(found) = book.spreads.iter().find(|s| s.id.0 == spread) else {
            return AssetFacts(known);
        };
        for asset in spread_assets(book, found) {
            if let Some(fact) = self.facts.get(&asset) {
                known.insert(asset, *fact);
            } else if self.package.asset_paths().any(|p| p == asset) {
                let fact = match self.package.asset_bytes(&asset) {
                    Ok(bytes) => sniff(&asset, &bytes),
                    Err(_) => AssetFact::Unreadable,
                };
                self.facts.insert(asset.clone(), fact);
                known.insert(asset, fact);
            }
        }
        AssetFacts(known)
    }

    /// Whether a crash now would lose changes a copy does not have: the book has changes worth
    /// keeping (a book never saved has them once touched), and it has changed since the copy.
    fn needs_a_copy(&self) -> bool {
        let touched = if self.path.is_none() {
            self.session.revision() > 0
        } else {
            self.session.is_dirty()
        };
        touched && self.autosaved != Some(self.session.revision())
    }

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
            styles: serde_json::to_value(&self.session.book().styles).unwrap_or_default(),
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
    /// The bundled fonts and the system's, found the first time a spread is laid out.
    fonts: OnceLock<FontBook>,
    /// The dictionaries: the one that comes with Folio, and those downloaded.
    pub(crate) spelling: Spelling,
    /// Copies of books with changes, for coming back after a crash.
    pub(crate) recovery: Recovery,
}

impl<R: Runtime> Books<R> {
    pub fn new(app: AppHandle<R>) -> Self {
        let generator = format!("Folio {} (Liminal HQ)", app.package_info().version);
        Self {
            app,
            inner: Mutex::new(Inner::default()),
            hooks: Mutex::new(Vec::new()),
            generator,
            fonts: OnceLock::new(),
            spelling: Spelling::default(),
            recovery: Recovery::default(),
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
        self.add_recovering(session, package, path, None)
    }

    /// Adds a book. `recovery_id` is the copy it was brought back from, which it goes on using.
    fn add_recovering(
        &self,
        session: Session,
        package: Package,
        path: Option<PathBuf>,
        recovery_id: Option<String>,
    ) -> BookSnapshot {
        let mut inner = locked(&self.inner);
        inner.next += 1;
        let book = OpenBook {
            id: format!("bs_{}", inner.next),
            session,
            package,
            path,
            recovery_id: recovery_id.unwrap_or_else(|| recovery::new_id(inner.next)),
            autosaved: None,
            facts: HashMap::new(),
            fonts: None,
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
        *package.settings_mut() = made.settings;
        Ok(self.add(Session::new(made.book), package, None))
    }

    /// Opens a `.folio` file, or returns the session it is already open in.
    pub fn open(&self, path: &Path) -> Result<BookSnapshot, Error> {
        let canonical = dunce::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
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
            open.path = Some(dunce::canonicalize(&target).unwrap_or(target));
            // What the copy kept is in the file now.
            self.recovery.remove(&open.recovery_id);
            open.autosaved = None;
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
            let closed = inner.books.remove(at);
            // Closed on purpose, saved or not: nothing is left to recover.
            self.recovery.remove(&closed.recovery_id);
            self.announce(&inner);
        }
        self.fire(BookEvent::Closed(id.into()));
        Ok(())
    }

    /// Writes a recovery copy of each book that has changes one does not have yet, and says how many
    /// were written. A copy that cannot be written is logged and tried again next time.
    pub fn autosave(&self) -> usize {
        let mut inner = locked(&self.inner);
        let mut written = 0;
        for open in inner.books.iter_mut().filter(|b| b.needs_a_copy()) {
            *open.package.book_mut() = open.session.book().clone();
            let title = open.session.book().title.clone();
            let revision = open.session.revision();
            match self.recovery.write(
                &open.recovery_id,
                &title,
                open.path.as_deref(),
                &mut open.package,
                &self.generator,
            ) {
                Ok(()) => {
                    open.autosaved = Some(revision);
                    written += 1;
                }
                Err(e) => log::warn!("could not write a recovery copy of `{title}`: {e}"),
            }
        }
        written
    }

    /// The recovery copies of books that are not open, newest first.
    pub fn recoverable(&self) -> Vec<Recoverable> {
        let open: Vec<String> = locked(&self.inner)
            .books
            .iter()
            .map(|b| b.recovery_id.clone())
            .collect();
        self.recovery.list(&open)
    }

    /// Opens a recovery copy as the book it was, still unsaved: its file is where it was kept, so a
    /// save puts it back, and a book never saved comes back untitled.
    pub fn restore(&self, id: &str) -> Result<BookSnapshot, Error> {
        let (package, note) = self.recovery.open(id)?;
        let session = Session::recovered(package.book().clone());
        let path = note.path.map(PathBuf::from);
        let snapshot = self.add_recovering(session, package, path, Some(note.id));
        self.fire(BookEvent::Opened(snapshot.summary.clone()));
        Ok(snapshot)
    }

    /// Gives a recovery copy up. A copy of a book that is open is left to that book.
    pub fn discard(&self, id: &str) {
        let open = locked(&self.inner)
            .books
            .iter()
            .any(|b| b.recovery_id == id);
        if !open {
            self.recovery.remove(id);
        }
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

    /// Lays out one spread of a book at the revision it is at now. The book is copied under the lock
    /// and laid out after it is let go, so the answer and its revision agree and a long layout never
    /// holds up another command.
    ///
    /// With `spelling` (the words added for every book), misspelt words are marked in the lines.
    pub fn layout_spread(
        &self,
        id: &str,
        spread: &str,
        spelling: Option<&[String]>,
    ) -> Result<SpreadLayoutResult, Error> {
        let base = self.fonts.get_or_init(FontBook::with_system_fonts);
        let (book, styles, revision, facts, layered) = {
            let mut inner = locked(&self.inner);
            let open = inner
                .books
                .iter_mut()
                .find(|b| b.id == id)
                .ok_or_else(|| Error::NoSuchBook(id.into()))?;
            let book = open.session.book().clone();
            let facts = open.facts_for(&book, spread);
            let layered = open.font_book(base);
            (
                book,
                style_sheet(&open.session.book().styles),
                open.session.revision(),
                facts,
                layered,
            )
        };
        let fonts = layered.as_deref().unwrap_or(base);
        // Words are checked as the book's dictionary says, leaving alone those the book ignores and
        // those added for every book.
        let mut ignore = book.proofing.ignore.clone();
        ignore.extend(spelling.unwrap_or_default().iter().cloned());
        let speller = spelling
            .and_then(|_| self.spelling.for_language(&book.language))
            .map(|(_, checker)| checker);
        let find = |text: &str| -> Vec<std::ops::Range<usize>> {
            speller
                .as_deref()
                .map(|checker| {
                    find_misspellings(checker, text, &ignore)
                        .into_iter()
                        .map(|m| m.bytes)
                        .collect()
                })
                .unwrap_or_default()
        };
        let mut layouter = Layouter::new(&book, &styles, fonts).with_assets(&facts);
        if speller.is_some() {
            layouter = layouter.with_proofing(&find);
        }
        let layout = layouter
            .layout_spread(&SpreadId(spread.to_string()))?
            .ok_or_else(|| Error::NoSuchSpread(spread.to_string()))?;
        Ok(SpreadLayoutResult { revision, layout })
    }

    /// Lays out the text of one master page of a book, with its tokens as written, at the revision
    /// the book is at now.
    pub fn layout_master(&self, id: &str, master: &str) -> Result<SpreadLayoutResult, Error> {
        let base = self.fonts.get_or_init(FontBook::with_system_fonts);
        let (book, styles, revision, layered) = {
            let mut inner = locked(&self.inner);
            let open = inner
                .books
                .iter_mut()
                .find(|b| b.id == id)
                .ok_or_else(|| Error::NoSuchBook(id.into()))?;
            let layered = open.font_book(base);
            (
                open.session.book().clone(),
                style_sheet(&open.session.book().styles),
                open.session.revision(),
                layered,
            )
        };
        let fonts = layered.as_deref().unwrap_or(base);
        let layouter = Layouter::new(&book, &styles, fonts);
        let layout = layouter
            .layout_master(&folio_model::MasterId(master.to_string()))?
            .ok_or_else(|| Error::NoSuchMaster(master.to_string()))?;
        Ok(SpreadLayoutResult { revision, layout })
    }

    /// The words of `text` the book's dictionary does not know and its ignore list does not hold.
    pub fn spell_check(
        &self,
        id: &str,
        text: &str,
        personal: &[String],
    ) -> Result<SpellCheck, Error> {
        let (language, ignore) = {
            let inner = locked(&self.inner);
            let open = inner
                .books
                .iter()
                .find(|b| b.id == id)
                .ok_or_else(|| Error::NoSuchBook(id.into()))?;
            let book = open.session.book();
            let mut ignore = book.proofing.ignore.clone();
            ignore.extend_from_slice(personal);
            (book.language.clone(), ignore)
        };
        let Some((dictionary, speller)) = self.spelling.for_language(&language) else {
            return Ok(SpellCheck {
                supported: false,
                dictionary: None,
                offer: self.spelling.offer(&language),
                misspellings: Vec::new(),
            });
        };
        Ok(SpellCheck {
            supported: true,
            dictionary: Some(dictionary),
            offer: self.spelling.offer(&language),
            misspellings: find_misspellings(&*speller, text, &ignore)
                .into_iter()
                .map(|m| Misspelling {
                    start: m.start as u32,
                    end: m.end as u32,
                    word: m.word,
                })
                .collect(),
        })
    }

    /// Every misspelt word in the book's own text frames, shapes, tables and flowing text, in the
    /// order of its spreads.
    pub fn spell_check_book(&self, id: &str, personal: &[String]) -> Result<BookSpelling, Error> {
        use folio_model::{Frame, InlineContent};
        let book = {
            let inner = locked(&self.inner);
            inner
                .books
                .iter()
                .find(|b| b.id == id)
                .ok_or_else(|| Error::NoSuchBook(id.into()))?
                .session
                .book()
                .clone()
        };
        let Some((dictionary, speller)) = self.spelling.for_language(&book.language) else {
            return Ok(BookSpelling {
                supported: false,
                dictionary: None,
                offer: self.spelling.offer(&book.language),
                words: Vec::new(),
                truncated: false,
            });
        };
        let mut ignore = book.proofing.ignore.clone();
        ignore.extend_from_slice(personal);
        let mut words = Vec::new();
        let mut truncated = false;
        let mut look =
            |spread: &str, frame: &str, cell: Option<(u32, u32)>, text: &str, replaceable: bool| {
                for m in find_misspellings(&*speller, text, &ignore) {
                    if words.len() >= MAX_LISTED_WORDS {
                        truncated = true;
                        return;
                    }
                    let before: String = text[..m.bytes.start]
                        .chars()
                        .rev()
                        .take(24)
                        .collect::<Vec<_>>()
                        .into_iter()
                        .rev()
                        .collect();
                    let after: String = text[m.bytes.end..].chars().take(24).collect();
                    words.push(MisspeltWord {
                        spread: spread.to_owned(),
                        frame: frame.to_owned(),
                        row: cell.map(|c| c.0),
                        col: cell.map(|c| c.1),
                        word: m.word,
                        start: m.start as u32,
                        end: m.end as u32,
                        context: format!(
                            "{}{}{}",
                            before.replace('\n', " "),
                            &text[m.bytes.clone()],
                            after.replace('\n', " ")
                        )
                        .trim()
                        .to_owned(),
                        replaceable,
                    });
                }
            };
        let plain = |content: &Option<InlineContent>| -> (String, bool) {
            let runs = content
                .as_ref()
                .map(|c| c.inline.as_slice())
                .unwrap_or_default();
            (
                runs.iter().map(|r| r.text.as_str()).collect(),
                runs.iter()
                    .all(|r| !r.bold && !r.italic && r.style.is_none()),
            )
        };
        let mut flowed = std::collections::HashSet::new();
        for spread in &book.spreads {
            for frame in &spread.frames {
                match frame {
                    Frame::Text(t) if t.flow.is_none() => {
                        let (text, ok) = plain(&t.content);
                        look(&spread.id.0, &t.base.id.0, None, &text, ok);
                    }
                    Frame::Text(t) => {
                        // Flowing text is one story across its frames: look at it once, at its first frame.
                        if let Some(flow) = t
                            .flow
                            .as_ref()
                            .and_then(|s| book.flows.iter().find(|f| f.id == s.id))
                        {
                            if flowed.insert(flow.id.0.clone()) {
                                look(&spread.id.0, &t.base.id.0, None, &flow.source, false);
                            }
                        }
                    }
                    Frame::Shape(sh) => {
                        let (text, ok) = plain(&sh.content);
                        look(
                            &spread.id.0,
                            &sh.base.id.0,
                            None,
                            &text,
                            ok && sh.shape.holds_text(),
                        );
                    }
                    Frame::Table(tb) => {
                        for (r, row) in tb.rows.iter().enumerate() {
                            for (c, cell) in row.iter().enumerate() {
                                let (text, ok) = plain(&cell.content);
                                look(
                                    &spread.id.0,
                                    &tb.base.id.0,
                                    Some((r as u32, c as u32)),
                                    &text,
                                    ok,
                                );
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        Ok(BookSpelling {
            supported: true,
            dictionary: Some(dictionary),
            offer: self.spelling.offer(&book.language),
            words,
            truncated,
        })
    }

    /// Corrections for `word` in the book's language, best first.
    pub fn spell_suggest(&self, id: &str, word: &str) -> Result<Vec<String>, Error> {
        let language = {
            let inner = locked(&self.inner);
            inner
                .books
                .iter()
                .find(|b| b.id == id)
                .ok_or_else(|| Error::NoSuchBook(id.into()))?
                .session
                .book()
                .language
                .clone()
        };
        Ok(self
            .spelling
            .for_language(&language)
            .map(|(_, s)| s.suggest(&word.replace('’', "'"), 6))
            .unwrap_or_default())
    }

    /// Reads a picture file into the book's package and returns its path there. The same bytes give
    /// the same path, so choosing a picture twice stores it once.
    pub fn import_image(&self, id: &str, path: &Path) -> Result<String, Error> {
        let extension = path
            .extension()
            .and_then(|e| e.to_str())
            .map(str::to_ascii_lowercase)
            .unwrap_or_default();
        if !IMAGE_EXTENSIONS.contains(&extension.as_str()) {
            return Err(Error::BadImage(format!(
                "`{}` is not a picture Folio can place (PNG, JPEG, GIF, WebP or SVG)",
                path.display()
            )));
        }
        // A link could lead anywhere: only a plain file is taken.
        let meta = std::fs::symlink_metadata(path)
            .map_err(|e| Error::BadImage(format!("`{}` could not be read: {e}", path.display())))?;
        if !meta.is_file() {
            return Err(Error::BadImage(format!(
                "`{}` is not a plain file",
                path.display()
            )));
        }
        let size = meta.len();
        if size > MAX_IMAGE_BYTES {
            return Err(Error::BadImage(format!(
                "`{}` is larger than {} MB",
                path.display(),
                MAX_IMAGE_BYTES / (1024 * 1024)
            )));
        }
        let bytes = std::fs::read(path)
            .map_err(|e| Error::BadImage(format!("`{}` could not be read: {e}", path.display())))?;
        if !looks_like(&extension, &bytes) {
            return Err(Error::BadImage(format!(
                "`{}` is not really a {} picture",
                path.display(),
                extension.to_ascii_uppercase()
            )));
        }
        let mut inner = locked(&self.inner);
        let open = inner
            .books
            .iter_mut()
            .find(|b| b.id == id)
            .ok_or_else(|| Error::NoSuchBook(id.into()))?;
        Ok(open.package.add_image(bytes, &extension))
    }

    /// The bytes of one of the book's pictures, for drawing it.
    pub fn asset_data(&self, id: &str, asset: &str) -> Result<Vec<u8>, Error> {
        let inner = locked(&self.inner);
        let open = inner
            .books
            .iter()
            .find(|b| b.id == id)
            .ok_or_else(|| Error::NoSuchBook(id.into()))?;
        Ok(open.package.asset_bytes(asset)?)
    }

    /// Everything preflight has to say about the whole book: each spread laid out and its issues
    /// gathered, the ones that wait on the person first and then in page order.
    pub fn preflight(&self, id: &str) -> Result<Vec<PreflightItem>, Error> {
        let spreads: Vec<String> = {
            let inner = locked(&self.inner);
            let open = inner
                .books
                .iter()
                .find(|b| b.id == id)
                .ok_or_else(|| Error::NoSuchBook(id.into()))?;
            open.session
                .book()
                .spreads
                .iter()
                .map(|s| s.id.0.clone())
                .collect()
        };
        let mut items = Vec::new();
        for spread in spreads {
            items.extend(self.layout_spread(id, &spread, None)?.layout.preflight);
        }
        // A stable sort keeps the pages in order within each severity.
        items.sort_by_key(|item| item.severity);
        Ok(items)
    }

    /// Every family the book can draw in, bundled first, then the book's own, then the computer's,
    /// each said once and with where it comes from.
    pub fn font_choices(&self, id: &str) -> Result<Vec<FontChoice>, Error> {
        let base = self.fonts.get_or_init(FontBook::with_system_fonts);
        let layered = {
            let mut inner = locked(&self.inner);
            let open = inner
                .books
                .iter_mut()
                .find(|b| b.id == id)
                .ok_or_else(|| Error::NoSuchBook(id.into()))?;
            open.font_book(base)
        };
        let fonts = layered.as_deref().unwrap_or(base);
        let mut seen: Vec<FontChoice> = Vec::new();
        for face in fonts.faces() {
            if seen
                .iter()
                .any(|c| c.family.eq_ignore_ascii_case(&face.family))
            {
                continue;
            }
            seen.push(FontChoice {
                origin: match face.source {
                    FontSource::Bundled => FontOrigin::Bundled,
                    FontSource::Book => FontOrigin::Book,
                    FontSource::System => FontOrigin::System,
                },
                family: face.family,
            });
        }
        Ok(seen)
    }

    /// Keeps the installed fonts of `families` inside the book, so it lays out the same on a
    /// computer without them (D-30). A font whose licence does not let its file travel, or that is
    /// too large, is left and reported; the rest are added to the package and the book's list in
    /// one undo step.
    pub fn keep_fonts(&self, id: &str, families: &[String]) -> Result<KeepFontsReport, Error> {
        let base = self.fonts.get_or_init(FontBook::with_system_fonts);
        let mut report = KeepFontsReport::default();
        // Reading the font files is slow and needs no lock.
        let wanted: Vec<(String, Vec<folio_layout::KeepableFace>)> = families
            .iter()
            .filter(|family| !FontBook::is_bundled(family))
            .map(|family| (family.clone(), base.installed_files_of(family)))
            .collect();
        let list = {
            let mut inner = locked(&self.inner);
            let open = inner
                .books
                .iter_mut()
                .find(|b| b.id == id)
                .ok_or_else(|| Error::NoSuchBook(id.into()))?;
            let mut list = open.session.book().fonts.clone();
            for (family, files) in wanted {
                if list.iter().any(|f| f.family.eq_ignore_ascii_case(&family)) {
                    report.kept.push(family);
                    continue;
                }
                if files.is_empty() {
                    report.skipped.push(SkippedFont {
                        family,
                        reason: FontSkip::NotInstalled,
                    });
                    continue;
                }
                if files.iter().any(|f| !f.rights.may_keep_with_book()) {
                    report.skipped.push(SkippedFont {
                        family,
                        reason: FontSkip::NotAllowed,
                    });
                    continue;
                }
                if files.iter().any(|f| f.data.len() > MAX_KEPT_FONT_BYTES) {
                    report.skipped.push(SkippedFont {
                        family,
                        reason: FontSkip::TooLarge,
                    });
                    continue;
                }
                for file in files {
                    let path = open
                        .package
                        .add_font(file.data.to_vec(), font_extension(&file.data));
                    list.push(BookFont {
                        family: file.family,
                        weight: file.weight,
                        italic: file.italic,
                        path,
                        index: file.index,
                    });
                }
                report.kept.push(family);
            }
            list
        };
        if list != self.book_fonts(id)? {
            self.apply(id, Intent::SetFonts { fonts: list })?;
        }
        Ok(report)
    }

    fn book_fonts(&self, id: &str) -> Result<Vec<BookFont>, Error> {
        let inner = locked(&self.inner);
        inner
            .books
            .iter()
            .find(|b| b.id == id)
            .map(|open| open.session.book().fonts.clone())
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

    /// How many steps of the book can be undone now: a mark for [`Books::rewind`].
    pub fn history_mark(&self, id: &str) -> Result<usize, Error> {
        let inner = locked(&self.inner);
        let open = inner
            .books
            .iter()
            .find(|b| b.id == id)
            .ok_or_else(|| Error::NoSuchBook(id.into()))?;
        Ok(open.session.undo_depth())
    }

    /// Undoes steps, one at a time and each announced as it goes, until only `mark` remain.
    pub fn rewind(&self, id: &str, mark: usize) -> Result<(), Error> {
        while self.history_mark(id)? > mark {
            if self.undo(id)?.is_none() {
                break;
            }
        }
        Ok(())
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

/// The styles a book has, with Folio's own behind them for any it never used.
fn style_sheet(styles: &folio_model::Styles) -> StyleSheet {
    StyleSheet::from_styles(styles)
        .with_fallback(&StyleSheet::from_styles(&folio_presets::preset_styles()))
}
