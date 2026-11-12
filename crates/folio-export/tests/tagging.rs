// Tests for tagged PDFs: the structure tree, the artifacts, the text a reader gets and the pixels
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

use folio_export::outline::Bookmarks;
use folio_export::outline::Navigation;
use folio_export::pages::{CoverPages, PageChoice};
use folio_export::pdf::{export_pdf, export_pdf_noted, Assets, PdfOptions, PdfTarget};
use folio_export::picture::{ImagePolicy, PictureCache};
use folio_layout::{FontBook, StyleSheet};
use folio_model::{
    Book, BookKind, Fit, Frame, FrameBase, FrameId, ImageFrame, ImageRef, InlineContent, Pt, Rect,
    Run, SpreadKind, StyleId, TextFrame,
};
use folio_presets::{new_book, NewBookSpec, Words};
use image::{ImageEncoder, Rgb, RgbImage};
use lopdf::{Dictionary, Document, Object};

struct Memory(HashMap<String, Vec<u8>>);

impl Assets for Memory {
    fn bytes(&self, path: &str) -> Option<Vec<u8>> {
        self.0.get(path).cloned()
    }
}

const ALT: &str = "A red square with a blue border";
const SENTENCE: &str = "The Zebrafish swam past officially difficult flowers in the afternoon \
sunlight while everybody watched and wondered where the pumpkins had gone.";

fn png() -> Vec<u8> {
    let mut image = RgbImage::from_pixel(120, 120, Rgb([200, 30, 30]));
    for i in 0..120 {
        for edge in [0, 1, 2, 117, 118, 119] {
            image.put_pixel(i, edge, Rgb([20, 30, 200]));
            image.put_pixel(edge, i, Rgb([20, 30, 200]));
        }
    }
    let mut out = Vec::new();
    image::codecs::png::PngEncoder::new(&mut out)
        .write_image(image.as_raw(), 120, 120, image::ExtendedColorType::Rgb8)
        .unwrap();
    out
}

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

fn words(id: &str, style: &str, x: f64, y: f64, w: f64, h: f64, runs: Vec<Run>) -> Frame {
    Frame::Text(TextFrame {
        base: base(id, x, y, w, h),
        style: StyleId(style.into()),
        content: Some(InlineContent {
            inline: runs,
            looks: None,
        }),
        flow: None,
        columns: 1,
        baseline_lock: false,
        overrides: None,
    })
}

fn plain(text: &str) -> Run {
    Run {
        text: text.into(),
        ..Default::default()
    }
}

fn picture(id: &str, x: f64, y: f64, w: f64, h: f64, alt: Option<&str>) -> Frame {
    let mut base = base(id, x, y, w, h);
    base.description = alt.map(str::to_owned);
    Frame::Image(ImageFrame {
        base,
        image: ImageRef {
            asset: Some("square.png".into()),
            fit: Fit::Fill,
            focus: (0.5, 0.5),
            crop: None,
        },
        placeholder_label: None,
    })
}

/// A picture book with a cover and a back, a described picture, page numbers in front of
/// pumpkins, a heading, a link and a long justified paragraph that hyphenates.
fn storybook() -> (Book, StyleSheet, Memory) {
    let made = new_book(&NewBookSpec {
        kind: BookKind::Picture,
        words: Words::Samples,
        ..NewBookSpec::default()
    })
    .unwrap();
    let mut book = made.book;
    book.trim.w = Pt(432.0);
    book.trim.h = Pt(432.0);
    book.metadata.isbn.paperback = Some("978-0-306-40615-7".into());
    let template = book
        .spreads
        .iter()
        .find(|s| s.kind == SpreadKind::Spread)
        .unwrap()
        .clone();
    let cover = book.spreads.first().cloned().unwrap();
    book.spreads = vec![cover];
    for i in 0..3 {
        let mut spread = template.clone();
        spread.id = folio_model::SpreadId(format!("sp_t{i}"));
        for (p, page) in spread.pages.iter_mut().enumerate() {
            page.id = folio_model::PageId(format!("pg_t{i}_{p}"));
            page.master = None;
        }
        let mut frames = vec![
            picture(&format!("art{i}"), 30.0, 30.0, 372.0, 200.0, Some(ALT)),
            words(
                &format!("tx{i}"),
                "body",
                452.0,
                40.0,
                340.0,
                300.0,
                vec![plain(SENTENCE)],
            ),
        ];
        for side in 0..2 {
            let x = 186.0 + side as f64 * 432.0;
            frames.push(picture(&format!("pk{i}{side}"), x, 372.0, 60.0, 48.0, None));
            frames.push(words(
                &format!("pn{i}{side}"),
                "body",
                x,
                385.0,
                60.0,
                22.0,
                vec![plain(&format!("{}", i * 2 + side + 1))],
            ));
        }
        if i == 0 {
            frames.push(words(
                "head",
                "display",
                30.0,
                240.0,
                372.0,
                60.0,
                vec![plain("Chapter One")],
            ));
            frames.push(words(
                "link",
                "body",
                30.0,
                320.0,
                372.0,
                30.0,
                vec![
                    plain("Read more at "),
                    Run {
                        text: "the website".into(),
                        href: Some("https://example.org/otters".into()),
                        ..Default::default()
                    },
                    plain(" or go back to the "),
                    Run {
                        text: "start".into(),
                        href: Some("#spread-2".into()),
                        ..Default::default()
                    },
                ],
            ));
        }
        spread.frames = frames;
        book.spreads.push(spread);
    }
    let styles = StyleSheet::from_styles(&book.styles);
    let assets = Memory(HashMap::from([("square.png".to_owned(), png())]));
    (book, styles, assets)
}

fn options(tagged: bool, target: PdfTarget) -> PdfOptions {
    PdfOptions {
        target,
        navigation: Navigation {
            tagged,
            ..Navigation::default()
        },
        pages: PageChoice {
            cover: CoverPages::FrontAndBackEnds,
            ..PageChoice::default()
        },
        crop_marks: target == PdfTarget::PrintShop,
        ..PdfOptions::default()
    }
}

fn export(book: &Book, styles: &StyleSheet, assets: &Memory, o: &PdfOptions) -> Vec<u8> {
    export_pdf(book, styles, &FontBook::bundled(), assets, o).unwrap()
}

/// The structure elements of a file in document order: their type and dictionary.
fn elements(doc: &Document) -> Vec<(String, Dictionary)> {
    fn walk(doc: &Document, object: &Object, out: &mut Vec<(String, Dictionary)>) {
        match object {
            Object::Reference(id) => {
                if let Ok(o) = doc.get_object(*id) {
                    walk(doc, o, out);
                }
            }
            Object::Array(items) => items.iter().for_each(|i| walk(doc, i, out)),
            Object::Dictionary(d) => {
                if let Ok(kind) = d.get(b"S").and_then(Object::as_name) {
                    out.push((String::from_utf8_lossy(kind).into_owned(), d.clone()));
                }
                if let Ok(kids) = d.get(b"K") {
                    walk(doc, kids, out);
                }
            }
            _ => {}
        }
    }
    let root = doc
        .catalog()
        .unwrap()
        .get(b"StructTreeRoot")
        .expect("a structure tree");
    let mut out = Vec::new();
    walk(doc, root, &mut out);
    out
}

fn text_of(doc: &Document, dict: &Dictionary, key: &[u8]) -> Option<String> {
    let object = dict.get(key).ok()?;
    let object = match object {
        Object::Reference(id) => doc.get_object(*id).ok()?,
        other => other,
    };
    match object {
        Object::String(bytes, _) => {
            if bytes.starts_with(&[0xFE, 0xFF]) {
                let units: Vec<u16> = bytes[2..]
                    .chunks(2)
                    .map(|c| u16::from_be_bytes([c[0], *c.get(1).unwrap_or(&0)]))
                    .collect();
                Some(String::from_utf16_lossy(&units))
            } else {
                Some(String::from_utf8_lossy(bytes).into_owned())
            }
        }
        _ => None,
    }
}

#[test]
fn a_tagged_file_has_a_structure_tree_a_language_and_a_title_that_the_viewer_shows() {
    let (book, styles, assets) = storybook();
    let bytes = export(&book, &styles, &assets, &options(true, PdfTarget::Sharing));
    let doc = Document::load_mem(&bytes).unwrap();
    let catalog = doc.catalog().unwrap();
    let mark = catalog.get(b"MarkInfo").unwrap().as_dict().unwrap();
    assert!(mark.get(b"Marked").unwrap().as_bool().unwrap());
    assert!(
        catalog.has(b"Lang"),
        "the document says what language it is in"
    );
    let prefs = catalog
        .get(b"ViewerPreferences")
        .unwrap()
        .as_dict()
        .unwrap();
    assert!(prefs.get(b"DisplayDocTitle").unwrap().as_bool().unwrap());
    let kinds: Vec<String> = elements(&doc).into_iter().map(|(k, _)| k).collect();
    for want in ["Document", "H1", "P", "Figure", "Link"] {
        assert!(kinds.iter().any(|k| k == want), "{want} in {kinds:?}");
    }
    // Every page has its structure parents.
    for &page in doc.get_pages().values() {
        let dict = doc.get_object(page).unwrap().as_dict().unwrap();
        assert!(dict.has(b"StructParents"), "a page's marks are in the tree");
    }
}

#[test]
fn an_untagged_file_has_no_structure_tree() {
    let (book, styles, assets) = storybook();
    let bytes = export(&book, &styles, &assets, &options(false, PdfTarget::Sharing));
    let doc = Document::load_mem(&bytes).unwrap();
    assert!(!doc.catalog().unwrap().has(b"StructTreeRoot"));
}

#[test]
fn a_picture_is_a_figure_with_its_description_and_the_page_numbers_and_pumpkins_are_artifacts() {
    let (book, styles, assets) = storybook();
    let bytes = export(&book, &styles, &assets, &options(true, PdfTarget::Sharing));
    let doc = Document::load_mem(&bytes).unwrap();
    let alts: Vec<String> = elements(&doc)
        .into_iter()
        .filter(|(kind, _)| kind == "Figure")
        .filter_map(|(_, d)| text_of(&doc, &d, b"Alt"))
        .collect();
    // Three pictures with a description, and the cover's picture if it has one: the pumpkins are not
    // among them.
    assert_eq!(
        alts.iter().filter(|a| a.as_str() == ALT).count(),
        3,
        "{alts:?}"
    );
    assert!(alts.iter().all(|a| !a.is_empty()));
    let figures = elements(&doc)
        .iter()
        .filter(|(kind, _)| kind == "Figure")
        .count();
    assert_eq!(figures, alts.len(), "every figure has a description here");
    // The numbers are in no paragraph, and page content marks artifacts.
    let paragraph_text: Vec<String> = elements(&doc)
        .iter()
        .filter(|(kind, _)| kind == "P")
        .filter_map(|(_, d)| text_of(&doc, d, b"ActualText"))
        .collect();
    assert!(paragraph_text.iter().all(|t| t != "1" && t != "2"));
    let mut artifacts = 0;
    for &page in doc.get_pages().values() {
        let content = doc.get_and_decode_page_content(page).unwrap();
        artifacts += content
            .operations
            .iter()
            .filter(|op| op.operator == "BDC" || op.operator == "BMC")
            .filter(|op| {
                op.operands
                    .first()
                    .and_then(|o| o.as_name().ok())
                    .is_some_and(|n| n == b"Artifact")
            })
            .count();
    }
    assert!(artifacts >= 12, "{artifacts} artifacts");
}

#[test]
fn a_link_is_a_link_element_that_holds_its_words_and_its_annotation() {
    let (book, styles, assets) = storybook();
    let bytes = export(&book, &styles, &assets, &options(true, PdfTarget::Sharing));
    let doc = Document::load_mem(&bytes).unwrap();
    let links: Vec<Dictionary> = elements(&doc)
        .into_iter()
        .filter(|(kind, _)| kind == "Link")
        .map(|(_, d)| d)
        .collect();
    assert_eq!(links.len(), 2, "one for each link in the words");
    // Each has an object reference to its annotation.
    let mut objr = 0;
    for link in &links {
        let kids = match link.get(b"K").unwrap() {
            Object::Array(items) => items.clone(),
            other => vec![other.clone()],
        };
        for kid in kids {
            let kid = match kid {
                Object::Reference(id) => doc.get_object(id).unwrap().clone(),
                other => other,
            };
            if let Object::Dictionary(d) = kid {
                if d.get(b"Type").and_then(Object::as_name).ok() == Some(b"OBJR") {
                    objr += 1;
                }
            }
        }
    }
    assert_eq!(objr, links.len());
    // And each annotation says where it goes.
    let mut said = Vec::new();
    for object in doc.objects.values() {
        let Object::Dictionary(d) = object else {
            continue;
        };
        if d.get(b"Subtype").and_then(Object::as_name).ok() == Some(b"Link") {
            said.push(text_of(&doc, d, b"Contents").unwrap_or_default());
        }
    }
    said.sort();
    assert_eq!(
        said,
        vec![
            "Go to page 2".to_string(),
            "Link to https://example.org/otters".to_string()
        ]
    );
}

#[test]
fn the_print_shop_file_keeps_its_structure_tree_and_title_through_the_pdf_x_step() {
    let (book, styles, assets) = storybook();
    let bytes = export(
        &book,
        &styles,
        &assets,
        &options(true, PdfTarget::PrintShop),
    );
    let doc = Document::load_mem(&bytes).unwrap();
    let catalog = doc.catalog().unwrap();
    assert!(catalog.has(b"OutputIntents"), "still PDF/X-4");
    assert!(catalog.has(b"StructTreeRoot"));
    assert!(catalog.has(b"MarkInfo"));
    let prefs = catalog
        .get(b"ViewerPreferences")
        .unwrap()
        .as_dict()
        .unwrap();
    assert!(prefs.get(b"DisplayDocTitle").unwrap().as_bool().unwrap());
    // The crop marks are artifacts, so nothing on a page is untagged content.
    let untagged = options(false, PdfTarget::PrintShop);
    let plain = export(&book, &styles, &assets, &untagged);
    assert!(!Document::load_mem(&plain)
        .unwrap()
        .catalog()
        .unwrap()
        .has(b"StructTreeRoot"));
}

fn tool(name: &str) -> bool {
    Command::new(name).arg("-v").output().is_ok() || Command::new(name).arg("-h").output().is_ok()
}

fn scratch(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("folio-tagging-{}-{name}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn tagging_changes_no_pixel() {
    if !tool("pdftoppm") {
        eprintln!("skipped: pdftoppm is not installed");
        return;
    }
    let (book, styles, assets) = storybook();
    let dir = scratch("pixels");
    let render = |tagged: bool| -> Vec<Vec<u8>> {
        let bytes = export(
            &book,
            &styles,
            &assets,
            &options(tagged, PdfTarget::Sharing),
        );
        let pdf = dir.join(format!("{tagged}.pdf"));
        std::fs::write(&pdf, bytes).unwrap();
        let prefix = dir.join(format!("{tagged}"));
        let status = Command::new("pdftoppm")
            .args(["-r", "100", "-png"])
            .arg(&pdf)
            .arg(&prefix)
            .status()
            .unwrap();
        assert!(status.success());
        let mut pages: Vec<_> = std::fs::read_dir(&dir)
            .unwrap()
            .map(|e| e.unwrap().path())
            .filter(|p| {
                p.file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(|n| n.starts_with(&format!("{tagged}-")) && n.ends_with(".png"))
            })
            .collect();
        pages.sort();
        pages.iter().map(|p| std::fs::read(p).unwrap()).collect()
    };
    let before = render(false);
    let after = render(true);
    // The cover, three spreads (two pages each) and the back cover.
    assert_eq!(before.len(), 1 + 6 + 1);
    assert_eq!(before.len(), after.len());
    for (page, (a, b)) in before.iter().zip(&after).enumerate() {
        let (a, b) = (
            image::load_from_memory(a).unwrap(),
            image::load_from_memory(b).unwrap(),
        );
        assert_eq!(a.as_bytes(), b.as_bytes(), "page {} differs", page + 1);
    }
    let _ = std::fs::remove_dir_all(dir);
}

/// What a reader gets from a page: the words with their spacing made plain, a word cut at the end
/// of a line put back together.
fn extracted(pdf: &Path, page: usize) -> String {
    let out = Command::new("pdftotext")
        .args(["-f", &page.to_string(), "-l", &page.to_string(), "-raw"])
        .arg(pdf)
        .arg("-")
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&out.stdout).replace("\u{AD}\n", "");
    let text = text.replace("-\n", "");
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[test]
fn the_text_a_reader_gets_is_the_text_of_the_book_with_its_spaces() {
    if !tool("pdftotext") {
        eprintln!("skipped: pdftotext is not installed");
        return;
    }
    let (mut book, styles, assets) = storybook();
    // One page with only the paragraph on it, narrow enough to be justified and hyphenated.
    let at = book.spreads.iter().position(|s| s.id.0 == "sp_t1").unwrap();
    book.spreads[at].frames = vec![words(
        "only",
        "body",
        452.0,
        40.0,
        200.0,
        340.0,
        vec![plain(SENTENCE)],
    )];
    let want = SENTENCE.split_whitespace().collect::<Vec<_>>().join(" ");
    let dir = scratch("text");
    for tagged in [false, true] {
        let bytes = export(
            &book,
            &styles,
            &assets,
            &options(tagged, PdfTarget::Sharing),
        );
        let pdf = dir.join(format!("{tagged}.pdf"));
        std::fs::write(&pdf, bytes).unwrap();
        // The cover, then the first spread (two pages), then this spread's two.
        let page = 1 + 2 + 2;
        assert_eq!(extracted(&pdf, page), want, "tagged: {tagged}");
    }
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn qpdf_finds_a_tagged_file_well_formed() {
    if !tool("qpdf") {
        eprintln!("skipped: qpdf is not installed");
        return;
    }
    let (book, styles, assets) = storybook();
    let dir = scratch("qpdf");
    for target in [PdfTarget::Sharing, PdfTarget::PrintShop] {
        let bytes = export(&book, &styles, &assets, &options(true, target));
        let pdf = dir.join(format!("{target:?}.pdf"));
        std::fs::write(&pdf, bytes).unwrap();
        let out = Command::new("qpdf")
            .arg("--check")
            .arg(&pdf)
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "{target:?}: {}",
            String::from_utf8_lossy(&out.stdout)
        );
    }
    let _ = std::fs::remove_dir_all(dir);
}

fn with_bookmarks(mut o: PdfOptions) -> PdfOptions {
    o.navigation.bookmarks = Bookmarks::Pages;
    o
}

fn xmp(doc: &Document) -> String {
    let catalog = doc.catalog().unwrap();
    let Ok(Object::Reference(id)) = catalog.get(b"Metadata") else {
        return String::new();
    };
    let stream = doc.get_object(*id).unwrap().as_stream().unwrap();
    String::from_utf8_lossy(
        &stream
            .decompressed_content()
            .unwrap_or_else(|_| stream.content.clone()),
    )
    .into_owned()
}

fn noted(
    book: &Book,
    styles: &StyleSheet,
    assets: &Memory,
    o: &PdfOptions,
) -> (Vec<u8>, Vec<String>) {
    export_pdf_noted(
        book,
        styles,
        &FontBook::bundled(),
        assets,
        o,
        &ImagePolicy::standard(),
        &PictureCache::default(),
    )
    .unwrap()
}

#[test]
fn a_book_that_qualifies_says_it_is_pdf_ua_and_one_that_does_not_says_why_and_stays_tagged() {
    let (mut book, styles, assets) = storybook();
    // The cover's own picture, if the template has one, is described too.
    for frame in book.spreads[0].frames.iter_mut() {
        if matches!(frame, Frame::Image(_)) && frame.base().description.is_none() {
            frame.base_mut().description = Some("The cover".into());
        }
    }
    let ua = |bytes: &[u8]| xmp(&Document::load_mem(bytes).unwrap()).contains("pdfuaid");
    let (good, notes) = noted(
        &book,
        &styles,
        &assets,
        &with_bookmarks(options(true, PdfTarget::Sharing)),
    );
    assert!(
        ua(&good),
        "a book with a title, bookmarks and every picture described qualifies"
    );
    assert!(notes.is_empty(), "{notes:?}");

    // A picture with no description keeps the file from the claim, and the note says so.
    book.spreads[2]
        .frames
        .push(picture("bare", 30.0, 300.0, 100.0, 60.0, None));
    let (bare, notes) = noted(
        &book,
        &styles,
        &assets,
        &with_bookmarks(options(true, PdfTarget::Sharing)),
    );
    assert!(!ua(&bare));
    let doc = Document::load_mem(&bare).unwrap();
    assert!(
        doc.catalog().unwrap().has(b"StructTreeRoot"),
        "still tagged"
    );
    assert_eq!(
        notes,
        vec!["Not marked as accessible (PDF/UA): a picture has no description".to_string()]
    );

    // Without bookmarks there is no outline, which PDF/UA asks for.
    let (none, notes) = noted(&book, &styles, &assets, &options(true, PdfTarget::Sharing));
    assert!(!ua(&none));
    assert_eq!(notes.len(), 1);

    // An untagged file makes no claim and says nothing.
    let (plain, notes) = noted(
        &book,
        &styles,
        &assets,
        &with_bookmarks(options(false, PdfTarget::Sharing)),
    );
    assert!(!ua(&plain));
    assert!(notes.is_empty());
}
