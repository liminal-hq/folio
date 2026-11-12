// Tests for what a print shop's PDF can be asked for: marks, bleed, room for marks and press ink
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use folio_export::pdf::{export_pdf, InkOptions, NoAssets, PdfOptions, PdfTarget};
use folio_export::shop::{ShopBleed, ShopOptions};
use folio_layout::{FontBook, StyleSheet};
use folio_model::{Book, BookKind};
use folio_presets::{new_book, NewBookSpec};
use lopdf::Document;

const PRESS: &str = "/usr/share/ghostscript/iccprofiles/default_cmyk.icc";

fn sample() -> (Book, StyleSheet) {
    let made = new_book(&NewBookSpec {
        kind: BookKind::Picture,
        ..NewBookSpec::default()
    })
    .unwrap();
    let styles = StyleSheet::from_styles(&made.book.styles);
    (made.book, styles)
}

fn shop(book: &Book, styles: &StyleSheet, crop_marks: bool, shop: ShopOptions) -> Document {
    let bytes = export_pdf(
        book,
        styles,
        &FontBook::bundled(),
        &NoAssets,
        &PdfOptions {
            target: PdfTarget::PrintShop,
            crop_marks,
            shop,
            ..PdfOptions::default()
        },
    )
    .unwrap();
    Document::load_mem(&bytes).unwrap()
}

fn media(doc: &Document) -> Vec<f64> {
    let (_, &page) = doc.get_pages().iter().next().unwrap();
    doc.get_object(page)
        .unwrap()
        .as_dict()
        .unwrap()
        .get(b"MediaBox")
        .unwrap()
        .as_array()
        .unwrap()
        .iter()
        .map(|n| f64::from(n.as_float().unwrap()))
        .collect()
}

#[test]
fn crop_marks_are_a_choice_and_without_them_there_is_no_room_for_them() {
    let (book, styles) = sample();
    let (w, bleed) = (book.trim.w.0, book.bleed.0);
    let with = media(&shop(&book, &styles, true, ShopOptions::default()));
    let without = media(&shop(&book, &styles, false, ShopOptions::default()));
    assert!((with[2] - (w + 2.0 * (bleed + 24.0))).abs() < 0.01);
    assert!((without[2] - (w + 2.0 * bleed)).abs() < 0.01);
}

#[test]
fn the_bleed_and_the_room_for_marks_can_be_set() {
    let (book, styles) = sample();
    let w = book.trim.w.0;
    let doc = shop(
        &book,
        &styles,
        true,
        ShopOptions {
            bleed: ShopBleed::Custom(9.0),
            slug: Some(36.0),
            ..ShopOptions::default()
        },
    );
    assert!((media(&doc)[2] - (w + 2.0 * (9.0 + 36.0))).abs() < 0.01);
    // A slug too small to hold the marks is made big enough.
    let small = shop(
        &book,
        &styles,
        true,
        ShopOptions {
            bleed: ShopBleed::Custom(0.0),
            slug: Some(2.0),
            ..ShopOptions::default()
        },
    );
    assert!((media(&small)[2] - (w + 2.0 * 18.0)).abs() < 0.01);
}

fn output_components(doc: &Document) -> i64 {
    let catalog = doc.catalog().unwrap();
    let intents = doc
        .dereference(catalog.get(b"OutputIntents").unwrap())
        .unwrap()
        .1
        .as_array()
        .unwrap();
    let intent = doc.dereference(&intents[0]).unwrap().1.as_dict().unwrap();
    doc.dereference(intent.get(b"DestOutputProfile").unwrap())
        .unwrap()
        .1
        .as_stream()
        .unwrap()
        .dict
        .get(b"N")
        .unwrap()
        .as_i64()
        .unwrap()
}

#[test]
fn a_press_profile_makes_the_file_cmyk_with_that_output_intent() {
    let Ok(profile) = std::fs::read(PRESS) else {
        return; // No CMYK profile is shipped; the test uses Ghostscript's when there is one.
    };
    let (book, styles) = sample();
    let ink = shop(
        &book,
        &styles,
        true,
        ShopOptions {
            ink: Some(InkOptions {
                profile: Some(profile),
            }),
            ..ShopOptions::default()
        },
    );
    assert_eq!(output_components(&ink), 4);
}

#[test]
fn ink_without_a_press_profile_leaves_the_shop_file_srgb() {
    let (book, styles) = sample();
    let doc = shop(
        &book,
        &styles,
        true,
        ShopOptions {
            ink: Some(InkOptions::default()),
            ..ShopOptions::default()
        },
    );
    assert_eq!(output_components(&doc), 3);
}
