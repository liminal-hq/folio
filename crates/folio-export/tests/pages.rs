// Tests for which pages a PDF has and how they sit on its sheets
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use folio_export::pages::{CoverPages, PageChoice, Sheets};
use folio_export::pdf::{export_pdf, NoAssets, PdfOptions, PdfTarget};
use folio_layout::{FontBook, StyleSheet};
use folio_model::{Book, BookKind, SpreadKind};
use folio_presets::{new_book, NewBookSpec};
use lopdf::Document;

fn sample() -> (Book, StyleSheet) {
    let made = new_book(&NewBookSpec {
        kind: BookKind::Picture,
        ..NewBookSpec::default()
    })
    .unwrap();
    let styles = StyleSheet::from_styles(&made.book.styles);
    (made.book, styles)
}

/// The width of each page of the file, in points.
fn widths(book: &Book, styles: &StyleSheet, target: PdfTarget, pages: PageChoice) -> Vec<f64> {
    let bytes = export_pdf(
        book,
        styles,
        &FontBook::bundled(),
        &NoAssets,
        &PdfOptions {
            target,
            pages,
            ..PdfOptions::default()
        },
    )
    .unwrap();
    let doc = Document::load_mem(&bytes).unwrap();
    doc.get_pages()
        .values()
        .map(|&id| {
            let media = doc
                .get_object(id)
                .unwrap()
                .as_dict()
                .unwrap()
                .get(b"MediaBox")
                .unwrap()
                .as_array()
                .unwrap();
            f64::from(media[2].as_float().unwrap())
        })
        .collect()
}

#[test]
fn a_range_keeps_only_those_pages() {
    let (book, styles) = sample();
    let all = widths(&book, &styles, PdfTarget::Sharing, PageChoice::default());
    let some = widths(
        &book,
        &styles,
        PdfTarget::Sharing,
        PageChoice {
            range: Some((2, 4)),
            ..PageChoice::default()
        },
    );
    assert!(all.len() > 4);
    assert_eq!(some.len(), 3);
}

#[test]
fn the_cover_is_the_front_the_whole_wrap_or_left_out() {
    let (book, styles) = sample();
    assert_eq!(book.spreads[0].kind, SpreadKind::Cover);
    let front = widths(&book, &styles, PdfTarget::Sharing, PageChoice::default());
    let none = widths(
        &book,
        &styles,
        PdfTarget::Sharing,
        PageChoice {
            cover: CoverPages::None,
            ..PageChoice::default()
        },
    );
    let wrap = widths(
        &book,
        &styles,
        PdfTarget::Sharing,
        PageChoice {
            cover: CoverPages::FrontAndBack,
            ..PageChoice::default()
        },
    );
    assert_eq!(none.len(), front.len() - 1);
    assert_eq!(wrap.len(), front.len());
    assert!(
        wrap[0] > front[0] * 1.9,
        "back, spine and front are on one page: {} against {}",
        wrap[0],
        front[0]
    );
}

#[test]
fn spreads_put_two_pages_on_one_sheet_and_leave_the_cover_alone() {
    let (book, styles) = sample();
    let single = widths(&book, &styles, PdfTarget::Sharing, PageChoice::default());
    let spreads = widths(
        &book,
        &styles,
        PdfTarget::Sharing,
        PageChoice {
            sheets: Sheets::Spreads,
            ..PageChoice::default()
        },
    );
    assert!(spreads.len() < single.len());
    assert!((spreads[0] - single[0]).abs() < 0.01, "the cover is alone");
    let widest = spreads.iter().cloned().fold(0.0, f64::max);
    assert!((widest - 2.0 * single[0]).abs() < 0.5, "{widest}");
}

#[test]
fn a_print_file_is_always_the_whole_book_as_single_pages() {
    let (book, styles) = sample();
    let plain = widths(&book, &styles, PdfTarget::Print, PageChoice::default());
    let asked = widths(
        &book,
        &styles,
        PdfTarget::Print,
        PageChoice {
            range: Some((1, 2)),
            cover: CoverPages::FrontAndBack,
            sheets: Sheets::Spreads,
        },
    );
    assert_eq!(plain, asked);
}
