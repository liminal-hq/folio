// Tests for the ways round a PDF: bookmarks, page labels and what the file says about itself
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use folio_export::outline::{Bookmarks, Chapter, Navigation, PdfInfo, ViewerLayout};
use folio_export::pdf::{export_pdf, NoAssets, PdfOptions, PdfTarget};
use folio_layout::{FontBook, StyleSheet};
use folio_model::{Book, BookKind, Numbering, Section};
use folio_presets::{new_book, NewBookSpec};
use lopdf::{Dictionary, Document, Object, ObjectId};

fn sample() -> (Book, StyleSheet) {
    let made = new_book(&NewBookSpec {
        kind: BookKind::Picture,
        ..NewBookSpec::default()
    })
    .unwrap();
    let mut book = made.book;
    // Front matter in roman numerals from the second spread.
    book.spreads[1].section = Some(Section {
        name: "Front".into(),
        numbering: Numbering::Roman,
        start: None,
    });
    book.spreads[2].section = Some(Section {
        name: "Story".into(),
        numbering: Numbering::Arabic,
        start: Some(1),
    });
    let styles = StyleSheet::from_styles(&book.styles);
    (book, styles)
}

fn export(book: &Book, styles: &StyleSheet, target: PdfTarget, navigation: Navigation) -> Document {
    let bytes = export_pdf(
        book,
        styles,
        &FontBook::bundled(),
        &NoAssets,
        &PdfOptions {
            target,
            navigation,
            ..PdfOptions::default()
        },
    )
    .unwrap();
    Document::load_mem(&bytes).unwrap()
}

fn text(object: &Object) -> String {
    match object {
        Object::String(bytes, _) => {
            if bytes.starts_with(&[0xFE, 0xFF]) {
                let units: Vec<u16> = bytes[2..]
                    .chunks(2)
                    .map(|c| u16::from_be_bytes([c[0], c[1]]))
                    .collect();
                String::from_utf16_lossy(&units)
            } else {
                String::from_utf8_lossy(bytes).into_owned()
            }
        }
        _ => String::new(),
    }
}

/// The bookmarks as (title, page number from 1).
fn bookmarks(doc: &Document) -> Vec<(String, u32)> {
    let pages: Vec<(u32, ObjectId)> = doc.get_pages().into_iter().collect();
    let Ok(outlines) = doc.catalog().unwrap().get(b"Outlines") else {
        return Vec::new();
    };
    let root = doc.dereference(outlines).unwrap().1.as_dict().unwrap();
    let mut found = Vec::new();
    let mut next = root.get(b"First").ok().cloned();
    while let Some(entry) = next {
        let dict: &Dictionary = doc.dereference(&entry).unwrap().1.as_dict().unwrap();
        let dest = doc
            .dereference(dict.get(b"Dest").unwrap())
            .unwrap()
            .1
            .as_array()
            .unwrap();
        let page = dest[0].as_reference().unwrap();
        let number = pages.iter().find(|(_, id)| *id == page).unwrap().0;
        found.push((text(dict.get(b"Title").unwrap()), number));
        next = dict.get(b"Next").ok().cloned();
    }
    found
}

fn info(doc: &Document) -> Dictionary {
    doc.trailer
        .get(b"Info")
        .and_then(|i| doc.dereference(i))
        .ok()
        .and_then(|(_, o)| o.as_dict().ok().cloned())
        .unwrap_or_default()
}

#[test]
fn chapters_become_bookmarks_that_lead_to_their_pages() {
    let (book, styles) = sample();
    let chapters = vec![
        Chapter {
            title: "The reeds".into(),
            spread: book.spreads[2].id.0.clone(),
            page: 0,
        },
        Chapter {
            title: "The moon".into(),
            spread: book.spreads[3].id.0.clone(),
            page: 1,
        },
    ];
    let doc = export(
        &book,
        &styles,
        PdfTarget::Sharing,
        Navigation {
            bookmarks: Bookmarks::Chapters(chapters),
            ..Navigation::default()
        },
    );
    let found = bookmarks(&doc);
    assert_eq!(found.len(), 2);
    assert_eq!(found[0].0, "The reeds");
    assert_eq!(found[1].0, "The moon");
    assert!(found[0].1 < found[1].1, "{found:?}");
}

#[test]
fn a_bookmark_for_each_page_or_each_spread() {
    let (book, styles) = sample();
    let pages = export(
        &book,
        &styles,
        PdfTarget::Sharing,
        Navigation {
            bookmarks: Bookmarks::Pages,
            ..Navigation::default()
        },
    );
    assert_eq!(bookmarks(&pages).len(), pages.get_pages().len());
    let spreads = export(
        &book,
        &styles,
        PdfTarget::Sharing,
        Navigation {
            bookmarks: Bookmarks::Spreads,
            ..Navigation::default()
        },
    );
    let n = bookmarks(&spreads).len();
    assert!(n > 0 && n < spreads.get_pages().len(), "{n}");
}

#[test]
fn pages_are_labelled_with_the_numbers_the_book_shows() {
    let (book, styles) = sample();
    let doc = export(
        &book,
        &styles,
        PdfTarget::Sharing,
        Navigation {
            page_labels: true,
            ..Navigation::default()
        },
    );
    let labels = doc.catalog().unwrap().get(b"PageLabels").unwrap();
    let labels = doc.dereference(labels).unwrap().1.as_dict().unwrap();
    let nums = labels.get(b"Nums").unwrap().as_array().unwrap();
    let styles_used: Vec<Vec<u8>> = nums
        .chunks(2)
        .filter_map(|pair| {
            let dict = doc.dereference(&pair[1]).ok()?.1.as_dict().ok()?;
            dict.get(b"S").ok()?.as_name().ok().map(<[u8]>::to_vec)
        })
        .collect();
    assert!(styles_used.iter().any(|s| s == b"r"), "roman front matter");
    assert!(styles_used.iter().any(|s| s == b"D"), "arabic story");
}

#[test]
fn the_file_says_what_it_is_about() {
    let (book, styles) = sample();
    let doc = export(
        &book,
        &styles,
        PdfTarget::Sharing,
        Navigation {
            info: PdfInfo {
                subject: Some("A picture book about an otter".into()),
                keywords: Some(vec!["otter".into(), "bedtime".into()]),
                ..PdfInfo::default()
            },
            viewer_layout: ViewerLayout::Spreads,
            ..Navigation::default()
        },
    );
    let info = info(&doc);
    assert_eq!(
        text(info.get(b"Subject").unwrap()),
        "A picture book about an otter"
    );
    assert!(text(info.get(b"Keywords").unwrap()).contains("bedtime"));
    let layout = doc.catalog().unwrap().get(b"PageLayout").unwrap();
    assert_eq!(layout.as_name().unwrap(), b"TwoPageRight");
}

#[test]
fn a_paperback_file_has_none_of_it_whatever_was_asked() {
    let (book, styles) = sample();
    let asked = Navigation {
        bookmarks: Bookmarks::Pages,
        info: PdfInfo {
            subject: Some("Not for KDP".into()),
            keywords: Some(vec!["nope".into()]),
            ..PdfInfo::default()
        },
        page_labels: true,
        links: true,
        viewer_layout: ViewerLayout::Spreads,
    };
    let doc = export(&book, &styles, PdfTarget::Print, asked);
    let catalog = doc.catalog().unwrap();
    assert!(!catalog.has(b"Outlines"));
    assert!(!catalog.has(b"PageLabels"));
    assert!(!catalog.has(b"PageLayout"));
    let info = info(&doc);
    assert!(!info.has(b"Subject") && !info.has(b"Keywords"));
    assert!(info.has(b"Title"));
}

#[test]
fn a_print_shop_file_keeps_its_bookmarks_and_stays_pdf_x() {
    let (book, styles) = sample();
    let doc = export(
        &book,
        &styles,
        PdfTarget::PrintShop,
        Navigation {
            bookmarks: Bookmarks::Pages,
            page_labels: true,
            ..Navigation::default()
        },
    );
    assert!(!bookmarks(&doc).is_empty());
    assert!(doc.catalog().unwrap().has(b"OutputIntents"));
}
