// Starts a book of every kind and checks it is a real book, in memory and on disk
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use folio_model::{BookKind, Colour, Frame, LayoutMode, Pt, SpreadKind};
use folio_package::{Package, SaveOptions};
use folio_presets::{
    kind_infos, new_book, new_book_with_ids, Look, NewBookSpec, Paper, PresetError, Words,
};
use folio_session::Session;

const KINDS: [BookKind; 6] = [
    BookKind::Picture,
    BookKind::Novel,
    BookKind::Textbook,
    BookKind::Illustrated,
    BookKind::Poetry,
    BookKind::Memoir,
];

fn spec(kind: BookKind) -> NewBookSpec {
    NewBookSpec {
        kind,
        ..NewBookSpec::default()
    }
}

#[test]
fn the_six_kinds_are_offered_in_order_with_their_sizes() {
    let infos = kind_infos();
    let kinds: Vec<_> = infos.iter().map(|i| i.kind).collect();
    assert_eq!(kinds, KINDS);
    let picture = &infos[0];
    assert_eq!(picture.label, "Picture book");
    assert_eq!(picture.layout, LayoutMode::Fixed);
    assert_eq!(picture.trims[0].label, "8.5 × 8.5 in");
    assert_eq!(picture.trims[0].width, Pt(612.0));
    assert_eq!(picture.trims.len(), 4);
    assert_eq!(infos[1].layout, LayoutMode::Flowing);
    assert!(infos
        .iter()
        .all(|i| !i.trims.is_empty() && i.default_pages > 0));
}

#[test]
fn every_kind_at_every_size_makes_a_book_that_survives_a_save_and_opens_in_a_session() {
    let dir = tempfile::tempdir().unwrap();
    for info in kind_infos() {
        for trim in 0..info.trims.len() {
            let made = new_book(&NewBookSpec {
                kind: info.kind,
                trim_index: trim as u32,
                ..NewBookSpec::default()
            })
            .unwrap();
            assert_eq!(made.book.trim.w, info.trims[trim].width);
            assert_eq!(made.book.spreads[0].kind, SpreadKind::Cover);

            let file = dir.path().join(format!("{:?}-{trim}.folio", info.kind));
            let original = made.book.clone();
            let mut package = Package::new(made.book);
            *package.settings_mut() = made.settings;
            package.save(&file, &SaveOptions::default()).unwrap();
            let reopened = Package::open(&file).unwrap();
            // Nothing is lost or nudged by writing and reading: lengths come back to the last bit.
            assert_eq!(reopened.book(), &original);
            let session = Session::new(reopened.book().clone());
            assert!(!session.book().spreads.is_empty());
        }
    }
}

#[test]
fn a_kind_starts_with_its_own_geometry() {
    let picture = new_book(&spec(BookKind::Picture)).unwrap().book;
    assert_eq!(picture.layout, LayoutMode::Fixed);
    assert_eq!(picture.bleed, Pt::from_in(0.125));
    assert_eq!(picture.margins.inside, Pt::from_in(0.5));
    assert_eq!(picture.grid.columns, 6);

    let novel = new_book(&spec(BookKind::Novel)).unwrap().book;
    assert_eq!(novel.layout, LayoutMode::Flowing);
    assert_eq!(novel.bleed, Pt(0.0));
    assert_eq!(novel.grid.columns, 1);
    assert_eq!(novel.trim.w, Pt::from_in(5.0));
    assert_eq!(novel.trim.h, Pt::from_in(8.0));

    let textbook = new_book(&spec(BookKind::Textbook)).unwrap().book;
    assert_eq!(textbook.grid.columns, 12);
}

#[test]
fn samples_give_a_cover_and_the_kinds_sample_spreads() {
    let picture = new_book(&spec(BookKind::Picture)).unwrap().book;
    assert_eq!(picture.title, "The Otter Who Couldn't Sleep");
    // The cover plus the three spreads after it in the sample.
    assert_eq!(picture.spreads.len(), 4);
    assert_eq!(picture.spreads[0].kind, SpreadKind::Cover);
    assert!(picture.spreads[1..]
        .iter()
        .all(|s| s.kind == SpreadKind::Spread));
    // The sample cover brings its own picture placeholder over the cover colour.
    assert!(matches!(
        picture.spreads[0].pages[0].bg,
        Some(folio_model::Background::Colour { .. })
    ));
    assert!(picture.spreads[0]
        .frames
        .iter()
        .any(|f| matches!(f, Frame::Image(_))));

    let novel = new_book(&spec(BookKind::Novel)).unwrap().book;
    // A generated cover, then the title page and the chapter opener, then a spread.
    let kinds: Vec<_> = novel.spreads.iter().map(|s| s.kind).collect();
    assert_eq!(
        kinds,
        [
            SpreadKind::Cover,
            SpreadKind::Single,
            SpreadKind::Opener,
            SpreadKind::Spread
        ]
    );
}

#[test]
fn an_empty_start_is_a_cover_and_one_blank_page() {
    let book = new_book(&NewBookSpec {
        words: Words::Empty,
        ..spec(BookKind::Novel)
    })
    .unwrap()
    .book;
    assert_eq!(book.title, "Untitled");
    assert_eq!(book.spreads.len(), 2);
    assert_eq!(book.spreads[1].kind, SpreadKind::Opener);
    assert!(book.spreads[1].frames.is_empty());
    assert_eq!(book.page_count(), 1);
}

#[test]
fn the_title_author_and_cover_colour_go_where_they_belong() {
    let made = new_book(&NewBookSpec {
        title: "Pip and the Moon".into(),
        subtitle: "A bedtime story".into(),
        author: "Scott".into(),
        cover_colour: "#1f3a5f".into(),
        ..spec(BookKind::Picture)
    })
    .unwrap();
    let book = made.book;
    assert_eq!(book.title, "Pip and the Moon");
    assert_eq!(book.subtitle.as_deref(), Some("A bedtime story"));
    assert_eq!(book.authors[0].name, "Scott");

    let texts: Vec<String> = book.spreads[0]
        .frames
        .iter()
        .filter_map(|f| match f {
            Frame::Text(t) => Some(t.content.as_ref().unwrap().inline[0].text.clone()),
            _ => None,
        })
        .collect();
    assert!(texts.contains(&"Pip and the Moon".to_owned()), "{texts:?}");
    assert!(texts.iter().any(|t| t.contains("Scott")), "{texts:?}");
    assert!(!texts.iter().any(|t| t.contains("Mom")), "{texts:?}");
    assert_eq!(
        book.spreads[0].pages[0].bg,
        Some(folio_model::Background::Colour {
            colour: Colour("#1f3a5f".into())
        })
    );
    assert_eq!(made.settings["coverColour"], "#1f3a5f");
}

#[test]
fn a_generated_cover_carries_the_title_and_author_over_a_background_colour() {
    let book = new_book(&NewBookSpec {
        title: "Watercolour Basics".into(),
        author: "Mara".into(),
        ..spec(BookKind::Textbook)
    })
    .unwrap()
    .book;
    let cover = &book.spreads[0];
    assert_eq!(cover.frames.len(), 2, "title, byline");
    // The colour is the page's background, which already runs to the bleed line.
    assert!(matches!(
        cover.pages[0].bg,
        Some(folio_model::Background::Colour { .. })
    ));
}

#[test]
fn a_different_size_scales_the_sample_pages_to_fit() {
    let first = new_book(&spec(BookKind::Picture)).unwrap().book;
    let wider = new_book(&NewBookSpec {
        trim_index: 2, // 10 × 8 in
        ..spec(BookKind::Picture)
    })
    .unwrap()
    .book;
    let rect = |b: &folio_model::Book| b.spreads[1].frames[1].base().rect;
    assert!((rect(&wider).w.0 / rect(&first).w.0 - 10.0 / 8.5).abs() < 1e-9);
    assert!((rect(&wider).h.0 / rect(&first).h.0 - 8.0 / 8.5).abs() < 1e-9);
}

#[test]
fn the_paper_isbn_and_look_are_kept() {
    let made = new_book(&NewBookSpec {
        paper: Paper::Cream,
        isbn: "978-0-306-40615-7".into(),
        look: Look::Classic,
        ..spec(BookKind::Memoir)
    })
    .unwrap();
    assert_eq!(made.settings["paper"], "cream");
    assert_eq!(made.settings["plannedPages"], 212);
    assert_eq!(
        made.book.metadata.isbn.paperback.as_deref(),
        Some("978-0-306-40615-7")
    );
    assert_eq!(made.book.styles.look, "classic");
    assert_eq!(made.book.styles.font_family, "Literata");
    assert_eq!(made.book.styles.styles["body"].size_pt, Some(10.5));
}

#[test]
fn every_style_a_frame_uses_is_in_the_styles_file() {
    for kind in KINDS {
        let made = new_book(&spec(kind)).unwrap();
        for spread in &made.book.spreads {
            for frame in &spread.frames {
                if let Frame::Text(t) = frame {
                    assert!(
                        made.book.styles.styles.contains_key(&t.style.0),
                        "{kind:?} uses `{}` but does not define it",
                        t.style.0
                    );
                }
            }
        }
    }
}

#[test]
fn bad_choices_are_refused_with_a_reason() {
    assert_eq!(
        new_book(&NewBookSpec {
            trim_index: 9,
            ..spec(BookKind::Picture)
        })
        .unwrap_err(),
        PresetError::NoSuchTrim(9)
    );
    assert!(matches!(
        new_book(&NewBookSpec {
            isbn: "123".into(),
            ..spec(BookKind::Picture)
        }),
        Err(PresetError::BadIsbn(_))
    ));
    assert!(matches!(
        new_book(&NewBookSpec {
            cover_colour: "purple".into(),
            ..spec(BookKind::Picture)
        }),
        Err(PresetError::BadColour(_))
    ));
    let wire = serde_json::to_value(PresetError::BadIsbn("123".into())).unwrap();
    assert_eq!(wire["kind"], "badIsbn");
}

#[test]
fn no_two_parts_of_a_book_share_an_id_and_a_counter_makes_them_repeatable() {
    let count = |mut next: u32| {
        move |prefix: &str| {
            next += 1;
            format!("{prefix}_{next}")
        }
    };
    let a = new_book_with_ids(&spec(BookKind::Textbook), &mut count(0))
        .unwrap()
        .book;
    let b = new_book_with_ids(&spec(BookKind::Textbook), &mut count(0))
        .unwrap()
        .book;
    assert_eq!(
        serde_json::to_value(&a).unwrap(),
        serde_json::to_value(&b).unwrap()
    );

    let mut seen = std::collections::BTreeSet::new();
    for spread in &a.spreads {
        assert!(seen.insert(spread.id.0.clone()));
        for page in &spread.pages {
            assert!(seen.insert(page.id.0.clone()));
        }
        for frame in &spread.frames {
            assert!(seen.insert(frame.base().id.0.clone()));
        }
    }
}
