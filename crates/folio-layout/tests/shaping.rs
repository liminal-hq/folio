// Tests for font resolution and shaping against the bundled families
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use folio_layout::{feature, shape, FontBook, FontRequest, Run};

fn book() -> FontBook {
    FontBook::bundled()
}

fn width(family: &str, weight: u16, italic: bool, text: &str, size: f64) -> f64 {
    let resolved = book()
        .resolve(&FontRequest::new(family, weight, italic))
        .unwrap();
    let mut run = Run::new(text, size);
    run.weight = weight;
    shape(&resolved.font, &run).width
}

#[test]
fn every_bundled_family_resolves_in_every_style_it_has() {
    let book = book();
    for family in FontBook::bundled_families() {
        for (weight, italic) in [(400, false), (700, false), (400, true), (700, true)] {
            let found = book
                .resolve(&FontRequest::new(family, weight, italic))
                .unwrap();
            assert!(!found.family_substituted, "{family} is bundled");
            assert_eq!(found.font.family(), family);
        }
    }
    assert_eq!(
        FontBook::bundled_families(),
        vec![
            "Literata",
            "Source Serif 4",
            "Inter",
            "Atkinson Hyperlegible"
        ]
    );
}

#[test]
fn the_bundled_fonts_may_be_embedded() {
    let book = book();
    for family in FontBook::bundled_families() {
        let found = book.resolve(&FontRequest::new(family, 400, false)).unwrap();
        assert!(found.font.embedding_allowed(), "{family}");
    }
}

#[test]
fn a_missing_family_is_replaced_and_says_so() {
    let found = book()
        .resolve(&FontRequest::new("Comic Papyrus", 400, false))
        .unwrap();
    assert!(found.family_substituted);
    assert_eq!(found.font.family(), "Literata");
    let sans = book()
        .resolve(&FontRequest::new("Fancy Sans", 400, false))
        .unwrap();
    assert_eq!(sans.font.family(), "Inter");
}

#[test]
fn family_names_match_without_regard_to_case() {
    let found = book()
        .resolve(&FontRequest::new("source serif 4", 400, false))
        .unwrap();
    assert!(!found.family_substituted);
    assert_eq!(found.font.family(), "Source Serif 4");
}

#[test]
fn a_family_with_no_italic_says_so() {
    let found = book()
        .resolve(&FontRequest::new("Inter", 400, true))
        .unwrap();
    assert!(found.italic_missing);
    assert!(!found.font.italic());
    let found = book()
        .resolve(&FontRequest::new("Literata", 400, true))
        .unwrap();
    assert!(!found.italic_missing);
    assert!(found.font.italic());
}

#[test]
fn the_same_font_is_loaded_once() {
    let book = book();
    let a = book
        .resolve(&FontRequest::new("Literata", 400, false))
        .unwrap();
    let b = book
        .resolve(&FontRequest::new("Literata", 700, false))
        .unwrap();
    assert!(std::sync::Arc::ptr_eq(&a.font, &b.font));
}

#[test]
fn a_variable_font_draws_any_weight_from_one_file() {
    let book = book();
    let literata = book
        .resolve(&FontRequest::new("Literata", 400, false))
        .unwrap();
    assert!(literata.font.is_variable());
    assert_eq!(literata.font.weight_for(600), 600);
    assert_eq!(literata.font.weight_for(5000), 900);
    let atkinson = book
        .resolve(&FontRequest::new("Atkinson Hyperlegible", 700, false))
        .unwrap();
    assert!(!atkinson.font.is_variable());
    assert_eq!(atkinson.font.weight_for(400), 700);
}

#[test]
fn bold_is_wider_than_regular() {
    let regular = width("Literata", 400, false, "Once upon a time", 12.0);
    let bold = width("Literata", 700, false, "Once upon a time", 12.0);
    assert!(bold > regular, "{bold} > {regular}");
    let regular = width(
        "Atkinson Hyperlegible",
        400,
        false,
        "Once upon a time",
        12.0,
    );
    let bold = width(
        "Atkinson Hyperlegible",
        700,
        false,
        "Once upon a time",
        12.0,
    );
    assert!(bold > regular);
}

#[test]
fn width_grows_in_proportion_to_the_size() {
    let at_10 = width("Inter", 400, false, "The otter slept", 10.0);
    let at_20 = width("Inter", 400, false, "The otter slept", 20.0);
    assert!(at_10 > 0.0);
    // Optical sizing nudges the shapes, so the ratio is close to 2, not exactly 2.
    assert!((at_20 / at_10 - 2.0).abs() < 0.1, "{}", at_20 / at_10);
}

#[test]
fn glyphs_carry_ascending_clusters_and_their_widths_sum_to_the_run() {
    let resolved = book()
        .resolve(&FontRequest::new("Literata", 400, false))
        .unwrap();
    let shaped = shape(&resolved.font, &Run::new("Hello, world", 14.0));
    assert_eq!(shaped.glyphs.len(), 12);
    let clusters: Vec<u32> = shaped.glyphs.iter().map(|g| g.cluster).collect();
    assert!(clusters.windows(2).all(|w| w[0] < w[1]));
    let sum: f64 = shaped.glyphs.iter().map(|g| g.advance).sum();
    assert!((sum - shaped.width).abs() < 1e-9);
    assert!(shaped.glyphs.iter().all(|g| g.advance > 0.0));
}

#[test]
fn ligatures_are_on_by_default_and_can_be_switched_off() {
    let resolved = book()
        .resolve(&FontRequest::new("Literata", 400, false))
        .unwrap();
    let on = shape(&resolved.font, &Run::new("office fi", 14.0));
    let off_feature = [feature("liga", 0).unwrap(), feature("clig", 0).unwrap()];
    let mut run = Run::new("office fi", 14.0);
    run.features = &off_feature;
    let off = shape(&resolved.font, &run);
    assert!(
        on.glyphs.len() < off.glyphs.len(),
        "{} < {}",
        on.glyphs.len(),
        off.glyphs.len()
    );
}

#[test]
fn small_capitals_change_the_shapes() {
    let resolved = book()
        .resolve(&FontRequest::new("Source Serif 4", 400, false))
        .unwrap();
    let plain = shape(&resolved.font, &Run::new("Otter", 14.0));
    let smcp = [feature("smcp", 1).unwrap()];
    let mut run = Run::new("Otter", 14.0);
    run.features = &smcp;
    let small = shape(&resolved.font, &run);
    assert_ne!(
        plain.glyphs.iter().map(|g| g.id).collect::<Vec<_>>(),
        small.glyphs.iter().map(|g| g.id).collect::<Vec<_>>()
    );
}

#[test]
fn tracking_adds_space_after_every_glyph() {
    let resolved = book()
        .resolve(&FontRequest::new("Inter", 400, false))
        .unwrap();
    let plain = shape(&resolved.font, &Run::new("spaced", 10.0));
    let mut run = Run::new("spaced", 10.0);
    run.tracking_em = 0.1;
    let tracked = shape(&resolved.font, &run);
    assert!((tracked.width - plain.width - 6.0).abs() < 1e-9);
}

#[test]
fn empty_text_shapes_to_nothing() {
    let resolved = book()
        .resolve(&FontRequest::new("Literata", 400, false))
        .unwrap();
    let shaped = shape(&resolved.font, &Run::new("", 12.0));
    assert!(shaped.glyphs.is_empty());
    assert_eq!(shaped.width, 0.0);
}

#[test]
fn metrics_scale_with_the_size() {
    let resolved = book()
        .resolve(&FontRequest::new("Literata", 400, false))
        .unwrap();
    let small = resolved.font.metrics(10.0);
    let big = resolved.font.metrics(20.0);
    assert!(small.ascent > 0.0 && small.descent > 0.0);
    assert!((big.ascent / small.ascent - 2.0).abs() < 1e-9);
    assert!(small.x_height < small.cap_height && small.cap_height < small.ascent);
}

#[test]
fn a_bad_feature_tag_is_refused() {
    assert!(feature("toolong", 1).is_none());
    assert!(feature("liga", 1).is_some());
}

#[test]
fn text_with_characters_the_font_lacks_still_shapes() {
    let resolved = book()
        .resolve(&FontRequest::new("Atkinson Hyperlegible", 400, false))
        .unwrap();
    let shaped = shape(&resolved.font, &Run::new("otter 🦦 slept", 12.0));
    assert!(shaped.width > 0.0);
    assert!(shaped.glyphs.iter().any(|g| g.id == 0), "a .notdef glyph");
}

#[test]
fn the_system_book_lists_the_bundled_families_first_and_still_resolves_them() {
    let book = FontBook::with_system_fonts();
    let families = book.families();
    assert_eq!(&families[..4], &FontBook::bundled_families()[..]);
    let found = book
        .resolve(&FontRequest::new("Literata", 400, false))
        .unwrap();
    assert!(!found.family_substituted);
}
