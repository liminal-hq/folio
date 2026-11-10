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
    // The cover page is the cover document, the first of them, and the rest are pages.
    let xhtml: Vec<_> = names.iter().filter(|n| n.contains("/text/")).collect();
    assert_eq!(xhtml.len(), pages);
    assert!(names.iter().any(|n| n == "OEBPS/text/cover.xhtml"));
    assert!(!names.iter().any(|n| n == "OEBPS/text/page-001.xhtml"));
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
    assert!(opf.contains(&format!(
        "<meta name=\"original-resolution\" content=\"{}x{}\"/>",
        book.trim.w.0.round(),
        book.trim.h.0.round()
    )));
    assert!(opf.contains("rendition:spread"));
    assert!(opf.contains("page-spread-left") && opf.contains("page-spread-right"));
    assert!(opf.contains("properties=\"cover-image\""));
    let first = read(&mut zip, "OEBPS/text/page-002.xhtml");
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

#[test]
fn a_line_is_one_positioned_box_of_running_text_with_its_spaces_and_nothing_placed_inside_it() {
    let (_, bytes) = sample(BookKind::Picture);
    let (mut zip, names) = open(&bytes);
    let pages: String = names
        .iter()
        .filter(|n| n.contains("/text/page-"))
        .map(|n| read(&mut zip, n))
        .collect();
    let lines: Vec<&str> = pages
        .split("<div class=\"ln\"")
        .skip(1)
        .map(|l| l.split("</div>").next().unwrap_or(""))
        .collect();
    assert!(!lines.is_empty());
    // Words are not each placed at a point, which a reader that sets them wider runs together.
    assert!(lines
        .iter()
        .all(|l| !l.contains("left:") || l.starts_with(" style=\"position:absolute")));
    assert!(lines
        .iter()
        .all(|l| l.matches("position:absolute").count() <= 1));
    // The words of a line are in the text with spaces between them.
    let spoken: Vec<String> = lines
        .iter()
        .map(|l| {
            let mut text = String::new();
            let mut tag = false;
            for c in l.split_once('>').map_or("", |x| x.1).chars() {
                match c {
                    '<' => tag = true,
                    '>' => tag = false,
                    c if !tag => text.push(c),
                    _ => {}
                }
            }
            text
        })
        .collect();
    assert!(spoken.iter().any(|t| t.trim().contains(' ')), "{spoken:?}");
}

#[test]
fn the_fonts_are_declared_at_the_weight_they_are_with_the_kind_of_file_they_are() {
    let (_, bytes) = sample(BookKind::Picture);
    let (mut zip, _) = open(&bytes);
    let css = read(&mut zip, "OEBPS/css/book.css");
    assert!(css.contains("@font-face"));
    assert!(!css.contains("font-weight:100 900"), "{css}");
    for face in css.lines().filter(|l| l.starts_with("@font-face")) {
        assert!(
            face.contains("format('truetype')") || face.contains("format('opentype')"),
            "{face}"
        );
        assert!(face.contains("font-style:normal") || face.contains("font-style:italic"));
    }
}

#[test]
fn the_first_document_is_the_cover_picture_as_a_reflowable_page_and_every_reader_is_told_so() {
    let (_, bytes) = sample(BookKind::Picture);
    let (mut zip, _) = open(&bytes);
    let opf = read(&mut zip, "OEBPS/content.opf");
    let spine = opf.split("<spine>").nth(1).unwrap();
    assert!(
        spine.trim_start().starts_with("<itemref idref=\"cover\""),
        "{spine}"
    );
    assert!(opf.contains("<item id=\"cover\" href=\"text/cover.xhtml\""));
    // EPUB 2 readers and Amazon look for these.
    assert!(opf.contains("<meta name=\"cover\" content=\"cover-image\"/>"));
    assert!(opf.contains("<reference type=\"cover\" title=\"Cover\" href=\"text/cover.xhtml\"/>"));
    let nav = read(&mut zip, "OEBPS/nav.xhtml");
    assert!(nav.contains("epub:type=\"landmarks\"") && nav.contains("epub:type=\"cover\""));
    let cover = read(&mut zip, "OEBPS/text/cover.xhtml");
    // A reflowable page in the fixed book, which a Kobo scales to the screen; a fixed one is cropped.
    assert!(!cover.contains("viewport"));
    assert!(spine.contains("rendition:layout-reflowable"));
    assert!(cover.contains("max-width:100%"));
    assert!(cover.contains("src=\"../images/cover.jpg\""));
}

#[test]
fn the_cover_picture_is_the_size_a_store_asks_for() {
    let (_, bytes) = sample(BookKind::Picture);
    let (mut zip, _) = open(&bytes);
    let mut cover = zip.by_name("OEBPS/images/cover.jpg").unwrap();
    let mut jpeg = Vec::new();
    cover.read_to_end(&mut jpeg).unwrap();
    let (w, h) = image::ImageReader::new(std::io::Cursor::new(jpeg))
        .with_guessed_format()
        .unwrap()
        .into_dimensions()
        .unwrap();
    assert_eq!((w, h), (1600, 2560));
}

#[test]
fn underlined_and_struck_words_are_set_with_text_decoration() {
    let (mut book, _) = sample(BookKind::Picture);
    for frame in book.spreads.iter_mut().flat_map(|s| s.frames.iter_mut()) {
        if let folio_model::Frame::Text(t) = frame {
            for run in t.content.iter_mut().flat_map(|c| c.inline.iter_mut()) {
                run.underline = true;
                run.strike = true;
            }
        }
    }
    let styles = StyleSheet::from_styles(&book.styles);
    let bytes = export_fixed_epub(&book, &styles, &FontBook::bundled(), &NoAssets).unwrap();
    let (mut zip, names) = open(&bytes);
    let pages: String = names
        .iter()
        .filter(|n| n.contains("/text/page-"))
        .map(|n| read(&mut zip, n))
        .collect();
    assert!(pages.contains("text-decoration:underline line-through"));
}

#[test]
fn words_in_another_language_are_spans_with_it() {
    let (mut book, _) = sample(BookKind::Picture);
    for frame in book.spreads.iter_mut().flat_map(|s| s.frames.iter_mut()) {
        if let folio_model::Frame::Text(t) = frame {
            for run in t.content.iter_mut().flat_map(|c| c.inline.iter_mut()) {
                run.lang = Some("de".into());
            }
        }
    }
    let styles = StyleSheet::from_styles(&book.styles);
    let bytes = export_fixed_epub(&book, &styles, &FontBook::bundled(), &NoAssets).unwrap();
    let (mut zip, names) = open(&bytes);
    let pages: String = names
        .iter()
        .filter(|n| n.contains("/text/page-"))
        .map(|n| read(&mut zip, n))
        .collect();
    assert!(pages.contains("<span lang=\"de\" xml:lang=\"de\" style="));
    for name in names.iter().filter(|n| n.contains("/text/page-")) {
        assert!(well_formed(&read(&mut zip, name)), "{name}");
    }
}

#[test]
fn linked_words_are_anchors_and_a_link_to_a_spread_goes_to_the_page_it_begins_on() {
    let (mut book, _) = sample(BookKind::Picture);
    let spreads = book.spreads.len();
    let mut set = 0;
    for frame in book.spreads.iter_mut().flat_map(|s| s.frames.iter_mut()) {
        if let folio_model::Frame::Text(t) = frame {
            for run in t.content.iter_mut().flat_map(|c| c.inline.iter_mut()) {
                run.href = Some(if set % 2 == 0 {
                    "https://example.org/?a=1&b=2".to_string()
                } else {
                    format!("#spread-{spreads}")
                });
                set += 1;
            }
        }
    }
    assert!(set > 1);
    let styles = StyleSheet::from_styles(&book.styles);
    let bytes = export_fixed_epub(&book, &styles, &FontBook::bundled(), &NoAssets).unwrap();
    let (mut zip, names) = open(&bytes);
    let pages: Vec<(String, String)> = names
        .iter()
        .filter(|n| n.contains("/text/page-"))
        .map(|n| (n.clone(), read(&mut zip, n)))
        .collect();
    let all: String = pages.iter().map(|(_, t)| t.as_str()).collect();
    assert!(all.contains("<a href=\"https://example.org/?a=1&amp;b=2\"><span"));
    // The last spread begins on a page of the book, and the link names that page's file.
    let internal: Vec<&str> = all
        .split("<a href=\"")
        .skip(1)
        .filter_map(|r| r.split('"').next())
        .filter(|h| h.starts_with("page-") || h.starts_with("cover"))
        .collect();
    assert!(!internal.is_empty());
    for href in internal {
        assert!(
            names.iter().any(|n| n.ends_with(&format!("/text/{href}"))),
            "{href} is not a file of the book"
        );
    }
    for (name, text) in &pages {
        assert!(well_formed(text), "{name}");
    }
}
