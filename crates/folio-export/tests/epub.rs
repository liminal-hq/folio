// Tests for the EPUB exporter: a well-formed package with the book's story in reading order
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::collections::HashMap;
use std::io::Read;

use folio_export::epub::{export_epub, EpubFlavour, EpubOptions};
use folio_export::pdf::{Assets, NoAssets};
use folio_layout::FontBook;
use folio_model::{Book, BookKind, Frame};
use folio_presets::{new_book, NewBookSpec};
use zip::ZipArchive;

/// A one-pixel PNG, so a picture frame has something to hold.
const PNG: &[u8] = &[
    0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x48, 0x44, 0x52,
    0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1f, 0x15, 0xc4,
    0x89, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9c, 0x63, 0xf8, 0xff, 0xff, 0x3f,
    0x00, 0x05, 0xfe, 0x02, 0xfe, 0xdc, 0xcc, 0x59, 0xe7, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4e,
    0x44, 0xae, 0x42, 0x60, 0x82,
];

struct One(HashMap<String, Vec<u8>>);
impl Assets for One {
    fn bytes(&self, path: &str) -> Option<Vec<u8>> {
        self.0.get(path).cloned()
    }
}

fn sample(kind: BookKind) -> Book {
    new_book(&NewBookSpec {
        kind,
        ..NewBookSpec::default()
    })
    .unwrap()
    .book
}

fn files(bytes: &[u8]) -> (ZipArchive<std::io::Cursor<Vec<u8>>>, Vec<String>) {
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

/// Every well-formed XML file reads to the end without an error.
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

fn epub(book: &Book, assets: &dyn Assets, flavour: EpubFlavour) -> Vec<u8> {
    let fonts = FontBook::bundled();
    export_epub(
        book,
        &fonts,
        assets,
        &EpubOptions {
            flavour,
            embed_fonts: true,
        },
    )
    .unwrap()
}

#[test]
fn the_package_has_its_parts_in_order_and_every_xml_file_is_well_formed() {
    let bytes = epub(&sample(BookKind::Novel), &NoAssets, EpubFlavour::Standard);
    let (mut zip, names) = files(&bytes);
    assert_eq!(names[0], "mimetype", "the media type comes first");
    {
        let entry = zip.by_index(0).unwrap();
        assert_eq!(entry.compression(), zip::CompressionMethod::Stored);
    }
    assert_eq!(read(&mut zip, "mimetype"), "application/epub+zip");
    for name in [
        "META-INF/container.xml",
        "OEBPS/content.opf",
        "OEBPS/nav.xhtml",
        "OEBPS/css/book.css",
    ] {
        assert!(names.iter().any(|n| n == name), "{name}");
    }
    for name in names
        .iter()
        .filter(|n| n.ends_with(".xhtml") || n.ends_with(".opf") || n.ends_with("container.xml"))
    {
        let text = read(&mut zip, name);
        assert!(well_formed(&text), "{name} is not well-formed:\n{text}");
    }
}

#[test]
fn everything_the_package_lists_is_in_it_and_the_spine_names_the_chapters() {
    let bytes = epub(&sample(BookKind::Novel), &NoAssets, EpubFlavour::Standard);
    let (mut zip, names) = files(&bytes);
    let opf = read(&mut zip, "OEBPS/content.opf");
    for href in opf
        .split("href=\"")
        .skip(1)
        .filter_map(|rest| rest.split('"').next())
    {
        assert!(
            names.iter().any(|n| n == &format!("OEBPS/{href}")),
            "the manifest lists {href} but the file is missing"
        );
    }
    assert!(opf.contains("<dc:title>"));
    assert!(opf.contains("<dc:language>en-CA</dc:language>"));
    assert!(opf.contains("properties=\"nav\""));
    assert!(opf.contains("<itemref idref=\"chapter-001\"/>"));
    assert!(opf.contains("dcterms:modified\">20"), "{opf}");
}

#[test]
fn the_story_is_there_as_text_in_reading_order_without_the_page_furniture() {
    let book = sample(BookKind::Picture);
    let bytes = epub(&book, &NoAssets, EpubFlavour::Standard);
    let (mut zip, names) = files(&bytes);
    let chapters: String = names
        .iter()
        .filter(|n| n.contains("/text/chapter-"))
        .map(|n| read(&mut zip, n))
        .collect();
    let first = chapters.find("Down by the reeds").expect("the first words");
    let second = chapters
        .find("Every night the moon rose")
        .expect("the next words");
    assert!(first < second, "reading order");
}

#[test]
fn a_picture_in_the_book_is_stored_listed_and_placed_with_its_description() {
    let mut book = sample(BookKind::Picture);
    for spread in &mut book.spreads {
        for frame in &mut spread.frames {
            if let Frame::Image(image) = frame {
                image.image.asset = Some("assets/images/abc.png".into());
                image.base.description = Some("Pip in the reeds".into());
            }
        }
    }
    let assets = One(HashMap::from([(
        "assets/images/abc.png".into(),
        PNG.to_vec(),
    )]));
    let bytes = epub(&book, &assets, EpubFlavour::Standard);
    let (mut zip, names) = files(&bytes);
    assert!(names.iter().any(|n| n == "OEBPS/images/image-001.png"));
    let opf = read(&mut zip, "OEBPS/content.opf");
    assert!(opf.contains("href=\"images/image-001.png\" media-type=\"image/png\""));
    let chapters: String = names
        .iter()
        .filter(|n| n.contains("/text/chapter-"))
        .map(|n| read(&mut zip, n))
        .collect();
    assert!(chapters.contains("<img src=\"../images/image-001.png\" alt=\"Pip in the reeds\"/>"));
}

#[test]
fn a_kobo_epub_wraps_each_sentence_in_a_numbered_span_and_a_standard_one_does_not() {
    let book = sample(BookKind::Novel);
    let kobo = epub(&book, &NoAssets, EpubFlavour::Kobo);
    let (mut zip, names) = files(&kobo);
    let chapter = names
        .iter()
        .find(|n| n.contains("/text/chapter-001"))
        .unwrap()
        .clone();
    let text = read(&mut zip, &chapter);
    assert!(text.contains("<span class=\"koboSpan\" id=\"kobo.1.1\">"));
    assert!(well_formed(&text));
    let standard = epub(&book, &NoAssets, EpubFlavour::Standard);
    let (mut zip, _) = files(&standard);
    assert!(!read(&mut zip, &chapter).contains("koboSpan"));
}

#[test]
fn embeddable_fonts_travel_with_the_file_and_the_css_names_them() {
    let bytes = epub(&sample(BookKind::Novel), &NoAssets, EpubFlavour::Standard);
    let (mut zip, names) = files(&bytes);
    assert!(names.iter().any(|n| n.starts_with("OEBPS/fonts/")));
    assert!(read(&mut zip, "OEBPS/css/book.css").contains("@font-face"));
    let opf = read(&mut zip, "OEBPS/content.opf");
    assert!(opf.contains("media-type=\"font/ttf\"") || opf.contains("media-type=\"font/otf\""));
}

#[test]
fn a_book_with_no_story_is_an_error() {
    let mut book = sample(BookKind::Novel);
    book.spreads.truncate(1);
    let fonts = FontBook::bundled();
    assert!(export_epub(&book, &fonts, &NoAssets, &EpubOptions::default()).is_err());
}

#[test]
fn a_novels_story_is_told_once_in_the_epub_though_it_runs_through_three_frames() {
    let book = sample(BookKind::Novel);
    assert_eq!(book.flows.len(), 1, "the sample novel pours into one flow");
    let bytes = epub(&book, &NoAssets, EpubFlavour::Standard);
    let (mut zip, names) = files(&bytes);
    let mut all = String::new();
    for name in names
        .iter()
        .filter(|n| n.ends_with(".xhtml") && !n.ends_with("nav.xhtml"))
    {
        all.push_str(&read(&mut zip, name));
    }
    for words in [
        "Mara had kept the light",
        "She did not open it straight away",
        "Inside was a single line",
    ] {
        assert_eq!(all.matches(words).count(), 1, "{words}");
    }
}
