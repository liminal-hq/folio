// Tests for the Sharing PDF that reads like a book: the front cover first, the back cover last
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use folio_export::outline::{BookmarkNames, Bookmarks, Chapter, Navigation};
use folio_export::pages::{CoverPages, PageChoice, Sheets};
use folio_export::pdf::{export_pdf, NoAssets, PdfOptions, PdfTarget};
use folio_layout::{FontBook, StyleSheet};
use folio_model::{
    Book, BookKind, Frame, FrameBase, FrameId, InlineContent, Pt, Rect, Run, StyleId,
    StyleOverrides, TextFrame,
};
use folio_presets::{new_book, NewBookSpec};
use lopdf::{Dictionary, Document, Object, ObjectId};

const BACK_WORDS: &str = "Quiet moon";
const BACK_SIZE: f64 = 37.0;

fn words(text: &str, href: Option<&str>) -> InlineContent {
    InlineContent {
        inline: vec![Run {
            text: text.to_string(),
            href: href.map(str::to_string),
            ..Default::default()
        }],
        looks: None,
    }
}

fn text_at(id: &str, x: f64, text: &str, href: Option<&str>) -> Frame {
    Frame::Text(TextFrame {
        base: FrameBase {
            id: FrameId(id.into()),
            rect: Rect {
                x: Pt(x),
                y: Pt(60.0),
                w: Pt(200.0),
                h: Pt(80.0),
            },
            rotation: 0.0,
            locked: false,
            from_master: None,
            description: None,
        },
        style: StyleId("body".into()),
        content: Some(words(text, href)),
        flow: None,
        columns: 1,
        baseline_lock: false,
        overrides: Some(StyleOverrides {
            size: Some(Pt(BACK_SIZE)),
            align: None,
            bold: None,
            italic: None,
            font: None,
            valign: None,
        }),
    })
}

/// A picture book whose back cover has words and a link on it.
fn sample() -> (Book, StyleSheet) {
    let made = new_book(&NewBookSpec {
        kind: BookKind::Picture,
        ..NewBookSpec::default()
    })
    .unwrap();
    let mut book = made.book;
    // The back is x = 0 to the trim width; the front starts one trim width and the spine in.
    let cover = &mut book.spreads[0];
    cover
        .frames
        .push(text_at("fr_back_words", 30.0, BACK_WORDS, None));
    cover.frames.push(text_at(
        "fr_back_link",
        30.0 + 0.0,
        "Link here",
        Some("https://example.org/back"),
    ));
    cover.frames.last_mut().unwrap().base_mut().rect.y = Pt(200.0);
    let styles = StyleSheet::from_styles(&book.styles);
    (book, styles)
}

fn sharing(book: &Book, styles: &StyleSheet, pages: PageChoice, navigation: Navigation) -> Vec<u8> {
    export_pdf(
        book,
        styles,
        &FontBook::bundled(),
        &NoAssets,
        &PdfOptions {
            target: PdfTarget::Sharing,
            pages,
            navigation,
            ..PdfOptions::default()
        },
    )
    .unwrap()
}

fn ends() -> PageChoice {
    PageChoice {
        cover: CoverPages::FrontAndBackEnds,
        ..PageChoice::default()
    }
}

fn page_ids(doc: &Document) -> Vec<ObjectId> {
    doc.get_pages().values().copied().collect()
}

fn width(doc: &Document, page: ObjectId) -> f64 {
    let media = doc
        .get_object(page)
        .unwrap()
        .as_dict()
        .unwrap()
        .get(b"MediaBox")
        .unwrap()
        .as_array()
        .unwrap();
    f64::from(media[2].as_float().unwrap())
}

fn content(doc: &Document, page: ObjectId) -> String {
    String::from_utf8_lossy(&doc.get_page_content(page).unwrap()).into_owned()
}

/// The words on page `number` (from 1), through `pdftotext` when this machine has it.
fn words_on(bytes: &[u8], number: usize) -> Option<String> {
    use std::io::Write;
    // Tests run side by side, so each call has a folder of its own.
    static CALLS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let call = CALLS.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let dir = std::env::temp_dir().join(format!("folio-back-{}-{call}", std::process::id()));
    std::fs::create_dir_all(&dir).ok()?;
    let file = dir.join("in.pdf");
    std::fs::File::create(&file).ok()?.write_all(bytes).ok()?;
    let out = std::process::Command::new("pdftotext")
        .args(["-f", &number.to_string(), "-l", &number.to_string()])
        .arg(&file)
        .arg("-")
        .output()
        .ok()?;
    let _ = std::fs::remove_dir_all(&dir);
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).into_owned())
}

fn text(object: &Object) -> String {
    match object {
        Object::String(bytes, _) if bytes.starts_with(&[0xFE, 0xFF]) => {
            let units: Vec<u16> = bytes[2..]
                .chunks(2)
                .map(|c| u16::from_be_bytes([c[0], c[1]]))
                .collect();
            String::from_utf16_lossy(&units)
        }
        Object::String(bytes, _) => String::from_utf8_lossy(bytes).into_owned(),
        _ => String::new(),
    }
}

/// The bookmarks as (title, page number from 1).
fn bookmarks(doc: &Document) -> Vec<(String, usize)> {
    let pages = page_ids(doc);
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
        let number = pages.iter().position(|id| *id == page).unwrap() + 1;
        found.push((text(dict.get(b"Title").unwrap()), number));
        next = dict.get(b"Next").ok().cloned();
    }
    found
}

fn annotations(doc: &Document, page: ObjectId) -> Vec<Dictionary> {
    let dict = doc.get_object(page).unwrap().as_dict().unwrap();
    let Ok(annots) = dict.get(b"Annots") else {
        return Vec::new();
    };
    doc.dereference(annots)
        .unwrap()
        .1
        .as_array()
        .unwrap()
        .iter()
        .map(|a| doc.dereference(a).unwrap().1.as_dict().unwrap().clone())
        .collect()
}

#[test]
fn the_file_opens_on_the_front_cover_and_ends_on_the_back_each_at_trim_size() {
    let (book, styles) = sample();
    let plain = Document::load_mem(&sharing(
        &book,
        &styles,
        PageChoice::default(),
        Navigation::default(),
    ))
    .unwrap();
    let bytes = sharing(&book, &styles, ends(), Navigation::default());
    let doc = Document::load_mem(&bytes).unwrap();
    let pages = page_ids(&doc);
    // The cover is the front alone in the plain file; the back is the one page more.
    assert_eq!(pages.len(), page_ids(&plain).len() + 1);
    let trim = book.trim.w.0;
    assert!((width(&doc, pages[0]) - trim).abs() < 0.01);
    assert!((width(&doc, *pages.last().unwrap()) - trim).abs() < 0.01);
    // The interior is as it was, between the two.
    assert_eq!(
        pages.len(),
        book.spreads.iter().map(|s| s.pages.len()).sum::<usize>() - book.spreads[0].pages.len() + 2
    );
}

#[test]
fn the_back_page_has_the_back_covers_words_and_the_front_does_not() {
    let (book, styles) = sample();
    let bytes = sharing(&book, &styles, ends(), Navigation::default());
    let doc = Document::load_mem(&bytes).unwrap();
    let pages = page_ids(&doc);
    let (first, last) = (pages[0], *pages.last().unwrap());
    // The back words are drawn at 37 points on the last page alone.
    assert!(
        content(&doc, last).contains("37 Tf"),
        "{}",
        content(&doc, last)
    );
    assert!(!content(&doc, first).contains("37 Tf"));
    if let Some(back) = words_on(&bytes, pages.len()) {
        assert!(back.contains(BACK_WORDS), "{back}");
        let front = words_on(&bytes, 1).unwrap();
        assert!(!front.contains(BACK_WORDS), "{front}");
    }
}

#[test]
fn the_covers_are_not_numbered_and_the_pages_between_still_are() {
    let (book, styles) = sample();
    let navigation = Navigation {
        page_labels: true,
        names: BookmarkNames {
            front: "Couverture".into(),
            back: "Quatrième".into(),
            page: "Page {n}".into(),
            pages: "Pages {from} à {to}".into(),
        },
        ..Navigation::default()
    };
    let doc = Document::load_mem(&sharing(&book, &styles, ends(), navigation)).unwrap();
    let labels = doc.catalog().unwrap().get(b"PageLabels").unwrap();
    let labels = doc.dereference(labels).unwrap().1.as_dict().unwrap();
    let nums = doc
        .dereference(labels.get(b"Nums").unwrap())
        .unwrap()
        .1
        .as_array()
        .unwrap();
    let mut prefixes = Vec::new();
    for pair in nums.chunks(2) {
        let dict = doc.dereference(&pair[1]).unwrap().1.as_dict().unwrap();
        prefixes.push((
            pair[0].as_i64().unwrap(),
            dict.get(b"P").map(text).unwrap_or_default(),
            dict.has(b"S"),
        ));
    }
    let last = page_ids(&doc).len() as i64 - 1;
    assert!(
        prefixes.contains(&(0, "Couverture".into(), false)),
        "{prefixes:?}"
    );
    assert!(
        prefixes.contains(&(last, "Quatrième".into(), false)),
        "{prefixes:?}"
    );
    // At least one interior page is numbered.
    assert!(prefixes.iter().any(|(_, _, numbered)| *numbered));
}

#[test]
fn bookmarks_name_the_covers_with_the_names_the_caller_passes() {
    let (book, styles) = sample();
    let chapters = vec![Chapter {
        title: "The reeds".into(),
        spread: book.spreads[2].id.0.clone(),
        page: 0,
    }];
    let navigation = Navigation {
        bookmarks: Bookmarks::Chapters(chapters),
        names: BookmarkNames {
            front: "Couverture".into(),
            back: "Quatrième".into(),
            page: "Page {n}".into(),
            pages: "Pages {from} à {to}".into(),
        },
        ..Navigation::default()
    };
    let doc = Document::load_mem(&sharing(&book, &styles, ends(), navigation.clone())).unwrap();
    let found = bookmarks(&doc);
    let pages = page_ids(&doc).len();
    assert_eq!(found.first(), Some(&("Couverture".to_string(), 1)));
    assert_eq!(found.last(), Some(&("Quatrième".to_string(), pages)));
    assert_eq!(found.len(), 3, "{found:?}");
    assert!(found[1].1 > 1 && found[1].1 < pages);

    let by_page = Navigation {
        bookmarks: Bookmarks::Pages,
        ..navigation
    };
    let doc = Document::load_mem(&sharing(&book, &styles, ends(), by_page)).unwrap();
    let found = bookmarks(&doc);
    assert_eq!(found.len(), pages);
    assert_eq!(found[0].0, "Couverture");
    assert_eq!(found[pages - 1], ("Quatrième".to_string(), pages));
}

#[test]
fn links_on_the_back_cover_are_on_the_back_page_and_links_to_the_cover_go_to_the_front() {
    let (book, styles) = sample();
    let doc = Document::load_mem(&sharing(&book, &styles, ends(), Navigation::default())).unwrap();
    let pages = page_ids(&doc);
    let on_back = annotations(&doc, *pages.last().unwrap());
    assert_eq!(on_back.len(), 1, "{on_back:?}");
    let action = doc
        .dereference(on_back[0].get(b"A").unwrap())
        .unwrap()
        .1
        .as_dict()
        .unwrap();
    assert_eq!(
        action.get(b"URI").unwrap().as_string().unwrap(),
        "https://example.org/back"
    );
    // Nor the front nor the interior carries it.
    for page in &pages[..pages.len() - 1] {
        assert!(annotations(&doc, *page).is_empty());
    }
}

#[test]
fn the_range_counts_interior_pages_and_the_covers_come_outside_it() {
    let (book, styles) = sample();
    let choice = PageChoice {
        range: Some((2, 4)),
        ..ends()
    };
    let doc = Document::load_mem(&sharing(&book, &styles, choice, Navigation::default())).unwrap();
    let pages = page_ids(&doc);
    assert_eq!(pages.len(), 3 + 2);
    assert!(content(&doc, *pages.last().unwrap()).contains("37 Tf"));
    // A range past the end leaves the covers alone.
    let none = PageChoice {
        range: Some((900, 999)),
        ..ends()
    };
    let doc = Document::load_mem(&sharing(&book, &styles, none, Navigation::default())).unwrap();
    assert_eq!(page_ids(&doc).len(), 2);
}

#[test]
fn spreads_keep_the_front_and_the_back_each_on_a_sheet_of_their_own() {
    let (book, styles) = sample();
    let choice = PageChoice {
        sheets: Sheets::Spreads,
        ..ends()
    };
    let doc = Document::load_mem(&sharing(&book, &styles, choice, Navigation::default())).unwrap();
    let pages = page_ids(&doc);
    let trim = book.trim.w.0;
    assert!((width(&doc, pages[0]) - trim).abs() < 0.01);
    assert!((width(&doc, *pages.last().unwrap()) - trim).abs() < 0.01);
    assert!(pages.iter().any(|p| width(&doc, *p) > 1.9 * trim));
    assert!(content(&doc, *pages.last().unwrap()).contains("37 Tf"));
}

#[test]
fn the_whole_cover_sheet_stays_one_wide_first_page_and_the_cover_can_be_left_out() {
    let (book, styles) = sample();
    let wide = PageChoice {
        cover: CoverPages::FrontAndBack,
        ..PageChoice::default()
    };
    let doc = Document::load_mem(&sharing(&book, &styles, wide, Navigation::default())).unwrap();
    let pages = page_ids(&doc);
    assert!(width(&doc, pages[0]) > 1.9 * book.trim.w.0);
    let off = PageChoice {
        cover: CoverPages::None,
        ..PageChoice::default()
    };
    let doc2 = Document::load_mem(&sharing(&book, &styles, off, Navigation::default())).unwrap();
    assert_eq!(page_ids(&doc2).len(), pages.len() - 1);
}

#[test]
fn a_press_file_never_gets_the_back_cover_whatever_was_asked() {
    let (book, styles) = sample();
    for target in [PdfTarget::Print, PdfTarget::PrintShop] {
        let make = |pages| {
            let bytes = export_pdf(
                &book,
                &styles,
                &FontBook::bundled(),
                &NoAssets,
                &PdfOptions {
                    target,
                    pages,
                    ..PdfOptions::default()
                },
            )
            .unwrap();
            page_ids(&Document::load_mem(&bytes).unwrap()).len()
        };
        assert_eq!(make(ends()), make(PageChoice::default()), "{target:?}");
    }
}

#[test]
fn a_good_isbn_puts_the_barcode_on_the_back_page_and_kdp_is_left_to_add_its_own() {
    use folio_model::BarcodeChoice;
    let (mut book, styles) = sample();
    let bytes_for = |book: &Book| sharing(book, &styles, ends(), Navigation::default());
    let size = |bytes: &[u8]| {
        let doc = Document::load_mem(bytes).unwrap();
        content(&doc, *page_ids(&doc).last().unwrap()).len()
    };
    let bare = size(&bytes_for(&book));
    book.metadata.isbn.paperback = Some("978-0-306-40615-7".into());
    let with = bytes_for(&book);
    assert!(size(&with) > bare + 200, "{} against {bare}", size(&with));
    if let Some(back) = words_on(&with, page_ids(&Document::load_mem(&with).unwrap()).len()) {
        assert!(back.contains("9780306406157"), "{back}");
    }
    // The front and the interior have none of it.
    let doc = Document::load_mem(&with).unwrap();
    assert!(!content(&doc, page_ids(&doc)[0]).contains("9780306406157"));
    // A bad ISBN, or leaving it to KDP, draws nothing.
    book.metadata.isbn.paperback = Some("978-0-306-40615-8".into());
    assert_eq!(size(&bytes_for(&book)), bare);
    book.metadata.isbn.paperback = Some("978-0-306-40615-7".into());
    book.barcode = BarcodeChoice::Kdp;
    assert_eq!(size(&bytes_for(&book)), bare);
}

#[test]
#[ignore]
fn dump_for_the_eye() {
    let (mut book, styles) = sample();
    book.metadata.isbn.paperback = Some("978-0-306-40615-7".into());
    let dir = std::env::var("FOLIO_DUMP").unwrap();
    std::fs::write(
        format!("{dir}/share.pdf"),
        sharing(&book, &styles, ends(), Navigation::default()),
    )
    .unwrap();
}

#[test]
fn a_book_without_chapters_gets_one_bookmark_for_each_spread_between_the_covers() {
    let (book, styles) = sample();
    let names = BookmarkNames {
        front: "Couverture".into(),
        back: "Quatrième".into(),
        page: "Page {n}".into(),
        pages: "Pages {from} à {to}".into(),
    };
    let navigation = Navigation {
        bookmarks: Bookmarks::Automatic(Vec::new()),
        names,
        ..Navigation::default()
    };
    let doc = Document::load_mem(&sharing(&book, &styles, ends(), navigation)).unwrap();
    let found = bookmarks(&doc);
    let pages = page_ids(&doc).len();
    assert_eq!(found.first(), Some(&("Couverture".to_string(), 1)));
    assert_eq!(found.last(), Some(&("Quatrième".to_string(), pages)));
    // One for each spread of the interior, in order, each landing on its first page.
    let spreads = book.spreads.len() - 1;
    assert_eq!(found.len(), spreads + 2, "{found:?}");
    assert!(found.windows(2).all(|w| w[0].1 < w[1].1), "{found:?}");
    assert_eq!(found[1].1, 2);
    assert!(
        found[1..found.len() - 1]
            .iter()
            .all(|(title, _)| title.starts_with("Pages ")),
        "{found:?}"
    );
    assert!(found.iter().any(|(t, _)| t.contains(" à ")), "{found:?}");
}

#[test]
fn a_book_with_chapters_keeps_them_between_the_covers() {
    let (book, styles) = sample();
    let chapters = vec![Chapter {
        title: "The reeds".into(),
        spread: book.spreads[2].id.0.clone(),
        page: 0,
    }];
    let navigation = Navigation {
        bookmarks: Bookmarks::Automatic(chapters),
        ..Navigation::default()
    };
    let doc = Document::load_mem(&sharing(&book, &styles, ends(), navigation)).unwrap();
    let found = bookmarks(&doc);
    let titles: Vec<&str> = found.iter().map(|(t, _)| t.as_str()).collect();
    assert_eq!(titles, ["Front cover", "The reeds", "Back cover"]);
}

#[test]
fn the_sharing_pdf_asks_for_automatic_bookmarks_and_a_paperback_for_none() {
    use folio_export::prefs::{resolve, PdfKind};
    let (book, _) = sample();
    let sharing = resolve(PdfKind::Sharing, &[], &book);
    assert!(sharing.wants_chapters);
    assert_eq!(
        sharing.extras.navigation.bookmarks,
        Bookmarks::Automatic(Vec::new())
    );
    assert_eq!(
        resolve(PdfKind::Kdp, &[], &book)
            .extras
            .navigation
            .bookmarks,
        Bookmarks::None
    );
}
