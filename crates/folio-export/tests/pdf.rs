// Tests for the PDF exporter: every kind of book comes out as a PDF with its pages, text and fonts
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use folio_export::pdf::{export_pdf, NoAssets, PdfOptions, PdfTarget};
use folio_layout::{FontBook, StyleSheet};
use folio_model::{Book, BookKind};
use folio_presets::{new_book, NewBookSpec};
use lopdf::Document;

const KINDS: [BookKind; 6] = [
    BookKind::Picture,
    BookKind::Novel,
    BookKind::Textbook,
    BookKind::Illustrated,
    BookKind::Poetry,
    BookKind::Memoir,
];

fn sample(kind: BookKind) -> (Book, StyleSheet) {
    let made = new_book(&NewBookSpec {
        kind,
        ..NewBookSpec::default()
    })
    .unwrap();
    let styles = StyleSheet::from_styles(&made.book.styles);
    (made.book, styles)
}

fn export(kind: BookKind, target: PdfTarget) -> (Book, Vec<u8>) {
    let (book, styles) = sample(kind);
    let fonts = FontBook::bundled();
    // Laying the pages out loads the fonts, which the export then finds by key.
    let bytes = export_pdf(
        &book,
        &styles,
        &fonts,
        &NoAssets,
        &PdfOptions {
            target,
            ink: None,
            crop_marks: target == PdfTarget::PrintShop,
        },
    )
    .unwrap();
    (book, bytes)
}

#[test]
fn every_kind_of_book_comes_out_as_a_pdf_with_a_page_for_each_page() {
    for kind in KINDS {
        let (book, bytes) = export(kind, PdfTarget::Sharing);
        assert!(bytes.starts_with(b"%PDF-"), "{kind:?}");
        let doc = Document::load_mem(&bytes).unwrap();
        let pages: usize = book.spreads.iter().map(|s| s.pages.len()).sum();
        assert_eq!(doc.get_pages().len(), pages, "{kind:?}");
    }
}

#[test]
fn the_pages_are_the_trim_size_and_the_words_are_shown_as_text() {
    let (book, bytes) = export(BookKind::Picture, PdfTarget::Sharing);
    let doc = Document::load_mem(&bytes).unwrap();
    let (_, &page) = doc.get_pages().iter().nth(1).unwrap();
    let media = doc
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
        .collect::<Vec<_>>();
    assert!((media[2] - book.trim.w.0).abs() < 0.01 && (media[3] - book.trim.h.0).abs() < 0.01);
    // The words are shown with a font, not drawn as shapes, so they stay text.
    let shows: usize = doc
        .get_pages()
        .values()
        .map(|&page| {
            doc.get_and_decode_page_content(page)
                .unwrap()
                .operations
                .iter()
                .filter(|op| matches!(op.operator.as_str(), "Tj" | "TJ"))
                .count()
        })
        .sum();
    assert!(shows > 0, "the book shows text");
}

#[test]
fn every_font_a_book_uses_is_embedded() {
    let (_, bytes) = export(BookKind::Novel, PdfTarget::Sharing);
    let doc = Document::load_mem(&bytes).unwrap();
    let mut fonts = 0;
    let mut embedded = 0;
    for object in doc.objects.values() {
        let Ok(dict) = object.as_dict() else { continue };
        if dict.get(b"Type").and_then(|t| t.as_name()).ok() == Some(b"FontDescriptor") {
            fonts += 1;
            if dict.has(b"FontFile") || dict.has(b"FontFile2") || dict.has(b"FontFile3") {
                embedded += 1;
            }
        }
    }
    assert!(fonts > 0, "the book has text");
    assert_eq!(fonts, embedded, "no font is left out of the file");
}

#[test]
fn the_print_pdf_has_no_bookmarks_and_no_metadata_beyond_the_title_and_author() {
    let (_, bytes) = export(BookKind::Picture, PdfTarget::Print);
    let doc = Document::load_mem(&bytes).unwrap();
    let catalog = doc.catalog().unwrap();
    assert!(!catalog.has(b"Outlines"), "no bookmarks");
    assert!(!catalog.has(b"Metadata"), "no XMP");
    let info = doc
        .trailer
        .get(b"Info")
        .and_then(|i| doc.dereference(i))
        .ok();
    let title = info
        .and_then(|(_, o)| o.as_dict().ok())
        .map(|d| d.has(b"Title"))
        .unwrap_or(false);
    assert!(title, "the title is kept");
}

#[test]
fn a_book_with_nothing_to_draw_is_an_error_not_an_empty_file() {
    let (mut book, styles) = sample(BookKind::Picture);
    book.spreads.clear();
    let fonts = FontBook::bundled();
    assert!(export_pdf(&book, &styles, &fonts, &NoAssets, &PdfOptions::default()).is_err());
}

fn boxes(doc: &Document, page: lopdf::ObjectId, key: &[u8]) -> Option<Vec<f64>> {
    doc.get_object(page)
        .unwrap()
        .as_dict()
        .unwrap()
        .get(key)
        .ok()?
        .as_array()
        .ok()
        .map(|a| a.iter().map(|n| n.as_float().unwrap() as f64).collect())
}

#[test]
fn a_print_pdf_is_the_interior_only_with_an_even_number_of_pages() {
    for kind in KINDS {
        let (book, bytes) = export(kind, PdfTarget::Print);
        let doc = Document::load_mem(&bytes).unwrap();
        let interior: usize = book.spreads.iter().skip(1).map(|s| s.pages.len()).sum();
        let expected = interior + interior % 2;
        assert_eq!(doc.get_pages().len(), expected, "{kind:?}");
        assert_eq!(doc.get_pages().len() % 2, 0, "{kind:?}");
    }
}

#[test]
fn when_anything_bleeds_every_page_has_the_bleed_on_its_outer_edge_and_a_trim_box() {
    let (book, bytes) = export(BookKind::Picture, PdfTarget::Print);
    assert!(book.bleed.0 > 0.0, "the sample picture book has a bleed");
    let doc = Document::load_mem(&bytes).unwrap();
    let (w, h, bleed) = (book.trim.w.0, book.trim.h.0, book.bleed.0);
    let pages: Vec<_> = doc.get_pages().into_values().collect();
    for (index, page) in pages.iter().enumerate() {
        let media = boxes(&doc, *page, b"MediaBox").unwrap();
        assert!((media[2] - (w + bleed)).abs() < 0.01, "page {index} width");
        assert!(
            (media[3] - (h + 2.0 * bleed)).abs() < 0.01,
            "page {index} height"
        );
        let trim = boxes(&doc, *page, b"TrimBox").expect("a trim box");
        // The page after the cover is a left-hand page: its bleed is on the left.
        let left = if index % 2 == 0 { bleed } else { 0.0 };
        assert!(
            (trim[0] - left).abs() < 0.01,
            "page {index} trim x: {trim:?}"
        );
        assert!((trim[2] - trim[0] - w).abs() < 0.01 && (trim[3] - trim[1] - h).abs() < 0.01);
        assert!(boxes(&doc, *page, b"BleedBox").is_some());
    }
}

#[test]
fn a_book_with_no_bleed_gets_pages_at_the_trim_size_and_no_trim_box() {
    let (book, bytes) = export(BookKind::Novel, PdfTarget::Print);
    assert_eq!(book.bleed.0, 0.0);
    let doc = Document::load_mem(&bytes).unwrap();
    let page = *doc.get_pages().values().next().unwrap();
    let media = boxes(&doc, page, b"MediaBox").unwrap();
    assert!((media[2] - book.trim.w.0).abs() < 0.01);
    assert!(boxes(&doc, page, b"TrimBox").is_none());
}

fn string(doc: &Document, dict: &lopdf::Dictionary, key: &[u8]) -> String {
    let object = doc.dereference(dict.get(key).unwrap()).unwrap().1;
    String::from_utf8_lossy(object.as_str().unwrap()).into_owned()
}

#[test]
fn the_print_shop_pdf_is_pdf_x4_with_an_output_intent_the_version_keys_and_an_id() {
    let (_, bytes) = export(BookKind::Picture, PdfTarget::PrintShop);
    let doc = Document::load_mem(&bytes).unwrap();
    let catalog = doc.catalog().unwrap();

    let intents = doc
        .dereference(catalog.get(b"OutputIntents").unwrap())
        .unwrap()
        .1
        .as_array()
        .unwrap();
    assert_eq!(intents.len(), 1);
    let intent = doc.dereference(&intents[0]).unwrap().1.as_dict().unwrap();
    assert_eq!(intent.get(b"S").unwrap().as_name().unwrap(), b"GTS_PDFX");
    assert_eq!(
        string(&doc, intent, b"OutputConditionIdentifier"),
        "sRGB IEC61966-2.1"
    );
    let profile = doc
        .dereference(intent.get(b"DestOutputProfile").unwrap())
        .unwrap()
        .1
        .as_stream()
        .unwrap();
    assert_eq!(profile.dict.get(b"N").unwrap().as_i64().unwrap(), 3);
    assert!(profile.content.len() > 100, "the profile is in the file");

    let info = doc
        .dereference(doc.trailer.get(b"Info").unwrap())
        .unwrap()
        .1
        .as_dict()
        .unwrap();
    assert_eq!(string(&doc, info, b"GTS_PDFXVersion"), "PDF/X-4");
    assert_eq!(info.get(b"Trapped").unwrap().as_name().unwrap(), b"False");
    assert!(doc.trailer.has(b"ID"), "a document ID");

    let xmp = doc
        .dereference(catalog.get(b"Metadata").unwrap())
        .unwrap()
        .1
        .as_stream()
        .unwrap();
    let packet = String::from_utf8_lossy(&xmp.content);
    assert!(
        packet.contains("<pdfxid:GTS_PDFXVersion>PDF/X-4</pdfxid:GTS_PDFXVersion>"),
        "{packet}"
    );
    assert!(packet.contains("xpacket end"));
}

#[test]
fn the_print_shop_pdf_has_every_page_with_the_bleed_all_round_and_room_for_crop_marks() {
    let (book, bytes) = export(BookKind::Picture, PdfTarget::PrintShop);
    let doc = Document::load_mem(&bytes).unwrap();
    let pages: Vec<_> = doc.get_pages().into_values().collect();
    let total: usize = book.spreads.iter().map(|s| s.pages.len()).sum();
    assert_eq!(pages.len(), total, "the cover is in it too");
    let (w, h, bleed, slug) = (book.trim.w.0, book.trim.h.0, book.bleed.0, 24.0);
    for page in pages {
        let media = boxes(&doc, page, b"MediaBox").unwrap();
        assert!((media[2] - (w + 2.0 * (bleed + slug))).abs() < 0.01);
        assert!((media[3] - (h + 2.0 * (bleed + slug))).abs() < 0.01);
        let trim = boxes(&doc, page, b"TrimBox").unwrap();
        assert!((trim[0] - (bleed + slug)).abs() < 0.01 && (trim[1] - (bleed + slug)).abs() < 0.01);
        let bleed_box = boxes(&doc, page, b"BleedBox").unwrap();
        assert!(
            (bleed_box[0] - slug).abs() < 0.01 && (bleed_box[2] - (media[2] - slug)).abs() < 0.01
        );
    }
}

#[test]
fn qpdf_finds_the_files_well_formed() {
    let Ok(_) = std::process::Command::new("qpdf").arg("--version").output() else {
        return; // qpdf is not installed here; the structural tests above still ran.
    };
    let dir = std::env::temp_dir().join(format!("folio-export-qpdf-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    for target in [PdfTarget::Sharing, PdfTarget::Print, PdfTarget::PrintShop] {
        for kind in [BookKind::Picture, BookKind::Novel] {
            let (_, bytes) = export(kind, target);
            let file = dir.join(format!("{kind:?}-{target:?}.pdf"));
            std::fs::write(&file, bytes).unwrap();
            let out = std::process::Command::new("qpdf")
                .arg("--check")
                .arg(&file)
                .output()
                .unwrap();
            assert!(
                out.status.success(),
                "{kind:?} {target:?}: {}",
                String::from_utf8_lossy(&out.stdout)
            );
        }
    }
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_file_of_ink_uses_cmyk_and_black_alone_for_the_text() {
    use folio_export::pdf::InkOptions;
    let (book, styles) = sample(BookKind::Picture);
    let fonts = FontBook::bundled();
    let bytes = export_pdf(
        &book,
        &styles,
        &fonts,
        &NoAssets,
        &PdfOptions {
            target: PdfTarget::Print,
            ink: Some(InkOptions::default()),
            crop_marks: false,
        },
    )
    .unwrap();
    let doc = Document::load_mem(&bytes).unwrap();
    let mut cmyk_fills = 0;
    let mut rgb_fills = 0;
    for page in doc.get_pages().into_values() {
        for op in doc.get_and_decode_page_content(page).unwrap().operations {
            match op.operator.as_str() {
                "k" | "K" => cmyk_fills += 1,
                "rg" | "RG" => rgb_fills += 1,
                _ => {}
            }
        }
    }
    assert!(cmyk_fills > 0, "colours are ink");
    assert_eq!(rgb_fills, 0, "none is left in RGB");
}
