// A book made, changed, saved, reopened and exported with no window or toolkit in sight
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use folio_engine::{
    Books, BooksChanged, Error, ExportColour, ExportProgress, ExportTarget, Host, PatchEvent,
};
use folio_model::{Intent, Pt, Rect, SpreadId, Surface};
use folio_presets::NewBookSpec;

/// A host with no one to tell: it remembers what it was told, and has no folders.
#[derive(Default)]
struct Recorder {
    patches: Mutex<Vec<PatchEvent>>,
    changes: Mutex<usize>,
    progress: Mutex<Vec<ExportProgress>>,
}

impl Host for Recorder {
    fn books_changed(&self, _: &BooksChanged) {
        *self.changes.lock().unwrap() += 1;
    }
    fn patched(&self, event: &PatchEvent) {
        self.patches.lock().unwrap().push(event.clone());
    }
    fn export_progress(&self, progress: &ExportProgress) {
        self.progress.lock().unwrap().push(progress.clone());
    }
    fn home_dir(&self) -> Option<PathBuf> {
        None
    }
    fn document_dir(&self) -> Option<PathBuf> {
        None
    }
    fn version(&self) -> String {
        "test".into()
    }
}

#[test]
fn a_book_is_made_changed_saved_reopened_and_exported_without_a_toolkit() {
    let host = Arc::new(Recorder::default());
    let books = Books::new(host.clone());
    let made = books.new_book(&NewBookSpec::default()).unwrap();
    let id = made.summary.id.clone();

    // A change goes in as an intent, comes out as a patch, and the host hears of it.
    let cover = &made.book.spreads[0];
    let patch = books
        .apply(
            &id,
            Intent::MoveFrame {
                on: Surface::Spread(SpreadId(cover.id.0.clone())),
                frame: cover.frames[0].base().id.clone(),
                rect: Rect {
                    x: Pt(12.0),
                    y: Pt(0.0),
                    w: Pt(100.0),
                    h: Pt(100.0),
                },
            },
        )
        .unwrap();
    assert!(patch.revision > made.revision);
    let heard = host.patches.lock().unwrap();
    assert_eq!(heard.len(), 1);
    assert_eq!(heard[0].id, id);
    assert!(heard[0].can_undo);
    drop(heard);

    // It is saved. While it is open here, a second set of books (another program) cannot open
    // the same file; once it is closed, it can.
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("Otter").join("Otter.folio");
    books.save(&id, Some(file.as_path())).unwrap();
    let again = Books::new(Arc::new(Recorder::default()));
    assert!(matches!(again.open(&file), Err(Error::Locked(_))));

    // It is exported, and the host hears which file the export reached.
    let report = books
        .export(&id, &[ExportTarget::SharingPdf], ExportColour::default())
        .unwrap();
    assert_eq!(report.files.len(), 1);
    let pdf = std::fs::read(&report.files[0].path).unwrap();
    assert!(pdf.starts_with(b"%PDF-"));
    assert!(!host.progress.lock().unwrap().is_empty());
    assert!(*host.changes.lock().unwrap() > 0);

    // Closed, the book can be opened by the other program, with the change in it.
    books.close(&id).unwrap();
    let reopened = again.open(&file).unwrap();
    let moved = &reopened.book.spreads[0].frames[0].base().rect;
    assert_eq!(moved.x, Pt(12.0));
}

fn nudge(books: &Books, id: &str, x: f64) -> Intent {
    let cover = &books.snapshot(id).unwrap().book.spreads[0];
    Intent::MoveFrame {
        on: Surface::Spread(SpreadId(cover.id.0.clone())),
        frame: cover.frames[0].base().id.clone(),
        rect: Rect {
            x: Pt(x),
            y: Pt(0.0),
            w: Pt(100.0),
            h: Pt(100.0),
        },
    }
}

fn first_x(books: &Books, id: &str) -> f64 {
    books.snapshot(id).unwrap().book.spreads[0].frames[0]
        .base()
        .rect
        .x
        .0
}

#[test]
fn an_agents_batch_is_one_undo_step_and_tells_the_window_who_made_it() {
    let host = Arc::new(Recorder::default());
    let books = Books::new(host.clone());
    let id = books.new_book(&NewBookSpec::default()).unwrap().summary.id;
    let original = first_x(&books, &id);

    let outcome = books
        .apply_batch(
            &id,
            vec![nudge(&books, &id, 10.0), nudge(&books, &id, 20.0)],
        )
        .unwrap();
    assert_eq!(outcome.applied, 2);
    assert_eq!(first_x(&books, &id), 20.0);
    assert!(host
        .patches
        .lock()
        .unwrap()
        .iter()
        .all(|p| p.by.as_deref() == Some("agent")));

    // Both changes are one step: a single undo takes them back together.
    books.undo(&id).unwrap();
    assert_eq!(first_x(&books, &id), original);
    assert!(!books.snapshot(&id).unwrap().can_undo);
}

#[test]
fn a_batch_with_a_refused_intent_changes_nothing() {
    let books = Books::new(Arc::new(Recorder::default()));
    let id = books.new_book(&NewBookSpec::default()).unwrap().summary.id;
    let original = first_x(&books, &id);
    let mut bad = nudge(&books, &id, 50.0);
    if let Intent::MoveFrame { frame, .. } = &mut bad {
        frame.0 = "fr_nobody".into();
    }

    let refused = books
        .apply_batch(&id, vec![nudge(&books, &id, 30.0), bad])
        .unwrap_err();
    assert!(matches!(
        refused,
        Error::Batch {
            index: 1,
            total: 2,
            ..
        }
    ));
    // The first intent was taken back, and cannot be brought back by a redo.
    assert_eq!(first_x(&books, &id), original);
    let snapshot = books.snapshot(&id).unwrap();
    assert!(!snapshot.can_undo);
    assert!(!snapshot.can_redo);
}

#[test]
fn the_template_gallery_draws_every_template_and_adding_one_is_one_undo_step() {
    let books = Books::new(Arc::new(Recorder::default()));
    let made = books.new_book(&NewBookSpec::default()).unwrap();
    let id = made.summary.id.clone();
    let gallery = books.template_gallery(&id).unwrap();
    assert_eq!(gallery.home, "Picture book");
    assert!(gallery.templates.len() >= 20);
    assert!(gallery
        .templates
        .iter()
        .all(|t| t.image.starts_with("data:image/png;base64,")));
    // A template goes after the last spread, and one undo takes it away.
    let spreads = made.book.spreads.len();
    let last = made.book.spreads.last().unwrap().id.0.clone();
    books.apply_template(&id, "pb-split", &last).unwrap();
    let after = books.snapshot(&id).unwrap().book;
    assert_eq!(after.spreads.len(), spreads + 1);
    assert!(!after.spreads.last().unwrap().frames.is_empty());
    books.undo(&id).unwrap().unwrap();
    assert_eq!(books.snapshot(&id).unwrap().book.spreads.len(), spreads);
    // A template that does not exist is refused, and so is one after the cover.
    assert!(books.apply_template(&id, "no-such", &last).is_err());
    assert!(books
        .apply_template(&id, "pb-split", &made.book.spreads[0].id.0)
        .is_err());
}
