// A book made, changed, saved, reopened and exported with no window or toolkit in sight
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use folio_engine::{
    Books, BooksChanged, Error, ExportColour, ExportProgress, ExportRequest, ExportTarget, Host,
    PatchEvent,
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

#[test]
fn export_settings_are_combined_and_each_target_says_what_it_changed() {
    use folio_model::{BookmarkChoice, ExportPrefs, PdfPrefs, PicturePreset};
    let books = Books::new(Arc::new(Recorder::default()));
    let id = books
        .new_book(&NewBookSpec::default())
        .unwrap()
        .summary
        .id
        .clone();
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("Otter").join("Otter.folio");
    books.save(&id, Some(file.as_path())).unwrap();

    // The person's defaults ask for small pictures everywhere and bookmarks for each page; this
    // export asks for none for the Sharing PDF.
    let mut user = ExportPrefs::default();
    user.pdf.sharing = PdfPrefs {
        preset: Some(PicturePreset::Small),
        bookmarks: Some(BookmarkChoice::Pages),
        ..PdfPrefs::default()
    };
    user.pdf.kdp = PdfPrefs {
        preset: Some(PicturePreset::Small),
        bookmarks: Some(BookmarkChoice::Pages),
        ..PdfPrefs::default()
    };
    let mut once = ExportPrefs::default();
    once.pdf.sharing.bookmarks = Some(BookmarkChoice::None);
    let report = books
        .export_with(
            &id,
            &ExportRequest {
                targets: vec![ExportTarget::SharingPdf, ExportTarget::KdpInterior],
                prefs: Some(once),
                user: Some(user),
                ..ExportRequest::default()
            },
        )
        .unwrap();
    assert_eq!(report.files.len(), 2);
    let outlines = |i: usize| {
        let doc = lopdf::Document::load(&report.files[i].path).unwrap();
        doc.catalog().unwrap().has(b"Outlines")
    };
    assert!(!outlines(0), "this export's choice beats the person's");
    assert!(!outlines(1), "a paperback has no bookmarks");
    assert!(
        report.notes.iter().any(|n| n.contains("Changed for KDP")),
        "{:?}",
        report.notes
    );
}

#[test]
fn a_books_own_export_choices_apply_and_its_folder_is_relative_and_falls_back_when_missing() {
    use folio_model::{BookmarkChoice, ExportPrefs, Intent};
    let books = Books::new(Arc::new(Recorder::default()));
    let id = books
        .new_book(&NewBookSpec::default())
        .unwrap()
        .summary
        .id
        .clone();
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("Otter").join("Otter.folio");
    books.save(&id, Some(file.as_path())).unwrap();
    std::fs::create_dir_all(dir.path().join("Otter").join("for-print")).unwrap();

    // The book chooses a folder beside its file, and bookmarks for each page.
    let mut prefs = ExportPrefs {
        folder: Some("for-print".into()),
        ..ExportPrefs::default()
    };
    prefs.pdf.sharing.bookmarks = Some(BookmarkChoice::Pages);
    books.apply(&id, Intent::SetExportPrefs { prefs }).unwrap();
    let report = books
        .export(&id, &[ExportTarget::SharingPdf], ExportColour::default())
        .unwrap();
    assert!(report.folder.ends_with("for-print"), "{}", report.folder);
    assert!(report.notes.is_empty());
    let doc = lopdf::Document::load(&report.files[0].path).unwrap();
    assert!(
        doc.catalog().unwrap().has(b"Outlines"),
        "the book's choice applies"
    );

    // A folder that is not on this computer falls back to `exports` and says so.
    let moved = ExportPrefs {
        folder: Some("/no/such/place/for/folio".into()),
        ..ExportPrefs::default()
    };
    books
        .apply(&id, Intent::SetExportPrefs { prefs: moved })
        .unwrap();
    let report = books
        .export(&id, &[ExportTarget::SharingPdf], ExportColour::default())
        .unwrap();
    assert!(report.folder.ends_with("exports"), "{}", report.folder);
    assert!(report
        .notes
        .iter()
        .any(|n| n.contains("not on this computer")));
}

#[test]
fn an_estimate_is_the_size_of_the_export_and_leaves_no_files() {
    use folio_model::{ExportPrefs, PdfPrefs, PicturePreset};
    let books = Books::new(Arc::new(Recorder::default()));
    let id = books
        .new_book(&NewBookSpec::default())
        .unwrap()
        .summary
        .id
        .clone();
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("Otter").join("Otter.folio");
    books.save(&id, Some(file.as_path())).unwrap();
    let request = ExportRequest {
        targets: vec![ExportTarget::SharingPdf, ExportTarget::KdpInterior],
        prefs: Some(ExportPrefs {
            pdf: folio_model::PdfPrefsByTarget {
                sharing: PdfPrefs {
                    preset: Some(PicturePreset::Small),
                    ..PdfPrefs::default()
                },
                ..Default::default()
            },
            ..ExportPrefs::default()
        }),
        ..ExportRequest::default()
    };
    let estimate = books.estimate_export(&id, &request).unwrap();
    assert!(!estimate.cancelled);
    assert!(!dir.path().join("Otter").join("exports").exists());
    let report = books.export_with(&id, &request).unwrap();
    let real: u64 = report.files.iter().map(|f| u64::from(f.bytes)).sum();
    let off = estimate.bytes.abs_diff(real) as f64 / real as f64;
    assert!(off < 0.01, "estimate {} against {real}", estimate.bytes);
}

#[test]
fn a_sharing_and_a_print_shop_pdf_name_the_app_and_a_paperback_does_not() {
    let books = Books::new(Arc::new(Recorder::default()));
    let id = books
        .new_book(&NewBookSpec::default())
        .unwrap()
        .summary
        .id
        .clone();
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("Otter").join("Otter.folio");
    books.save(&id, Some(file.as_path())).unwrap();
    let report = books
        .export_with(
            &id,
            &ExportRequest {
                targets: vec![
                    ExportTarget::SharingPdf,
                    ExportTarget::PrintShopPdf,
                    ExportTarget::KdpInterior,
                ],
                ..ExportRequest::default()
            },
        )
        .unwrap();
    let info = |i: usize| {
        let doc = lopdf::Document::load(&report.files[i].path).unwrap();
        let id = doc.trailer.get(b"Info").unwrap().as_reference().unwrap();
        let info = doc.get_object(id).unwrap().as_dict().unwrap().clone();
        let text = |key: &[u8]| {
            info.get(key)
                .ok()
                .and_then(|o| o.as_str().ok())
                .map(|b| String::from_utf8_lossy(b).into_owned())
        };
        (text(b"Creator"), text(b"Producer"))
    };
    for i in [0, 1] {
        assert_eq!(
            info(i),
            (Some("Folio".into()), Some("Folio test (krilla)".into()))
        );
    }
    assert_eq!(info(2), (None, None));
}

/// A book whose first picture frame inside shows a feathered PNG on a pink page.
fn book_with_a_feathered_picture(books: &Books) -> (String, tempfile::TempDir) {
    use folio_model::{Background, BackgroundTarget, Colour, Frame, Intent, SpreadKind};
    use image::{ImageEncoder, Rgba, RgbaImage};
    let id = books
        .new_book(&NewBookSpec::default())
        .unwrap()
        .summary
        .id
        .clone();
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("Otter").join("Otter.folio");
    books.save(&id, Some(file.as_path())).unwrap();

    let disc = RgbaImage::from_fn(1500, 1500, |x, y| {
        let (dx, dy) = (f64::from(x) - 750.0, f64::from(y) - 750.0);
        let a = ((700.0 - (dx * dx + dy * dy).sqrt()) / 120.0).clamp(0.0, 1.0);
        Rgba([200, 30, 30, (a * 255.0) as u8])
    });
    let mut png = Vec::new();
    image::codecs::png::PngEncoder::new(&mut png)
        .write_image(disc.as_raw(), 1500, 1500, image::ExtendedColorType::Rgba8)
        .unwrap();
    let asset = books.import_image_bytes(&id, png, "png").unwrap();

    let book = books.snapshot(&id).unwrap().book;
    let spread = book
        .spreads
        .iter()
        .find(|s| {
            s.kind != SpreadKind::Cover && s.frames.iter().any(|f| matches!(f, Frame::Image(_)))
        })
        .expect("a picture frame inside");
    let frame = spread
        .frames
        .iter()
        .find_map(|f| match f {
            Frame::Image(i) => Some(i.base.id.clone()),
            _ => None,
        })
        .unwrap();
    let intents = [
        Intent::SetImage {
            frame,
            asset: Some(asset),
            fit: None,
            focus: None,
        },
        Intent::SetBackground {
            target: BackgroundTarget::Page {
                spread: spread.id.clone(),
                page: spread.pages[0].id.clone(),
            },
            bg: Some(Background::Colour {
                colour: Colour("#ffc0cb".into()),
            }),
        },
    ];
    for intent in intents {
        books.apply(&id, intent).unwrap();
    }
    (id, dir)
}

fn soft_masks(path: &str) -> usize {
    let doc = lopdf::Document::load(path).unwrap();
    doc.objects
        .values()
        .filter_map(|o| o.as_stream().ok())
        .filter(|s| s.dict.has(b"SMask"))
        .count()
}

#[test]
fn a_flattened_estimate_is_the_size_of_the_export_and_press_files_hold_no_soft_masks() {
    let books = Books::new(Arc::new(Recorder::default()));
    let (id, _dir) = book_with_a_feathered_picture(&books);
    let request = ExportRequest {
        targets: vec![
            ExportTarget::SharingPdf,
            ExportTarget::KdpInterior,
            ExportTarget::PrintShopPdf,
        ],
        ..ExportRequest::default()
    };
    let estimate = books.estimate_export(&id, &request).unwrap();
    let report = books.export_with(&id, &request).unwrap();
    let real: u64 = report.files.iter().map(|f| u64::from(f.bytes)).sum();
    let off = estimate.bytes.abs_diff(real) as f64 / real as f64;
    assert!(off < 0.01, "estimate {} against {real}", estimate.bytes);
    // The Sharing PDF keeps its transparency live, and the press files hold none.
    let masks: Vec<usize> = report.files.iter().map(|f| soft_masks(&f.path)).collect();
    assert!(masks[0] > 0, "{masks:?}");
    assert_eq!(&masks[1..], &[0, 0], "{masks:?}");
}

#[test]
fn asking_a_sharing_pdf_to_flatten_changes_its_size_estimate_and_the_file_the_same_way() {
    use folio_model::{ExportPrefs, PdfPrefs};
    let books = Books::new(Arc::new(Recorder::default()));
    let (id, _dir) = book_with_a_feathered_picture(&books);
    let sized = |flatten: bool| {
        let request = ExportRequest {
            targets: vec![ExportTarget::SharingPdf],
            prefs: Some(ExportPrefs {
                pdf: folio_model::PdfPrefsByTarget {
                    sharing: PdfPrefs {
                        flatten: Some(flatten),
                        ..PdfPrefs::default()
                    },
                    ..Default::default()
                },
                ..ExportPrefs::default()
            }),
            ..ExportRequest::default()
        };
        let estimate = books.estimate_export(&id, &request).unwrap();
        let report = books.export_with(&id, &request).unwrap();
        let real = u64::from(report.files[0].bytes);
        assert!(
            estimate.bytes.abs_diff(real) as f64 / (real as f64) < 0.01,
            "estimate {} against {real}",
            estimate.bytes
        );
        (soft_masks(&report.files[0].path), real)
    };
    let (live, live_bytes) = sized(false);
    let (flat, flat_bytes) = sized(true);
    assert!(live > 0 && flat == 0);
    assert_ne!(live_bytes, flat_bytes);
}
