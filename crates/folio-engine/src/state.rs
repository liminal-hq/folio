// The open books: one session and package each, one writer, events to every window
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, OnceLock};

use folio_layout::assets::sniff;
use folio_layout::preflight::spread_assets;
use folio_layout::wire::PreflightItem;
use folio_layout::{
    AssetFact, AssetFacts, BookFace, FlowCache, FontBook, FontSource, Layouter, StyleSheet,
};
use folio_model::{BookFont, Intent, SpreadId};
use folio_package::{Package, SaveOptions};
use folio_presets::{kind_infos, new_book, suggest_save_path, KindInfo, NewBookSpec};
use folio_session::{Patch, Session};
use folio_spell::{find_misspellings, Checker};

use crate::error::Error;
use crate::recovery::{self, Recovery};
use crate::spelling::Spelling;
use crate::wire::{
    BatchOutcome, BookSnapshot, BookSpelling, BookSummary, BooksChanged, CoverCanvas,
    CoverSheetPreview, ExportColour, ExportProgress, ExportReport, ExportTarget, ExportedFile,
    FontChoice, FontLicence, FontOrigin, FontSkip, FontUse, KeepFontsReport, Misspelling,
    MisspeltWord, PatchEvent, PreviewChapter, PreviewImage, ProofColour, ProofPage, ProofSpread,
    ReaderPreview, Recoverable, SkippedFont, SpellCheck, SpreadLayoutResult, StoryChapter,
};
use crate::Host;

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

/// The pictures an export reads, by their path in the package.
struct MemoryAssets(HashMap<String, Vec<u8>>);

impl folio_export::pdf::Assets for MemoryAssets {
    fn bytes(&self, path: &str) -> Option<Vec<u8>> {
        self.0.get(path).cloned()
    }
}

/// Every picture the book draws: on its pages, on its master pages and as page backgrounds.
fn used_assets(book: &folio_model::Book) -> Vec<String> {
    let mut found: Vec<String> = book
        .spreads
        .iter()
        .flat_map(|s| spread_assets(book, s))
        .collect();
    for master in &book.master_pages {
        for frame in &master.frames {
            if let folio_model::Frame::Image(image) = frame {
                found.extend(image.image.asset.clone());
            }
        }
        if let Some(folio_model::Background::Picture { asset, .. }) = &master.bg {
            found.push(asset.clone());
        }
    }
    found.extend(book.back_notes.iter().filter_map(|n| n.picture.clone()));
    found.sort();
    found.dedup();
    found
}

/// The hold on a book's file while it is open: an exclusive lock on a small file beside it, which
/// the system lets go of if the program dies, so no lock is ever left stale. Two programs that
/// both open the book (the app and an agent's server, say) would each overwrite the other's saves.
struct BookLock {
    file: std::fs::File,
    path: PathBuf,
}

impl BookLock {
    /// Takes the lock beside the book at `book`. `None` when it cannot be had for a reason other
    /// than another program holding it (a folder that cannot be written): the book opens unlocked.
    fn take(book: &Path) -> Result<Option<Self>, Error> {
        let Some(name) = book.file_name() else {
            return Ok(None);
        };
        let mut name = name.to_os_string();
        name.push(".lock");
        let path = book.with_file_name(name);
        let file = match std::fs::OpenOptions::new()
            .create(true)
            .truncate(false)
            .write(true)
            .open(&path)
        {
            Ok(file) => file,
            Err(e) => {
                log::debug!("no lock beside {}: {e}", book.display());
                return Ok(None);
            }
        };
        match file.try_lock() {
            Ok(()) => Ok(Some(Self { file, path })),
            Err(std::fs::TryLockError::WouldBlock) => {
                Err(Error::Locked(book.to_string_lossy().into_owned()))
            }
            Err(std::fs::TryLockError::Error(e)) => {
                log::debug!("no lock on {}: {e}", book.display());
                Ok(None)
            }
        }
    }
}

impl Drop for BookLock {
    fn drop(&mut self) {
        // Removed while still held, so no one else can have taken it in between.
        let _ = std::fs::remove_file(&self.path);
        let _ = self.file.unlock();
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
    /// The flows laid out at one revision of the book, shared by the spreads laid out at it.
    flow_cache: Option<(u64, Arc<FlowCache>)>,
    /// Keeps other programs from opening the book's file while it is open here.
    lock: Option<BookLock>,
}

impl OpenBook {
    /// The cache of laid out flows for the book as it is now: the one kept while the revision stays
    /// the same, a new one when it has changed.
    fn flows_laid(&mut self) -> Arc<FlowCache> {
        let revision = self.session.revision();
        match &self.flow_cache {
            Some((at, cache)) if *at == revision => Arc::clone(cache),
            _ => {
                let cache = Arc::new(FlowCache::new());
                self.flow_cache = Some((revision, Arc::clone(&cache)));
                cache
            }
        }
    }

    /// The paper that decides how thick the spine is: the book's own, and for a book made before the
    /// paper was part of it, the one its settings name.
    fn paper_in_use(&self) -> folio_model::Paper {
        let book = self.session.book();
        if book.paper != folio_model::Paper::default() {
            return book.paper;
        }
        match self
            .package
            .settings()
            .get("paper")
            .and_then(|p| p.as_str())
        {
            Some("cream") => folio_model::Paper::Cream,
            Some("colour") => folio_model::Paper::Colour,
            _ => folio_model::Paper::White,
        }
    }

    /// The book as it is drawn or written in RGB: with every swatch reference replaced by the colour
    /// the swatch shows as.
    fn shown_book(&self) -> folio_model::Book {
        let mut book = self.session.book().clone();
        folio_model::resolve_swatches(&mut book);
        book
    }

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
pub struct Books {
    host: Arc<dyn Host>,
    inner: Mutex<Inner>,
    hooks: Mutex<Vec<BookHook>>,
    generator: String,
    /// The bundled fonts and the system's, found the first time a spread is laid out.
    fonts: OnceLock<FontBook>,
    /// The dictionaries: the one that comes with Folio, and those downloaded.
    pub spelling: Spelling,
    /// Copies of books with changes, for coming back after a crash.
    pub recovery: Recovery,
    /// Set to stop the export in progress before its next file.
    cancel_export: AtomicBool,
}

impl Books {
    pub fn new(host: Arc<dyn Host>) -> Self {
        let generator = format!("Folio {} (Liminal HQ)", host.version());
        Self {
            host,
            inner: Mutex::new(Inner::default()),
            hooks: Mutex::new(Vec::new()),
            generator,
            fonts: OnceLock::new(),
            spelling: Spelling::default(),
            recovery: Recovery::default(),
            cancel_export: AtomicBool::new(false),
        }
    }

    /// Brings the words of the Markdown or plain text file at `path` in as the story of the book's
    /// first flow, in one undo step. A book with no flow has no story to put them in.
    pub fn import_text(&self, id: &str, path: &str) -> Result<folio_import::ImportReport, Error> {
        self.import_text_with(id, path, &folio_import::StyleMap::new())
    }

    /// As `import_text`, with the person's choice of Folio style for each style of a Word or
    /// OpenDocument file.
    pub fn import_text_with(
        &self,
        id: &str,
        path: &str,
        mapping: &folio_import::StyleMap,
    ) -> Result<folio_import::ImportReport, Error> {
        /// More than this is not a manuscript someone typed; it would also stall the page.
        const MAX_BYTES: u64 = 8 * 1024 * 1024;
        let file = Path::new(path);
        let known = file
            .extension()
            .and_then(|e| e.to_str())
            .map(str::to_ascii_lowercase);
        if !matches!(
            known.as_deref(),
            Some("md" | "markdown" | "txt" | "docx" | "odt" | "rtf")
        ) {
            return Err(Error::Import(
                "Folio can bring in Word (.docx), OpenDocument (.odt), Rich Text (.rtf), Markdown \
                 (.md) and plain text (.txt) files so far. Other formats are not supported yet."
                    .into(),
            ));
        }
        let size = std::fs::metadata(file)
            .map_err(|e| Error::Import(format!("{path}: {e}")))?
            .len();
        if size > MAX_BYTES {
            return Err(Error::Import(
                "That file is too large to bring in (the limit is 8 MB).".into(),
            ));
        }
        let imported = if matches!(known.as_deref(), Some("docx" | "odt" | "rtf")) {
            let bytes = std::fs::read(file).map_err(|e| Error::Import(format!("{path}: {e}")))?;
            if known.as_deref() == Some("docx") {
                folio_import::import_docx_with(&bytes, mapping)
                    .map_err(|e| Error::Import(e.to_string()))?
            } else if known.as_deref() == Some("odt") {
                folio_import::import_odt_with(&bytes, mapping)
                    .map_err(|e| Error::Import(e.to_string()))?
            } else {
                folio_import::import_rtf(&bytes).map_err(|e| Error::Import(e.to_string()))?
            }
        } else {
            let text = std::fs::read_to_string(file).map_err(|e| {
                Error::Import(format!(
                    "{path} could not be read as text ({e}). Save it as UTF-8 and try again."
                ))
            })?;
            folio_import::import_markdown(&text)
        };
        if imported.story.is_empty() {
            return Err(Error::Import("That file has no words to bring in.".into()));
        }
        let (flow, count) = {
            let inner = locked(&self.inner);
            let open = inner
                .books
                .iter()
                .find(|b| b.id == id)
                .ok_or_else(|| Error::NoSuchBook(id.into()))?;
            let flow = open
                .session
                .book()
                .flows
                .first()
                .map(|f| (f.id.clone(), f.paragraphs.len()))
                .ok_or_else(|| {
                    Error::Import(
                        "This book's pages hold their own words, so there is no story to bring \
                         them into. Use a flowing book such as a novel, or chain its text frames \
                         first."
                            .into(),
                    )
                })?;
            (flow.0, flow.1)
        };
        self.apply(
            id,
            Intent::SpliceStory {
                flow,
                at: 0,
                remove: count,
                insert: imported.story,
            },
        )?;
        Ok(imported.report)
    }

    /// Stops the export in progress after the file it is making.
    pub fn cancel_export(&self) {
        self.cancel_export.store(true, Ordering::SeqCst);
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
        self.host.books_changed(&BooksChanged { books });
    }

    fn announce_patch(&self, book: &OpenBook, patch: &Patch, by: Option<&str>) {
        let event = PatchEvent {
            id: book.id.clone(),
            patch: patch.clone(),
            can_undo: book.session.can_undo(),
            can_redo: book.session.can_redo(),
            by: by.map(str::to_owned),
        };
        self.host.patched(&event);
    }

    /// The cover of the book `spec` would make, as a `data:image/png` URI.
    pub fn preview_new_book(&self, spec: &NewBookSpec) -> Result<String, Error> {
        use base64::Engine as _;
        let made = new_book(spec)?;
        let styles = style_sheet(&made.book.styles);
        let fonts = FontBook::bundled();
        let cover = made
            .book
            .spreads
            .first()
            .ok_or_else(|| Error::Export("the book has no cover".into()))?;
        let scene = Layouter::new(&made.book, &styles, &fonts)
            .scene_spread(&cover.id)
            .map_err(|e| Error::Export(e.to_string()))?
            .and_then(|s| s.pages.into_iter().next())
            .ok_or_else(|| Error::Export("the cover has no page".into()))?;
        let png =
            folio_export::raster::page_to_png(&scene, &fonts, &MemoryAssets(HashMap::new()), 360)
                .map_err(|e| Error::Export(e.to_string()))?;
        Ok(format!(
            "data:image/png;base64,{}",
            base64::engine::general_purpose::STANDARD.encode(png)
        ))
    }

    pub fn presets(&self) -> Vec<KindInfo> {
        kind_infos()
    }

    /// Where Save as… starts for a book called `title`: `<Books>/<Title>/<Title>.folio`, with
    /// `Books` in the home folder (in Documents on Windows).
    pub fn default_save_path(&self, title: &str) -> PathBuf {
        let base = if cfg!(windows) {
            self.host.document_dir()
        } else {
            self.host.home_dir()
        };
        let books = base.unwrap_or_default().join("Books");
        suggest_save_path(title, &books)
    }

    fn add(&self, session: Session, package: Package, path: Option<PathBuf>) -> BookSnapshot {
        self.add_recovering(session, package, path, None, None)
    }

    /// Adds a book. `recovery_id` is the copy it was brought back from, which it goes on using.
    fn add_recovering(
        &self,
        session: Session,
        package: Package,
        path: Option<PathBuf>,
        recovery_id: Option<String>,
        lock: Option<BookLock>,
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
            flow_cache: None,
            lock,
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
        let lock = BookLock::take(&canonical)?;
        let package = Package::open(&canonical)?;
        let session = Session::new(package.book().clone());
        let snapshot = self.add_recovering(session, package, Some(canonical), None, lock);
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
            // Saving somewhere new takes that file's lock, and lets go of the old one's.
            let canonical = dunce::canonicalize(&target).unwrap_or_else(|_| target.clone());
            let moved = open.path.as_deref() != Some(canonical.as_path());
            let taken = if moved || open.lock.is_none() {
                BookLock::take(&canonical)?
            } else {
                None
            };
            *open.package.book_mut() = open.session.book().clone();
            let options = SaveOptions {
                generator: self.generator.clone(),
                ..SaveOptions::default()
            };
            open.package.save(&target, &options)?;
            open.session.mark_saved();
            if taken.is_some() {
                open.lock = taken;
            }
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

    /// Lets go of every book's lock without closing the books, as the system does when the program
    /// dies. Tests of recovery use it to stand in for a crash.
    #[doc(hidden)]
    pub fn release_locks(&self) {
        for open in locked(&self.inner).books.iter_mut() {
            open.lock = None;
        }
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
        let lock = match &path {
            Some(path) => BookLock::take(path)?,
            None => None,
        };
        let snapshot = self.add_recovering(session, package, path, Some(note.id), lock);
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
        let (book, styles, revision, facts, layered, flows) = {
            let mut inner = locked(&self.inner);
            let open = inner
                .books
                .iter_mut()
                .find(|b| b.id == id)
                .ok_or_else(|| Error::NoSuchBook(id.into()))?;
            let book = open.shown_book();
            let facts = open.facts_for(&book, spread);
            let layered = open.font_book(base);
            (
                book,
                style_sheet(&open.session.book().styles),
                open.session.revision(),
                facts,
                layered,
                open.flows_laid(),
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
        // Flows laid out for one set of misspelt words are not shared with another.
        let variant = {
            use std::hash::{Hash, Hasher};
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            speller.is_some().hash(&mut hasher);
            ignore.hash(&mut hasher);
            hasher.finish()
        };
        let mut layouter = Layouter::new(&book, &styles, fonts)
            .with_assets(&facts)
            .with_flow_cache(&flows, variant);
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
                open.shown_book(),
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
                                look(&spread.id.0, &t.base.id.0, None, &flow.text(), false);
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
        let ignore = self.book_ignore(id)?;
        let mut items = Vec::new();
        for spread in spreads {
            items.extend(self.layout_spread(id, &spread, None)?.layout.preflight);
        }
        for item in &mut items {
            item.key = item.compute_key();
            item.ignored = ignore.contains(&item.key);
        }
        // A stable sort keeps the pages in order within each severity.
        items.sort_by_key(|item| item.severity);
        Ok(items)
    }

    /// The chapters of the book's stories (paragraphs in the heading style) and the spread each
    /// begins on, in the order of the stories.
    pub fn story_chapters(&self, id: &str) -> Result<Vec<StoryChapter>, Error> {
        let book = {
            let inner = locked(&self.inner);
            let open = inner
                .books
                .iter()
                .find(|b| b.id == id)
                .ok_or_else(|| Error::NoSuchBook(id.into()))?;
            open.session.book().clone()
        };
        let mut out = Vec::new();
        for flow in &book.flows {
            let chapters: Vec<(usize, String)> = flow
                .paragraphs
                .iter()
                .enumerate()
                .filter(|(_, p)| p.style.as_ref().is_some_and(|s| s.0 == "h1"))
                .map(|(i, p)| {
                    (
                        i,
                        p.inline.iter().map(|r| r.text.as_str()).collect::<String>(),
                    )
                })
                .filter(|(_, title)| !title.trim().is_empty())
                .collect();
            if chapters.is_empty() {
                continue;
            }
            // The first spread each of the story's paragraphs is seen on.
            let mut first_seen: HashMap<u32, String> = HashMap::new();
            for spread in &book.spreads {
                let laid = self.layout_spread(id, &spread.id.0, None)?;
                for frame in &laid.layout.frames {
                    if !flow.frames.iter().any(|f| f.0 == frame.frame.0) {
                        continue;
                    }
                    for line in &frame.lines {
                        if let Some(p) = line.paragraph {
                            first_seen.entry(p).or_insert_with(|| spread.id.0.clone());
                        }
                    }
                }
            }
            for (index, title) in chapters {
                if let Some(spread) = first_seen.get(&(index as u32)) {
                    out.push(StoryChapter {
                        title: title.trim().to_owned(),
                        spread: spread.clone(),
                    });
                }
            }
        }
        Ok(out)
    }

    /// A PNG of one page of a spread, `width` pixels wide, as it is drawn: the picture an agent
    /// looks at to check its work. `page` counts from 0 within the spread.
    pub fn render_page(
        &self,
        id: &str,
        spread: &str,
        page: usize,
        width: u32,
    ) -> Result<Vec<u8>, Error> {
        let base = self.fonts.get_or_init(FontBook::with_system_fonts);
        let (book, styles, assets, layered) = {
            let mut inner = locked(&self.inner);
            let open = inner
                .books
                .iter_mut()
                .find(|b| b.id == id)
                .ok_or_else(|| Error::NoSuchBook(id.into()))?;
            let book = open.shown_book();
            let mut assets: HashMap<String, Vec<u8>> = HashMap::new();
            for path in used_assets(&book) {
                if let Ok(bytes) = open.package.asset_bytes(&path) {
                    assets.insert(path, bytes);
                }
            }
            let styles = style_sheet(&book.styles);
            (book, styles, assets, open.font_book(base))
        };
        let fonts = layered.as_deref().unwrap_or(base);
        let scene = Layouter::new(&book, &styles, fonts)
            .scene_spread(&SpreadId(spread.to_owned()))
            .map_err(|e| Error::Export(e.to_string()))?
            .ok_or_else(|| Error::Export(format!("no spread {spread}")))?;
        let pages = scene.pages.len();
        let page = scene.pages.into_iter().nth(page).ok_or_else(|| {
            Error::Export(format!(
                "spread {spread} has {pages} pages, not {}",
                page + 1
            ))
        })?;
        folio_export::raster::page_to_png(
            &page,
            fonts,
            &MemoryAssets(assets),
            width.clamp(64, 4000),
        )
        .map_err(|e| Error::Export(e.to_string()))
    }

    /// A spread as a press would print it, through the CMYK ICC profile at `profile`.
    pub fn proof_spread(
        &self,
        id: &str,
        spread: &str,
        profile: &str,
    ) -> Result<ProofSpread, Error> {
        use base64::Engine as _;
        let press = std::fs::read(profile).map_err(|e| Error::Export(format!("{profile}: {e}")))?;
        let base = self.fonts.get_or_init(FontBook::with_system_fonts);
        let (book, styles, assets, layered) = {
            let mut inner = locked(&self.inner);
            let open = inner
                .books
                .iter_mut()
                .find(|b| b.id == id)
                .ok_or_else(|| Error::NoSuchBook(id.into()))?;
            let book = open.shown_book();
            let mut assets: HashMap<String, Vec<u8>> = HashMap::new();
            for path in used_assets(&book) {
                if let Ok(bytes) = open.package.asset_bytes(&path) {
                    assets.insert(path, bytes);
                }
            }
            let styles = style_sheet(&book.styles);
            (book, styles, assets, open.font_book(base))
        };
        let fonts = layered.as_deref().unwrap_or(base);
        let pages = folio_export::proof::proof_spread(
            &book,
            &styles,
            fonts,
            &MemoryAssets(assets),
            &SpreadId(spread.to_owned()),
            &press,
            720,
        )
        .map_err(|e| Error::Export(format!("{profile}: {e}")))?
        .ok_or_else(|| Error::Export(format!("no spread {spread}")))?;
        let colours = folio_export::proof::proof_colours(&book, &press)
            .map_err(|e| Error::Export(format!("{profile}: {e}")))?
            .into_iter()
            .map(|c| ProofColour {
                name: c.name,
                display: c.display,
                proofed: c.proofed,
                outside: c.outside,
            })
            .collect();
        let uri = |png: Vec<u8>| {
            format!(
                "data:image/png;base64,{}",
                base64::engine::general_purpose::STANDARD.encode(png)
            )
        };
        Ok(ProofSpread {
            pages: pages
                .into_iter()
                .map(|p| ProofPage {
                    original: uri(p.original),
                    proofed: uri(p.proofed),
                    outside: p.outside,
                })
                .collect(),
            colours,
        })
    }

    /// The paperback cover sheet as the exporter would make it (back, spine and front), as a
    /// `data:image/png` URI. `None` when the book has no cover.
    pub fn preview_cover_sheet(&self, id: &str) -> Result<Option<CoverSheetPreview>, Error> {
        use base64::Engine as _;
        let base = self.fonts.get_or_init(FontBook::with_system_fonts);
        let (book, assets, paper, layered) = {
            let mut inner = locked(&self.inner);
            let open = inner
                .books
                .iter_mut()
                .find(|b| b.id == id)
                .ok_or_else(|| Error::NoSuchBook(id.into()))?;
            let book = open.session.book().clone();
            let mut assets: HashMap<String, Vec<u8>> = HashMap::new();
            for path in used_assets(&book) {
                if let Ok(bytes) = open.package.asset_bytes(&path) {
                    assets.insert(path, bytes);
                }
            }
            (book, assets, open.paper_in_use(), open.font_book(base))
        };
        let fonts = layered.as_deref().unwrap_or(base);
        let Some(sheet) = folio_export::cover::cover_sheet(&book, paper) else {
            return Ok(None);
        };
        let styles = style_sheet(&sheet.styles);
        let scene = Layouter::new(&sheet, &styles, fonts)
            .scene_spread(&sheet.spreads[0].id)
            .map_err(|e| Error::Export(e.to_string()))?
            .and_then(|s| s.pages.into_iter().next())
            .ok_or_else(|| Error::Export("the cover sheet has no page".into()))?;
        let png = folio_export::raster::page_to_png(&scene, fonts, &MemoryAssets(assets), 1400)
            .map_err(|e| Error::Export(e.to_string()))?;
        let maths = folio_model::paperback_cover(book.trim, folio_model::spine_pages(&book), paper);
        Ok(Some(CoverSheetPreview {
            image: format!(
                "data:image/png;base64,{}",
                base64::engine::general_purpose::STANDARD.encode(png)
            ),
            width: maths.width.0,
            height: maths.height.0,
            edge: maths.edge.0,
            cover: book.trim.w.0,
            spine: maths.spine.0,
            spine_text: maths.spine_text_allowed,
            barcode: maths.barcode,
        }))
    }

    /// The cover sheet as a one-spread book with its layout, for the canvas to draw and edit.
    /// `None` when the book has no cover.
    pub fn cover_sheet_canvas(&self, id: &str) -> Result<Option<CoverCanvas>, Error> {
        let base = self.fonts.get_or_init(FontBook::with_system_fonts);
        let (book, paper, layered) = {
            let mut inner = locked(&self.inner);
            let open = inner
                .books
                .iter_mut()
                .find(|b| b.id == id)
                .ok_or_else(|| Error::NoSuchBook(id.into()))?;
            (
                open.session.book().clone(),
                open.paper_in_use(),
                open.font_book(base),
            )
        };
        let fonts = layered.as_deref().unwrap_or(base);
        let Some(sheet) = folio_export::cover::cover_sheet(&book, paper) else {
            return Ok(None);
        };
        let styles = style_sheet(&sheet.styles);
        let layout = Layouter::new(&sheet, &styles, fonts)
            .layout_spread(&sheet.spreads[0].id)
            .map_err(|e| Error::Export(e.to_string()))?
            .ok_or_else(|| Error::Export("the cover sheet has no page".into()))?;
        let maths = folio_model::paperback_cover(book.trim, folio_model::spine_pages(&book), paper);
        Ok(Some(CoverCanvas {
            book: sheet,
            layout,
            cover: book.trim.w.0,
            spine: maths.spine.0,
        }))
    }

    /// The book as an e-reader would show it, from the same chapters and CSS the EPUB ships.
    pub fn reader_preview(&self, id: &str) -> Result<ReaderPreview, Error> {
        use base64::Engine as _;
        let (book, assets) = {
            let inner = locked(&self.inner);
            let open = inner
                .books
                .iter()
                .find(|b| b.id == id)
                .ok_or_else(|| Error::NoSuchBook(id.into()))?;
            let book = open.shown_book();
            let mut assets: HashMap<String, Vec<u8>> = HashMap::new();
            for path in used_assets(&book) {
                if let Ok(bytes) = open.package.asset_bytes(&path) {
                    assets.insert(path, bytes);
                }
            }
            (book, assets)
        };
        let made = folio_export::epub::reader_preview(&book, &MemoryAssets(assets))
            .map_err(|e| Error::Export(e.to_string()))?;
        Ok(ReaderPreview {
            title: made.title,
            language: made.language,
            css: made.css,
            chapters: made
                .chapters
                .into_iter()
                .map(|c| PreviewChapter {
                    title: c.title,
                    body: c.body,
                })
                .collect(),
            images: made
                .images
                .into_iter()
                .map(|(name, media, bytes)| PreviewImage {
                    data_uri: format!(
                        "data:{media};base64,{}",
                        base64::engine::general_purpose::STANDARD.encode(bytes)
                    ),
                    name,
                })
                .collect(),
        })
    }

    /// Makes the files for `targets` from a book and writes them into its `exports` folder: beside
    /// the book's file when it has been saved, and in its folder under `Books` when it has not.
    /// Each file is written whole under a temporary name and renamed, so a failed export leaves no
    /// half-written file.
    pub fn export(
        &self,
        id: &str,
        targets: &[ExportTarget],
        colour: ExportColour,
    ) -> Result<ExportReport, Error> {
        // The press's profile is read once; a file that is not a CMYK profile is refused up front.
        let ink = match (&colour.profile, colour.cmyk) {
            (Some(path), _) => {
                let bytes =
                    std::fs::read(path).map_err(|e| Error::Export(format!("{path}: {e}")))?;
                folio_colour::Profile::from_icc(&bytes)
                    .map_err(|e| Error::Export(format!("{path}: {e}")))?;
                Some(folio_export::pdf::InkOptions {
                    profile: Some(bytes),
                })
            }
            (None, true) => Some(folio_export::pdf::InkOptions::default()),
            (None, false) => None,
        };
        let base = self.fonts.get_or_init(FontBook::with_system_fonts);
        let (book, styles, assets, layered, folder, paper) = {
            let mut inner = locked(&self.inner);
            let open = inner
                .books
                .iter_mut()
                .find(|b| b.id == id)
                .ok_or_else(|| Error::NoSuchBook(id.into()))?;
            let book = open.session.book().clone();
            // Pictures are read under the lock and used outside it, so the book stays usable.
            let mut assets: HashMap<String, Vec<u8>> = HashMap::new();
            for path in used_assets(&book) {
                if let Ok(bytes) = open.package.asset_bytes(&path) {
                    assets.insert(path, bytes);
                }
            }
            let paper = open.paper_in_use();
            let folder = match &open.path {
                Some(path) => path.parent().map(Path::to_path_buf).unwrap_or_default(),
                None => self
                    .default_save_path(&book.title)
                    .parent()
                    .map(Path::to_path_buf)
                    .unwrap_or_default(),
            }
            .join("exports");
            let styles = style_sheet(&book.styles);
            let layered = open.font_book(base);
            (book, styles, assets, layered, folder, paper)
        };
        let fonts = layered.as_deref().unwrap_or(base);
        std::fs::create_dir_all(&folder)
            .map_err(|e| Error::Export(format!("{}: {e}", folder.display())))?;
        let source = MemoryAssets(assets);
        let mut report = ExportReport {
            folder: folder.to_string_lossy().into_owned(),
            files: Vec::new(),
            cancelled: false,
        };
        self.cancel_export.store(false, Ordering::SeqCst);
        for (done, &target) in targets.iter().enumerate() {
            let progress = ExportProgress {
                id: id.to_owned(),
                target,
                done: done as u32,
                total: targets.len() as u32,
            };
            self.host.export_progress(&progress);
            if self.cancel_export.swap(false, Ordering::SeqCst) {
                report.cancelled = true;
                break;
            }
            let made = folio_export::export_target(
                match target {
                    ExportTarget::SharingPdf => folio_export::Target::SharingPdf,
                    ExportTarget::KdpInterior => folio_export::Target::KdpInterior,
                    ExportTarget::PrintShopPdf => folio_export::Target::PrintShopPdf,
                    ExportTarget::KdpCover => folio_export::Target::KdpCover,
                    ExportTarget::EbookCover => folio_export::Target::EbookCover,
                    ExportTarget::KindleEpub => folio_export::Target::KindleEpub,
                    ExportTarget::FixedEpub => folio_export::Target::FixedEpub,
                    ExportTarget::KoboEpub => folio_export::Target::KoboEpub,
                },
                paper,
                &book,
                &styles,
                fonts,
                &source,
                ink.as_ref(),
            )
            .map_err(|e| Error::Export(e.to_string()))?;
            for file in made {
                let path = folder.join(&file.file_name);
                let partial = folder.join(format!(".{}.partial", file.file_name));
                std::fs::write(&partial, &file.bytes)
                    .and_then(|()| std::fs::rename(&partial, &path))
                    .map_err(|e| Error::Export(format!("{}: {e}", path.display())))?;
                report.files.push(ExportedFile {
                    target,
                    path: path.to_string_lossy().into_owned(),
                    bytes: file.bytes.len().try_into().unwrap_or(u32::MAX),
                });
            }
        }
        Ok(report)
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

    /// The typefaces the book uses and where each comes from, for the Fonts panel.
    pub fn fonts_in_use(&self, id: &str) -> Result<Vec<FontUse>, Error> {
        let base = self.fonts.get_or_init(FontBook::with_system_fonts);
        let (book, layered) = {
            let mut inner = locked(&self.inner);
            let open = inner
                .books
                .iter_mut()
                .find(|b| b.id == id)
                .ok_or_else(|| Error::NoSuchBook(id.into()))?;
            (open.session.book().clone(), open.font_book(base))
        };
        let fonts = layered.as_deref().unwrap_or(base);
        let choices = self.font_choices(id)?;
        Ok(folio_layout::families::families_used(&book)
            .into_iter()
            .map(|family| {
                let found = fonts.resolve(&folio_layout::FontRequest::new(&family, 400, false));
                let (missing, licence) = match &found {
                    Ok(r) => {
                        let rights = r.font.embedding_rights();
                        let licence = if rights.may_keep_with_book() {
                            FontLicence::Free
                        } else if rights.may_embed_in_pdf() {
                            FontLicence::PrintOnly
                        } else {
                            FontLicence::Restricted
                        };
                        (r.family_substituted, licence)
                    }
                    Err(_) => (true, FontLicence::Restricted),
                };
                let origin = choices
                    .iter()
                    .find(|c| c.family.eq_ignore_ascii_case(&family))
                    .map_or(FontOrigin::System, |c| c.origin);
                FontUse {
                    family,
                    origin,
                    missing,
                    licence,
                }
            })
            .collect())
    }

    /// Takes the fonts of `families` out of the book, so it is laid out from the computer's again,
    /// in one undo step. The files leave the package at the next save.
    pub fn unkeep_fonts(&self, id: &str, families: &[String]) -> Result<(), Error> {
        let kept = self.book_fonts(id)?;
        let left: Vec<BookFont> = kept
            .iter()
            .filter(|f| !families.iter().any(|x| x.eq_ignore_ascii_case(&f.family)))
            .cloned()
            .collect();
        if left.len() == kept.len() {
            return Ok(());
        }
        let gone: Vec<String> = kept
            .iter()
            .filter(|f| !left.contains(f))
            .map(|f| f.path.clone())
            .collect();
        self.apply(
            id,
            Intent::SetFonts {
                fonts: left.clone(),
            },
        )?;
        // A file another kept font still names stays; the rest are not wanted.
        let mut inner = locked(&self.inner);
        if let Some(open) = inner.books.iter_mut().find(|b| b.id == id) {
            for path in gone {
                if !left.iter().any(|f| f.path == path) {
                    open.package.remove_asset(&path);
                }
            }
        }
        Ok(())
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

    fn book_ignore(&self, id: &str) -> Result<Vec<String>, Error> {
        let inner = locked(&self.inner);
        inner
            .books
            .iter()
            .find(|b| b.id == id)
            .map(|open| open.session.book().preflight_ignore.clone())
            .ok_or_else(|| Error::NoSuchBook(id.into()))
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
        self.apply_by(id, intent, None)
    }

    /// As `apply`, saying who is making the change when it is not the person (`agent`).
    fn apply_by(&self, id: &str, intent: Intent, by: Option<&str>) -> Result<Patch, Error> {
        // A change to a story may leave it with more words than its frames have room for.
        let refit = changes_a_story(&intent);
        let edits_words = edits_a_story(&intent);
        let base = self.fonts.get_or_init(FontBook::with_system_fonts);
        let layered = if refit {
            let mut inner = locked(&self.inner);
            inner
                .books
                .iter_mut()
                .find(|b| b.id == id)
                .and_then(|open| open.font_book(base))
        } else {
            None
        };
        self.change(
            id,
            by,
            |session| {
                let depth = session.undo_depth();
                let mut patch = session.apply(intent)?;
                if !refit {
                    return Ok(patch);
                }
                let user_step = session.undo_depth() > depth;
                let after_user = session.undo_depth();
                let styles = style_sheet(&session.book().styles);
                let fonts = layered.as_deref().unwrap_or(base);
                let flows: Vec<folio_model::FlowId> = session
                    .book()
                    .flows
                    .iter()
                    .filter(|f| f.auto_add_pages)
                    .map(|f| f.id.clone())
                    .collect();
                for flow in flows {
                    let needed = Layouter::new(session.book(), &styles, fonts)
                        .frames_needed(&flow)
                        .map_err(Error::from)?;
                    if needed > 0 {
                        let more = session.apply(Intent::AddFlowPages {
                            flow,
                            count: needed,
                        })?;
                        patch.ops.extend(more.ops);
                        patch.revision = more.revision;
                        continue;
                    }
                    // A story whose words were edited may have shrunk, leaving frames, and so pages,
                    // with nothing in them. Chaining a frame or restyling does not give pages back.
                    if !edits_words {
                        continue;
                    }
                    let layouter = Layouter::new(session.book(), &styles, fonts);
                    let used = layouter.frames_used(&flow).map_err(Error::from)?;
                    let have = session
                        .book()
                        .flows
                        .iter()
                        .find(|f| f.id == flow)
                        .map_or(0, |f| f.frames.len());
                    if let Some(used) = used.filter(|u| *u < have) {
                        let less = session.apply(Intent::TrimFlowPages { flow, keep: used })?;
                        patch.ops.extend(less.ops);
                        patch.revision = less.revision;
                    }
                }
                // The pages added for the change are taken back with it.
                let added = session.undo_depth() - after_user;
                if user_step && added > 0 {
                    session.join_last(added + 1);
                }
                Ok(patch)
            },
            |patch: &Patch| Some(patch),
        )
    }

    /// Applies several intents as one undo step, by an agent: all of them or none. The step is
    /// closed on both sides, so nothing the person was doing merges into it, and a refused intent
    /// takes back what came before it in the batch.
    pub fn apply_batch(&self, id: &str, intents: Vec<Intent>) -> Result<BatchOutcome, Error> {
        const BY: Option<&str> = Some("agent");
        let total = intents.len();
        let mark = self.history_mark(id)?;
        self.apply_by(id, Intent::EndUndoStep, BY)?;
        for (index, intent) in intents.into_iter().enumerate() {
            if let Err(e) = self.apply_by(id, intent, BY) {
                self.rewind(id, mark)?;
                self.change(
                    id,
                    BY,
                    |session| {
                        session.clear_redo();
                        Ok(())
                    },
                    |_: &()| None,
                )?;
                return Err(Error::Batch {
                    index,
                    total,
                    message: e.to_string(),
                });
            }
        }
        self.change(
            id,
            BY,
            |session| {
                // The batch is one step, so one undo takes all of it back.
                let steps = session.undo_depth().saturating_sub(mark);
                session.join_last(steps);
                session.apply(Intent::EndUndoStep)?;
                Ok(BatchOutcome {
                    revision: session.revision(),
                    applied: total,
                    can_undo: session.can_undo(),
                    can_redo: session.can_redo(),
                })
            },
            |_: &BatchOutcome| None,
        )
    }

    /// Undoes the last step, or `None` when there is nothing to undo.
    pub fn undo(&self, id: &str) -> Result<Option<Patch>, Error> {
        self.change(
            id,
            None,
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
            None,
            |session| Ok(session.redo()),
            |patch: &Option<Patch>| patch.as_ref(),
        )
    }

    /// Runs a change on a book's session and tells every window what happened: the patch when
    /// there is one, and the list of books when this one became changed or clean.
    fn change<T>(
        &self,
        id: &str,
        by: Option<&str>,
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
            self.announce_patch(open, patch, by);
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

/// Whether an intent can change how much room a story needs: its words, their styles, or the frames
/// they run through.
fn changes_a_story(intent: &Intent) -> bool {
    matches!(
        intent,
        Intent::SetParagraphText { .. }
            | Intent::SpliceStory { .. }
            | Intent::SplitParagraph { .. }
            | Intent::JoinParagraphs { .. }
            | Intent::StyleParagraphs { .. }
            | Intent::StyleStoryRange { .. }
            | Intent::LinkFrame { .. }
            | Intent::UnlinkFrame { .. }
            | Intent::SetStyle { .. }
            | Intent::OverrideStyle { .. }
    )
}

/// Whether an intent edits the words of a story, which can make it shorter.
fn edits_a_story(intent: &Intent) -> bool {
    matches!(
        intent,
        Intent::SetParagraphText { .. }
            | Intent::SpliceStory { .. }
            | Intent::SplitParagraph { .. }
            | Intent::JoinParagraphs { .. }
    )
}
