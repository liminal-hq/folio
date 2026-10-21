// A book made, changed, saved, reopened and exported with no window or toolkit in sight
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use folio_engine::{
    Books, BooksChanged, ExportColour, ExportProgress, ExportTarget, Host, PatchEvent,
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

    // It is saved, and a second set of books opens the same file.
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("Otter").join("Otter.folio");
    books.save(&id, Some(file.as_path())).unwrap();
    let again = Books::new(Arc::new(Recorder::default()));
    let reopened = again.open(&file).unwrap();
    let moved = &reopened.book.spreads[0].frames[0].base().rect;
    assert_eq!(moved.x, Pt(12.0));

    // It is exported, and the host hears which file the export reached.
    let report = books
        .export(&id, &[ExportTarget::SharingPdf], ExportColour::default())
        .unwrap();
    assert_eq!(report.files.len(), 1);
    let pdf = std::fs::read(&report.files[0].path).unwrap();
    assert!(pdf.starts_with(b"%PDF-"));
    assert!(!host.progress.lock().unwrap().is_empty());
    assert!(*host.changes.lock().unwrap() > 0);
}
