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
fn the_back_has_the_description_and_a_clear_box_for_the_barcode() {
    let (mut book, _) = sample(BookKind::Picture);
    book.metadata.description = "A small otter who cannot sleep.".into();
    let frames = &cover_sheet(&book, Paper::White).unwrap().spreads[0].frames;
    assert!(frames.iter().any(|f| f.base().id.0 == "cover_blurb"));
    let Some(Frame::Shape(barcode)) = frames.iter().find(|f| f.base().id.0 == "cover_barcode")
    else {
        panic!("a barcode box")
    };
    assert_eq!(barcode.fill.as_ref().map(|c| c.0.as_str()), Some("#ffffff"));
    // No description, no blurb frame.
    book.metadata.description.clear();
    assert!(!cover_sheet(&book, Paper::White).unwrap().spreads[0]
        .frames
        .iter()
        .any(|f| f.base().id.0 == "cover_blurb"));
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
