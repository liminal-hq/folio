// Decoration: page numbers, their backdrops and frames marked decorative
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use folio_layout::decoration::{Decoration, Decorations};
use folio_layout::wire::PreflightKind;
use folio_layout::{FontBook, Layouter, StyleSheet};
use folio_model::{
    Book, BookKind, Fit, Frame, FrameBase, FrameId, ImageFrame, ImageRef, InlineContent, Pt, Rect,
    Run, SpreadKind, StyleId, TextFrame,
};
use folio_presets::{new_book, NewBookSpec, Words};

fn base(id: &str, x: f64, y: f64, w: f64, h: f64) -> FrameBase {
    FrameBase {
        id: FrameId(id.into()),
        rect: Rect {
            x: Pt(x),
            y: Pt(y),
            w: Pt(w),
            h: Pt(h),
        },
        rotation: 0.0,
        locked: false,
        from_master: None,
        description: None,
        decorative: false,
    }
}

fn words(id: &str, x: f64, y: f64, w: f64, h: f64, text: &str) -> Frame {
    Frame::Text(TextFrame {
        base: base(id, x, y, w, h),
        style: StyleId("body".into()),
        content: Some(InlineContent::of(vec![Run {
            text: text.into(),
            ..Default::default()
        }])),
        flow: None,
        columns: 1,
        baseline_lock: false,
        overrides: None,
    })
}

fn picture(id: &str, x: f64, y: f64, w: f64, h: f64, alt: Option<&str>) -> Frame {
    let mut base = base(id, x, y, w, h);
    base.description = alt.map(str::to_owned);
    Frame::Image(ImageFrame {
        base,
        image: ImageRef {
            asset: Some("a.png".into()),
            fit: Fit::Fill,
            focus: (0.5, 0.5),
            crop: None,
        },
        placeholder_label: None,
    })
}

/// A square book of `n` two-page spreads shaped like a real picture book: a page number in a
/// pumpkin at the foot of each page, and a picture and some words above.
fn book(n: usize) -> (Book, StyleSheet) {
    let made = new_book(&NewBookSpec {
        kind: BookKind::Picture,
        words: Words::Samples,
        ..NewBookSpec::default()
    })
    .unwrap();
    let mut book = made.book;
    book.trim.w = Pt(612.0);
    book.trim.h = Pt(612.0);
    book.margins.top = Pt(36.0);
    book.margins.bottom = Pt(36.0);
    let template = book
        .spreads
        .iter()
        .find(|s| s.kind == SpreadKind::Spread)
        .unwrap()
        .clone();
    book.spreads.retain(|s| s.kind != SpreadKind::Spread);
    for i in 0..n {
        let mut spread = template.clone();
        spread.id = folio_model::SpreadId(format!("sp_{i}"));
        spread.frames = vec![
            picture(&format!("fr_i{i}"), 36.0, 30.0, 540.0, 485.0, None),
            picture(&format!("fr_pk{i}a"), 276.0, 526.0, 60.0, 48.0, None),
            words(
                &format!("fr_pn{i}a"),
                276.0,
                541.0,
                60.0,
                22.0,
                &format!("{}", 2 * i),
            ),
            picture(&format!("fr_pk{i}b"), 888.0, 526.0, 60.0, 48.0, None),
            words(
                &format!("fr_pn{i}b"),
                888.0,
                541.0,
                60.0,
                22.0,
                &format!("{}", 2 * i + 1),
            ),
            words(
                &format!("fr_t{i}"),
                678.0,
                36.0,
                456.0,
                480.0,
                "Once upon a time.",
            ),
        ];
        book.spreads.push(spread);
    }
    let styles = StyleSheet::from_styles(&book.styles);
    (book, styles)
}

/// Where the spread `sp_i` of `book(n)` sits among the book's spreads.
fn at(book: &Book, i: usize) -> usize {
    book.spreads
        .iter()
        .position(|s| s.id.0 == format!("sp_{i}"))
        .unwrap()
}

fn kinds(book: &Book, styles: &StyleSheet, spread: usize, kind: PreflightKind) -> Vec<String> {
    let fonts = FontBook::bundled();
    let layout = Layouter::new(book, styles, &fonts)
        .layout_spread(&book.spreads[spread].id)
        .unwrap()
        .unwrap();
    layout
        .preflight
        .iter()
        .filter(|i| i.kind == kind)
        .map(|i| i.frame.as_ref().unwrap().0.clone())
        .collect()
}

#[test]
fn repeated_numbers_at_the_foot_of_a_page_are_page_numbers_and_their_pumpkins_are_backdrops() {
    let (book, _) = book(4);
    let found = Decorations::of_book(&book);
    let sp = &book.spreads[at(&book, 1)].id;
    let get = |f: &str| found.of(sp, &FrameId(f.into()));
    assert_eq!(get("fr_pn1a"), Some(Decoration::PageNumber));
    assert_eq!(get("fr_pn1b"), Some(Decoration::PageNumber));
    assert_eq!(get("fr_pk1a"), Some(Decoration::PageNumberBackdrop));
    assert_eq!(get("fr_pk1b"), Some(Decoration::PageNumberBackdrop));
    // The story, its picture and the full-page picture are content.
    assert_eq!(get("fr_t1"), None);
    assert_eq!(get("fr_i1"), None);
}

#[test]
fn preflight_asks_for_the_big_picture_but_not_for_the_pumpkins() {
    let (book, styles) = book(4);
    let missing = kinds(&book, &styles, at(&book, 1), PreflightKind::AltMissing);
    assert_eq!(missing, vec!["fr_i1".to_string()]);
}

#[test]
fn a_number_alone_in_the_book_is_not_assumed_to_be_a_page_number() {
    let (mut book, _) = book(1);
    let first = at(&book, 0);
    book.spreads[first]
        .frames
        .retain(|f| f.base().id.0 != "fr_pn0b");
    let first = at(&book, 0);
    book.spreads[first]
        .frames
        .retain(|f| f.base().id.0 != "fr_pk0b");
    // One left page number, nowhere repeated.
    let found = Decorations::of_book(&book);
    assert_eq!(
        found.of(&book.spreads[at(&book, 0)].id, &FrameId("fr_pn0a".into())),
        None
    );
    assert_eq!(
        found.of(&book.spreads[at(&book, 0)].id, &FrameId("fr_pk0a".into())),
        None
    );
}

#[test]
fn words_and_pictures_that_do_not_look_like_page_numbers_stay_content() {
    let (mut book, _) = book(4);
    let sp = book.spreads[at(&book, 1)].id.clone();
    let second = at(&book, 1);
    let frames = &mut book.spreads[second].frames;
    // A number up the page, big words, a pumpkin the person described.
    frames.push(words("fr_up", 276.0, 200.0, 60.0, 22.0, "7"));
    frames.push(words("fr_big", 36.0, 500.0, 500.0, 70.0, "12"));
    frames.retain(|f| f.base().id.0 != "fr_pk1a");
    frames.insert(
        1,
        picture("fr_pk1a", 276.0, 526.0, 60.0, 48.0, Some("A pumpkin")),
    );
    let found = Decorations::of_book(&book);
    let get = |f: &str| found.of(&sp, &FrameId(f.into()));
    assert_eq!(get("fr_up"), None);
    assert_eq!(get("fr_big"), None);
    assert_eq!(get("fr_pk1a"), None, "a described picture is content");
    assert_eq!(get("fr_pn1a"), Some(Decoration::PageNumber));
}

#[test]
fn a_frame_the_person_marked_is_decoration_and_an_empty_description_is_not() {
    let (mut book, styles) = book(2);
    // Nothing marks the big picture decorative by itself.
    assert_eq!(
        kinds(&book, &styles, at(&book, 0), PreflightKind::AltMissing),
        vec!["fr_i0".to_string()]
    );
    let first = at(&book, 0);
    for frame in book.spreads[first].frames.iter_mut() {
        if frame.base().id.0 == "fr_i0" {
            frame.base_mut().decorative = true;
        }
        if frame.base().id.0 == "fr_t0" {
            frame.base_mut().decorative = true;
        }
    }
    assert!(kinds(&book, &styles, at(&book, 0), PreflightKind::AltMissing).is_empty());
    let found = Decorations::of_book(&book);
    let sp = &book.spreads[at(&book, 0)].id;
    assert_eq!(
        found.of(sp, &FrameId("fr_i0".into())),
        Some(Decoration::Marked)
    );
    assert_eq!(
        found.of(sp, &FrameId("fr_t0".into())),
        Some(Decoration::Marked)
    );
}
