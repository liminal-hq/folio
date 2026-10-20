// Tests for the soft proof: a spread drawn as a press would print it
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use folio_export::pdf::NoAssets;
use folio_export::proof::proof_spread;
use folio_layout::{FontBook, StyleSheet};
use folio_presets::{new_book, NewBookSpec};

const PRESS: &str = "/usr/share/ghostscript/iccprofiles/default_cmyk.icc";

#[test]
fn a_spread_comes_back_as_each_page_drawn_and_proofed() {
    // A CMYK profile is not shipped; the test uses the one Ghostscript has, when there is one.
    let Ok(press) = std::fs::read(PRESS) else {
        return;
    };
    let made = new_book(&NewBookSpec::default()).unwrap();
    let styles = StyleSheet::from_styles(&made.book.styles);
    let spread = made.book.spreads[1].id.clone();
    let pages = proof_spread(
        &made.book,
        &styles,
        &FontBook::bundled(),
        &NoAssets,
        &spread,
        &press,
        300,
    )
    .unwrap()
    .unwrap();
    assert_eq!(pages.len(), made.book.spreads[1].pages.len());
    for page in &pages {
        let a = image::load_from_memory(&page.original).unwrap();
        let b = image::load_from_memory(&page.proofed).unwrap();
        assert_eq!((a.width(), a.height()), (b.width(), b.height()));
        assert_ne!(page.original, page.proofed);
        assert!((0.0..=100.0).contains(&page.outside));
    }
}

#[test]
fn a_file_that_is_not_a_profile_is_refused() {
    let made = new_book(&NewBookSpec::default()).unwrap();
    let styles = StyleSheet::from_styles(&made.book.styles);
    let spread = made.book.spreads[1].id.clone();
    assert!(proof_spread(
        &made.book,
        &styles,
        &FontBook::bundled(),
        &NoAssets,
        &spread,
        b"nope",
        300
    )
    .is_err());
}
