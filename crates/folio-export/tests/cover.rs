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

/// The numbers before each `k` operator in a page's content: the cyan, magenta, yellow and black
/// of every colour set in ink.
fn ink_colours(doc: &Document) -> Vec<[f32; 4]> {
    let page = *doc.get_pages().values().next().unwrap();
    let content = doc.get_and_decode_page_content(page).unwrap();
    content
        .operations
        .iter()
        .filter(|op| op.operator == "k" && op.operands.len() == 4)
        .map(|op| {
            let n: Vec<f32> = op.operands.iter().map(|o| o.as_float().unwrap()).collect();
            [n[0], n[1], n[2], n[3]]
        })
        .collect()
}

#[test]
fn a_swatch_with_ink_prints_as_exactly_that_ink_and_one_without_is_separated() {
    use folio_model::{BookSwatch, Colour, Ink, SwatchId};
    let (mut book, styles) = sample(BookKind::Picture);
    book.styles.swatches = vec![
        BookSwatch {
            id: SwatchId("brand".into()),
            name: "Brand".into(),
            display: Colour("#000000".into()),
            ink: Some(Ink {
                c: 20,
                m: 100,
                y: 80,
                k: 10,
            }),
        },
        BookSwatch {
            id: SwatchId("plain".into()),
            name: "Plain".into(),
            display: Colour("#336699".into()),
            ink: None,
        },
    ];
    book.spreads[0].pages[0].bg = Some(folio_model::Background::Colour {
        colour: Colour("swatch:brand".into()),
    });
    let doc = Document::load_mem(&cover_pdf(&book, &styles, Paper::White)).unwrap();
    let near = |a: f32, b: f32| (a - b).abs() < 0.01;
    // Twenty, a hundred, eighty and ten per cent of the plates, as the swatch says.
    assert!(
        ink_colours(&doc)
            .iter()
            .any(|c| near(c[0], 0.2) && near(c[1], 1.0) && near(c[2], 0.8) && near(c[3], 0.1)),
        "{:?}",
        ink_colours(&doc)
    );
    // A swatch without ink is separated from its display colour like any hex colour: a blue has
    // cyan and little yellow.
    book.spreads[0].pages[0].bg = Some(folio_model::Background::Colour {
        colour: Colour("swatch:plain".into()),
    });
    let doc = Document::load_mem(&cover_pdf(&book, &styles, Paper::White)).unwrap();
    assert!(ink_colours(&doc).iter().any(|c| c[0] > 0.4 && c[2] < c[0]));
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
