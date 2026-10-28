// Tests for the paperback cover sheet: its size follows the pages, and the spine only takes text when thick enough
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use folio_export::cover::{cover_sheet, spine_pages, SPINE_TEXT_FROM};
use folio_export::job::{export_target, Target};
use folio_export::pdf::NoAssets;
use folio_layout::{FontBook, StyleSheet};
use folio_model::{paperback_cover, Book, BookKind, Frame, FrameId, PageId, Paper, SpreadId};
use folio_presets::{new_book, NewBookSpec};
use lopdf::Document;

fn sample(kind: BookKind) -> (Book, StyleSheet) {
    let made = new_book(&NewBookSpec {
        kind,
        ..NewBookSpec::default()
    })
    .unwrap();
    let styles = StyleSheet::from_styles(&made.book.styles);
    (made.book, styles)
}

/// The book with enough copies of its last spread to be `pages` long inside.
fn thick(mut book: Book, pages: u32) -> Book {
    let template = book.spreads.last().unwrap().clone();
    let mut n = 0;
    while spine_pages(&book) < pages {
        n += 1;
        let mut spread = template.clone();
        spread.id = SpreadId(format!("sp_extra_{n}"));
        for page in &mut spread.pages {
            page.id = PageId(format!("pg_extra_{n}_{}", page.id.0));
        }
        for frame in &mut spread.frames {
            let id = FrameId(format!("fr_extra_{n}_{}", frame.base().id.0));
            match frame {
                Frame::Text(f) => f.base.id = id,
                Frame::Image(f) => f.base.id = id,
                Frame::Shape(f) => f.base.id = id,
                Frame::Table(f) => f.base.id = id,
                _ => {}
            }
        }
        book.spreads.push(spread);
    }
    book
}

fn cover_pdf(book: &Book, styles: &StyleSheet, paper: Paper) -> Vec<u8> {
    let fonts = FontBook::bundled();
    let made = export_target(
        Target::KdpCover,
        paper,
        book,
        styles,
        &fonts,
        &NoAssets,
        None,
    )
    .unwrap();
    assert_eq!(made.len(), 1);
    assert_eq!(
        made[0].file_name,
        "the-otter-who-couldnt-sleep-paperback-cover.pdf"
    );
    made[0].bytes.clone()
}

#[test]
fn the_sheet_is_two_covers_and_a_spine_wide_with_the_bleed_all_round() {
    let (book, styles) = sample(BookKind::Picture);
    let sheet = cover_sheet(&book, Paper::White).unwrap();
    let maths = paperback_cover(book.trim, spine_pages(&book), Paper::White);
    assert_eq!(sheet.spreads.len(), 1);
    assert!((sheet.trim.w.0 + 2.0 * sheet.bleed.0 - maths.width.0).abs() < 0.01);
    assert!((sheet.trim.h.0 + 2.0 * sheet.bleed.0 - maths.height.0).abs() < 0.01);

    let bytes = cover_pdf(&book, &styles, Paper::White);
    let doc = Document::load_mem(&bytes).unwrap();
    assert_eq!(doc.get_pages().len(), 1, "one sheet");
    let page = *doc.get_pages().values().next().unwrap();
    let media: Vec<f64> = doc
        .get_object(page)
        .unwrap()
        .as_dict()
        .unwrap()
        .get(b"MediaBox")
        .unwrap()
        .as_array()
        .unwrap()
        .iter()
        .map(|n| n.as_float().unwrap() as f64)
        .collect();
    assert!(
        (media[2] - maths.width.0).abs() < 0.05,
        "{media:?} vs {}",
        maths.width.0
    );
    assert!((media[3] - maths.height.0).abs() < 0.05);
}

#[test]
fn a_thicker_book_has_a_wider_spine_and_paper_changes_it() {
    let (book, _) = sample(BookKind::Picture);
    let thin = cover_sheet(&book, Paper::White).unwrap().trim.w.0;
    let thick_book = thick(book.clone(), 200);
    let wide = cover_sheet(&thick_book, Paper::White).unwrap().trim.w.0;
    assert!(wide > thin);
    let cream = cover_sheet(&thick_book, Paper::Cream).unwrap().trim.w.0;
    assert!(cream > wide, "cream paper is thicker");
}

#[test]
fn the_spine_carries_the_title_only_from_the_page_count_where_kdp_allows_text() {
    let (book, _) = sample(BookKind::Picture);
    let has_spine = |b: &Book| {
        cover_sheet(b, Paper::White).unwrap().spreads[0]
            .frames
            .iter()
            .any(|f| f.base().id.0 == "cover_spine")
    };
    assert!(!has_spine(&book), "a picture book is too thin");
    assert!(!has_spine(&thick(book.clone(), SPINE_TEXT_FROM - 2)));
    let enough = thick(book, SPINE_TEXT_FROM);
    assert!(has_spine(&enough));
    let spine = cover_sheet(&enough, Paper::White).unwrap().spreads[0]
        .frames
        .iter()
        .find(|f| f.base().id.0 == "cover_spine")
        .cloned()
        .unwrap();
    assert_eq!(spine.base().rotation, 90.0, "it reads down the spine");
}

#[test]
fn the_sheet_is_the_cover_spread_with_a_clear_box_for_the_barcode_over_it() {
    let (mut book, _) = sample(BookKind::Picture);
    // What the person put on the back is a frame of the cover, kept where it is and not locked.
    let mut back = folio_model::words_frame(
        "fr_back",
        folio_model::Rect {
            x: folio_model::Pt(30.0),
            y: folio_model::Pt(60.0),
            w: folio_model::Pt(120.0),
            h: folio_model::Pt(40.0),
        },
        0.0,
        "Praise for the otter",
    );
    back.base_mut().locked = false;
    book.spreads[0].frames.push(back);
    let frames = &cover_sheet(&book, Paper::White).unwrap().spreads[0].frames;
    let mine = frames.iter().find(|f| f.base().id.0 == "fr_back").unwrap();
    assert_eq!(mine.base().rect.x.0, 30.0);
    assert!(!mine.base().locked);
    let Some(Frame::Shape(barcode)) = frames.iter().find(|f| f.base().id.0 == "cover_barcode")
    else {
        panic!("a barcode box")
    };
    assert_eq!(barcode.fill.as_ref().map(|c| c.0.as_str()), Some("#ffffff"));
    assert!(
        barcode.base.locked,
        "what Folio draws is not the person's to move"
    );
    // Folio no longer makes a blurb of its own: the description is a frame of the cover.
    assert!(!frames.iter().any(|f| f.base().id.0 == "cover_blurb"));
}

#[test]
fn the_cover_is_in_ink_with_no_rgb_left_and_no_bookmarks_or_xmp() {
    let (book, styles) = sample(BookKind::Picture);
    let bytes = cover_pdf(&book, &styles, Paper::White);
    let doc = Document::load_mem(&bytes).unwrap();
    let page = *doc.get_pages().values().next().unwrap();
    let ops = doc.get_and_decode_page_content(page).unwrap().operations;
    assert!(ops
        .iter()
        .any(|op| matches!(op.operator.as_str(), "k" | "K")));
    assert!(!ops
        .iter()
        .any(|op| matches!(op.operator.as_str(), "rg" | "RG")));
    let catalog = doc.catalog().unwrap();
    assert!(!catalog.has(b"Outlines") && !catalog.has(b"Metadata"));
}

#[test]
fn a_book_with_no_cover_makes_no_sheet() {
    let (mut book, _) = sample(BookKind::Picture);
    book.spreads.remove(0);
    assert!(cover_sheet(&book, Paper::White).is_none());
}

fn bar_count(book: &Book) -> usize {
    let sheet = cover_sheet(book, Paper::White).unwrap();
    sheet
        .spreads
        .iter()
        .flat_map(|s| &s.frames)
        .filter(|f| f.base().id.0.starts_with("cover_bar_"))
        .count()
}

#[test]
fn a_good_paperback_isbn_is_drawn_as_a_barcode_and_a_bad_one_leaves_the_box_clear() {
    let (mut book, _) = sample(BookKind::Novel);
    assert_eq!(bar_count(&book), 0);
    book.metadata.isbn.paperback = Some("978-0-306-40615-8".into());
    assert_eq!(bar_count(&book), 0);
    book.metadata.isbn.paperback = Some("978-0-306-40615-7".into());
    // An EAN-13 has 30 bars: the guards and the digits.
    assert!(
        (25..=35).contains(&bar_count(&book)),
        "{}",
        bar_count(&book)
    );
}

#[test]
fn the_barcode_follows_the_books_choice_and_a_price_addon_adds_bars() {
    use folio_model::BarcodeChoice;
    let (mut book, _) = sample(BookKind::Novel);
    book.metadata.isbn.paperback = Some("978-0-306-40615-7".into());
    let auto = bar_count(&book);
    assert!((25..=35).contains(&auto), "{auto}");
    book.barcode = BarcodeChoice::Kdp;
    assert_eq!(bar_count(&book), 0, "KDP adds its own");
    book.barcode = BarcodeChoice::Own { addon: None };
    assert_eq!(bar_count(&book), auto);
    book.barcode = BarcodeChoice::Own {
        addon: Some("90000".into()),
    };
    let sheet = cover_sheet(&book, Paper::White).unwrap();
    let addon_bars = sheet
        .spreads
        .iter()
        .flat_map(|s| &s.frames)
        .filter(|f| {
            f.base().id.0.starts_with("cover_addon_") && f.base().id.0 != "cover_addon_digits"
        })
        .count();
    assert!(addon_bars >= 8, "{addon_bars}");
    assert!(sheet
        .spreads
        .iter()
        .flat_map(|s| &s.frames)
        .any(|f| f.base().id.0 == "cover_addon_digits"));
}

#[test]
fn the_sheet_has_guides_at_the_folds_and_the_safe_areas_for_the_canvas_to_snap_to() {
    use folio_model::Axis;
    let (book, _) = sample(BookKind::Picture);
    let sheet = cover_sheet(&book, Paper::White).unwrap();
    let w = book.trim.w.0;
    let spine = paperback_cover(book.trim, spine_pages(&book), Paper::White)
        .spine
        .0;
    let xs: Vec<f64> = sheet.spreads[0]
        .guides
        .iter()
        .filter(|g| g.axis == Axis::X)
        .map(|g| g.pos.0)
        .collect();
    for fold in [0.0, w, w + spine, 2.0 * w + spine, 18.0, w - 18.0] {
        assert!(
            xs.iter().any(|x| (x - fold).abs() < 1e-9),
            "a guide at {fold}"
        );
    }
    assert!(sheet.spreads[0].guides.iter().all(|g| g.locked));
}

#[test]
fn the_cover_of_an_older_book_moves_onto_the_sheet_with_its_back_cover_words_as_frames() {
    use folio_model::{upgrade_cover, BackNote, Pt, Rect, SCHEMA_VERSION};
    let (mut book, _) = sample(BookKind::Picture);
    let front = book.spreads[0].frames.len();
    let x_before = book.spreads[0].frames[0].base().rect.x.0;
    book.schema = 2;
    book.metadata.description = "A small otter who cannot sleep.".into();
    book.back_notes = vec![
        BackNote {
            id: "n1".into(),
            text: "Praise for the otter".into(),
            rect: Rect {
                x: Pt(30.0),
                y: Pt(200.0),
                w: Pt(120.0),
                h: Pt(40.0),
            },
            size: 11.0,
            picture: None,
            shape: None,
            fill: None,
            rotation: Some(15.0),
        },
        BackNote {
            id: "n2".into(),
            text: String::new(),
            rect: Rect {
                x: Pt(30.0),
                y: Pt(60.0),
                w: Pt(100.0),
                h: Pt(100.0),
            },
            size: 12.0,
            picture: Some("assets/images/otter.png".into()),
            shape: None,
            fill: None,
            rotation: None,
        },
    ];
    upgrade_cover(&mut book, Some(Paper::Cream));
    assert_eq!(book.schema, SCHEMA_VERSION);
    assert_eq!(book.paper, Paper::Cream);
    assert!(book.back_notes.is_empty());
    let frames = &book.spreads[0].frames;
    // The front is set past the back and the spine.
    let offset = folio_model::front_offset(&book).0;
    assert_eq!(frames[0].base().rect.x.0, x_before + offset);
    // The description and the two notes are frames of the cover, after the front's own.
    assert_eq!(frames.len(), front + 3);
    let by_id = |id: &str| frames.iter().find(|f| f.base().id.0 == id).unwrap();
    assert!(matches!(by_id("blurb"), Frame::Text(_)));
    assert_eq!(by_id("note_n1").base().rotation, 15.0);
    assert_eq!(by_id("note_n1").base().rect.x.0, 30.0);
    assert!(matches!(by_id("note_n2"), Frame::Image(_)));
    // A book already on this schema is left as it is.
    let again = book.clone();
    upgrade_cover(&mut book, None);
    assert_eq!(book, again);
}

#[test]
fn the_blurb_follows_the_description_until_the_person_changes_the_words_in_it() {
    use folio_model::sync_blurb;
    let (mut book, _) = sample(BookKind::Picture);
    let text = |book: &Book| -> Option<String> {
        book.spreads[0].frames.iter().find_map(|f| match f {
            Frame::Text(t) if t.base.id.0 == "blurb" => t
                .content
                .as_ref()
                .map(|c| c.inline.iter().map(|r| r.text.as_str()).collect()),
            _ => None,
        })
    };
    // A description typed where there was none makes the frame.
    sync_blurb(&mut book, "", "First words");
    assert_eq!(text(&book).as_deref(), Some("First words"));
    // While the frame still holds the last description, a new one replaces it.
    sync_blurb(&mut book, "First words", "Second words");
    assert_eq!(text(&book).as_deref(), Some("Second words"));
    // Words changed on the cover are the person's, and a new description leaves them alone.
    if let Some(Frame::Text(t)) = book.spreads[0]
        .frames
        .iter_mut()
        .find(|f| f.base().id.0 == "blurb")
    {
        t.content.as_mut().unwrap().inline[0].text = "My own words".into();
    }
    sync_blurb(&mut book, "Second words", "Third words");
    assert_eq!(text(&book).as_deref(), Some("My own words"));
}
