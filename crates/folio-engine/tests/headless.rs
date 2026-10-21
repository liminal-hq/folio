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

#[test]
fn the_punctuation_tool_finds_what_it_would_tidy_and_tidies_it_in_one_undo_step() {
    use folio_engine::{PunctuationRule, ToolFix, ToolReport, ToolRequest};
    let books = Books::new(Arc::new(Recorder::default()));
    let made = books.new_book(&NewBookSpec::default()).unwrap();
    let id = made.summary.id.clone();
    let cover = &made.book.spreads[0];
    let frame = cover
        .frames
        .iter()
        .find(|f| matches!(f, folio_model::Frame::Text(_)))
        .unwrap()
        .base()
        .id
        .clone();
    let words = |books: &Books| {
        let book = books.snapshot(&id).unwrap().book;
        let folio_model::Frame::Text(text) = book.spreads[0]
            .frames
            .iter()
            .find(|f| f.base().id == frame)
            .unwrap()
        else {
            panic!("a text frame")
        };
        folio_model::text_of(&text.content.as_ref().unwrap().inline)
    };
    books
        .apply(
            &id,
            Intent::SetText {
                frame: frame.clone(),
                text: "\"Wait--what...\"  she said".into(),
            },
        )
        .unwrap();
    let ToolReport::Punctuation { found } =
        books.book_tool(&id, &ToolRequest::Punctuation).unwrap()
    else {
        panic!("a punctuation report")
    };
    let count = |rule: PunctuationRule| found.iter().find(|f| f.rule == rule).map(|f| f.count);
    assert_eq!(count(PunctuationRule::Quotes), Some(2));
    assert_eq!(count(PunctuationRule::Dashes), Some(1));
    assert_eq!(count(PunctuationRule::Ellipses), Some(1));
    assert_eq!(count(PunctuationRule::Spaces), Some(1));
    assert!(!found[0].examples.is_empty());
    // Only the rules asked for are applied, and all of it undoes as one step.
    let fixed = books
        .book_tool_fix(
            &id,
            &ToolFix::Punctuation {
                rules: vec![PunctuationRule::Quotes, PunctuationRule::Ellipses],
            },
        )
        .unwrap();
    assert_eq!(fixed.changed, 1);
    assert_eq!(words(&books), "“Wait--what…”  she said");
    books.undo(&id).unwrap().unwrap();
    assert_eq!(words(&books), "\"Wait--what...\"  she said");
    // With nothing to do, nothing is changed and nothing is added to the history.
    let none = books
        .book_tool_fix(&id, &ToolFix::Punctuation { rules: Vec::new() })
        .unwrap();
    assert_eq!(none.changed, 0);
}

#[test]
fn the_styles_tool_finds_boxes_that_differ_from_their_style_and_puts_them_back() {
    use folio_engine::{ToolFix, ToolReport, ToolRequest};
    let books = Books::new(Arc::new(Recorder::default()));
    let made = books.new_book(&NewBookSpec::default()).unwrap();
    let id = made.summary.id.clone();
    let cover = made.book.spreads[0].id.clone();
    let frame = made.book.spreads[0]
        .frames
        .iter()
        .find(|f| matches!(f, folio_model::Frame::Text(_)))
        .unwrap()
        .base()
        .id
        .clone();
    let strays = |books: &Books| {
        let ToolReport::Styles { frames } = books.book_tool(&id, &ToolRequest::Styles).unwrap()
        else {
            panic!("a styles report")
        };
        frames
    };
    let _ = cover;
    assert!(strays(&books).is_empty());
    books
        .apply(
            &id,
            Intent::OverrideStyle {
                frame: frame.clone(),
                overrides: folio_model::StyleOverrides {
                    size: Some(Pt(30.0)),
                    bold: Some(true),
                    ..Default::default()
                },
            },
        )
        .unwrap();
    let found = strays(&books);
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].changes, vec!["size".to_owned(), "bold".to_owned()]);
    let fixed = books
        .book_tool_fix(
            &id,
            &ToolFix::ResetStyles {
                frames: vec![frame.0.clone()],
            },
        )
        .unwrap();
    assert_eq!(fixed.changed, 1);
    assert!(strays(&books).is_empty());
}

#[test]
fn the_copyright_page_is_added_after_the_title_page_and_refreshed_with_the_same_tool() {
    use folio_engine::{CopyrightFields, ToolFix, ToolReport, ToolRequest};
    let books = Books::new(Arc::new(Recorder::default()));
    let made = books.new_book(&NewBookSpec::default()).unwrap();
    let id = made.summary.id.clone();
    let spreads = made.book.spreads.len();
    let fields = CopyrightFields {
        year: 2026,
        holder: "Jane L. Morris".into(),
        rights: "All rights reserved.".into(),
        printing: "First edition".into(),
    };
    let ToolReport::Copyright { text, present } = books
        .book_tool(
            &id,
            &ToolRequest::Copyright {
                fields: fields.clone(),
            },
        )
        .unwrap()
    else {
        panic!("a copyright report")
    };
    assert!(text.starts_with("Copyright © 2026 Jane L. Morris"));
    assert!(!present);
    let done = books
        .book_tool_fix(
            &id,
            &ToolFix::CopyrightPage {
                fields: fields.clone(),
            },
        )
        .unwrap();
    assert_eq!(done.changed, 1);
    let after = books.snapshot(&id).unwrap().book;
    assert_eq!(after.spreads.len(), spreads + 1);
    // The page is straight after the title page, with the frame the tool names.
    assert!(after.spreads[2]
        .frames
        .iter()
        .any(|f| f.base().id.0 == "copyright"));
    let ToolReport::Copyright { present, .. } = books
        .book_tool(
            &id,
            &ToolRequest::Copyright {
                fields: fields.clone(),
            },
        )
        .unwrap()
    else {
        panic!("a copyright report")
    };
    assert!(present);
    // Run again with a new printing: the same page, new words, and no second spread.
    let again = CopyrightFields {
        printing: "Second edition".into(),
        ..fields
    };
    books
        .book_tool_fix(&id, &ToolFix::CopyrightPage { fields: again })
        .unwrap();
    let later = books.snapshot(&id).unwrap().book;
    assert_eq!(later.spreads.len(), spreads + 1);
    books.undo(&id).unwrap().unwrap();
    books.undo(&id).unwrap().unwrap();
    assert_eq!(books.snapshot(&id).unwrap().book.spreads.len(), spreads);
}

#[test]
fn the_contents_lists_each_chapter_with_the_page_it_begins_on() {
    use folio_engine::{ToolFix, ToolReport, ToolRequest};
    let books = Books::new(Arc::new(Recorder::default()));
    let made = books
        .new_book(&NewBookSpec {
            kind: folio_model::BookKind::Novel,
            ..NewBookSpec::default()
        })
        .unwrap();
    let id = made.summary.id.clone();
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("story.md");
    std::fs::write(
        &file,
        "# One\n\nFirst words.\n\n# Two\n\nSecond words.\n\n# Three\n\nThird.\n",
    )
    .unwrap();
    books.import_text(&id, &file.to_string_lossy()).unwrap();
    let ToolReport::Contents { entries, present } =
        books.book_tool(&id, &ToolRequest::Contents).unwrap()
    else {
        panic!("a contents report")
    };
    // Each chapter of the story is listed, with the label of a page.
    let titles: Vec<&str> = entries.iter().map(|e| e.title.as_str()).collect();
    assert_eq!(titles, vec!["One", "Two", "Three"]);
    assert!(entries
        .iter()
        .all(|e| !e.title.is_empty() && !e.page.is_empty()));
    assert!(!present);
    books.book_tool_fix(&id, &ToolFix::ContentsPage).unwrap();
    let after = books.snapshot(&id).unwrap().book;
    let toc = after
        .spreads
        .iter()
        .find(|s| s.frames.iter().any(|f| f.base().id.0 == "toc_titles"))
        .expect("the contents page");
    assert_eq!(toc.frames.len(), 3);
}
