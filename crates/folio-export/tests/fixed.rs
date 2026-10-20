// Tests for the fixed-layout EPUB: a page for each page, the words where layout put them
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::io::Read;

use folio_export::fixed::export_fixed_epub;
use folio_export::pdf::NoAssets;
use folio_layout::{FontBook, StyleSheet};
use folio_model::{Book, BookKind};
use folio_presets::{new_book, NewBookSpec};
use zip::ZipArchive;

fn sample(kind: BookKind) -> (Book, Vec<u8>) {
    let made = new_book(&NewBookSpec {
        kind,
        ..NewBookSpec::default()
    })
    .unwrap();
    let styles = StyleSheet::from_styles(&made.book.styles);
    let bytes = export_fixed_epub(&made.book, &styles, &FontBook::bundled(), &NoAssets).unwrap();
    (made.book, bytes)
}

fn open(bytes: &[u8]) -> (ZipArchive<std::io::Cursor<Vec<u8>>>, Vec<String>) {
    let mut zip = ZipArchive::new(std::io::Cursor::new(bytes.to_vec())).unwrap();
    let names = (0..zip.len())
        .map(|i| zip.by_index(i).unwrap().name().to_string())
        .collect();
    (zip, names)
}

fn read(zip: &mut ZipArchive<std::io::Cursor<Vec<u8>>>, name: &str) -> String {
    let mut text = String::new();
    zip.by_name(name)
        .unwrap()
        .read_to_string(&mut text)
        .unwrap();
    text
}

fn well_formed(text: &str) -> bool {
    let mut reader = quick_xml::Reader::from_str(text);
    loop {
        match reader.read_event() {
            Ok(quick_xml::events::Event::Eof) => return true,
            Err(_) => return false,
            _ => {}
        }
    }
}

#[test]
fn there_is_an_xhtml_page_for_every_page_and_every_xml_file_is_well_formed() {
    let (book, bytes) = sample(BookKind::Picture);
    let (mut zip, names) = open(&bytes);
    assert_eq!(names[0], "mimetype");
    let pages: usize = book.spreads.iter().map(|s| s.pages.len()).sum();
    let xhtml: Vec<_> = names.iter().filter(|n| n.contains("/text/page-")).collect();
    assert_eq!(xhtml.len(), pages);
    for name in names
        .iter()
        .filter(|n| n.ends_with(".xhtml") || n.ends_with(".opf"))
    {
        assert!(well_formed(&read(&mut zip, name)), "{name}");
    }
}

#[test]
fn the_package_says_the_pages_are_fixed_and_which_side_each_faces() {
    let (book, bytes) = sample(BookKind::Picture);
    let (mut zip, _) = open(&bytes);
    let opf = read(&mut zip, "OEBPS/content.opf");
    assert!(opf.contains("<meta property=\"rendition:layout\">pre-paginated</meta>"));
    assert!(opf.contains("rendition:spread"));
    assert!(opf.contains("page-spread-left") && opf.contains("page-spread-right"));
    assert!(opf.contains("properties=\"cover-image\""));
    let first = read(&mut zip, "OEBPS/text/page-001.xhtml");
    assert!(first.contains(&format!(
        "name=\"viewport\" content=\"width={}, height={}\"",
        book.trim.w.0, book.trim.h.0
    )));
}

#[test]
fn the_words_are_live_text_placed_where_layout_put_them_in_the_fonts_they_were_set_in() {
    let (_, bytes) = sample(BookKind::Picture);
    let (mut zip, names) = open(&bytes);
    let pages: String = names
        .iter()
        .filter(|n| n.contains("/text/page-"))
        .map(|n| read(&mut zip, n))
        .collect();
    assert!(pages.contains("reeds"));
    assert!(pages.contains("position:absolute;left:"));
    assert!(pages.contains("white-space:pre"));
    let css = read(&mut zip, "OEBPS/css/book.css");
    assert!(css.contains("@font-face"));
    assert!(names.iter().any(|n| n.starts_with("OEBPS/fonts/")));
    // The cover picture a store shows is in the package too.
    assert!(names.iter().any(|n| n == "OEBPS/images/cover.jpg"));
}

#[test]
fn every_kind_of_book_makes_a_fixed_epub() {
    for kind in [
        BookKind::Picture,
        BookKind::Textbook,
        BookKind::Illustrated,
        BookKind::Poetry,
        BookKind::Memoir,
        BookKind::Novel,
    ] {
        let (_, bytes) = sample(kind);
        assert!(bytes.starts_with(b"PK"), "{kind:?}");
    }
}
