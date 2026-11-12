// Tests for flattened PDFs: a picture with feathered edges blended onto the page, or left see-through
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::collections::HashMap;

use folio_export::pdf::{export_pdf_noted, Assets, InkOptions, PdfOptions, PdfTarget};
use folio_export::picture::{ImagePolicy, PictureCache};
use folio_layout::{FontBook, StyleSheet};
use folio_model::{
    Background, Book, BookKind, Colour, Fit, Frame, FrameBase, FrameId, ImageFrame, ImageRef, Pt,
    Rect, SpreadKind,
};
use folio_presets::{new_book, NewBookSpec};
use image::{ImageEncoder, Rgb, RgbImage, Rgba, RgbaImage};
use lopdf::{Dictionary, Document, Object};

struct Memory(HashMap<String, Vec<u8>>);

impl Assets for Memory {
    fn bytes(&self, path: &str) -> Option<Vec<u8>> {
        self.0.get(path).cloned()
    }
}

const PINK: [u8; 3] = [255, 192, 203];
const SIDE: u32 = 600;
/// The picture's own colour, wherever it is not see-through.
const RED: [u8; 3] = [200, 30, 30];

/// A red disc whose edge fades out over 50 pixels, with nothing at all outside it.
fn feathered() -> RgbaImage {
    RgbaImage::from_fn(SIDE, SIDE, |x, y| {
        let (dx, dy) = (f64::from(x) - 299.5, f64::from(y) - 299.5);
        let d = (dx * dx + dy * dy).sqrt();
        let a = ((250.0 - d) / 50.0).clamp(0.0, 1.0);
        Rgba([RED[0], RED[1], RED[2], (a * 255.0).round() as u8])
    })
}

fn png_of(image: &image::DynamicImage) -> Vec<u8> {
    let mut out = Vec::new();
    image::codecs::png::PngEncoder::new(&mut out)
        .write_image(
            image.as_bytes(),
            image.width(),
            image.height(),
            image.color().into(),
        )
        .unwrap();
    out
}

fn assets() -> Memory {
    let flat = RgbImage::from_fn(640, 640, |x, _| {
        if x < 320 {
            Rgb([20, 120, 40])
        } else {
            Rgb([30, 30, 200])
        }
    });
    Memory(HashMap::from([
        ("fade.png".to_string(), png_of(&feathered().into())),
        ("flat.png".to_string(), png_of(&flat.into())),
    ]))
}

fn frame(id: &str, asset: &str, x: f64, y: f64, side: f64, rotation: f64) -> Frame {
    Frame::Image(ImageFrame {
        base: FrameBase {
            id: FrameId(id.into()),
            rect: Rect {
                x: Pt(x),
                y: Pt(y),
                w: Pt(side),
                h: Pt(side),
            },
            rotation,
            locked: false,
            from_master: None,
            description: None,
        },
        image: ImageRef {
            asset: Some(asset.into()),
            fit: Fit::Fill,
            focus: (0.5, 0.5),
            crop: None,
        },
        placeholder_label: None,
    })
}

/// A picture book with only the frames given, on its first two pages that are not the cover, each
/// page on the colour given (or on the bare paper).
fn book_with(pages: [(Option<&str>, Vec<Frame>); 2]) -> (Book, StyleSheet) {
    let mut book = new_book(&NewBookSpec {
        kind: BookKind::Picture,
        ..NewBookSpec::default()
    })
    .unwrap()
    .book;
    book.master_pages.clear();
    let mut inside = book
        .spreads
        .iter_mut()
        .filter(|s| s.kind != SpreadKind::Cover);
    for (colour, frames) in pages {
        let spread = inside.next().expect("the sample has pages inside");
        spread.frames = frames;
        spread.bg_span = None;
        for page in &mut spread.pages {
            page.master = None;
            page.bg = colour.map(|c| Background::Colour {
                colour: Colour(c.into()),
            });
        }
    }
    // The pages that are left are bare.
    for spread in inside {
        spread.frames.clear();
        for page in &mut spread.pages {
            page.master = None;
            page.bg = None;
        }
    }
    let styles = StyleSheet::from_styles(&book.styles);
    (book, styles)
}

fn export(
    book: &Book,
    styles: &StyleSheet,
    target: PdfTarget,
    flatten: Option<bool>,
    ink: bool,
) -> (Vec<u8>, Vec<String>) {
    export_pdf_noted(
        book,
        styles,
        &FontBook::bundled(),
        &assets(),
        &PdfOptions {
            target,
            ink: ink.then(InkOptions::default),
            flatten,
            ..PdfOptions::default()
        },
        &ImagePolicy::for_target(target),
        &PictureCache::default(),
    )
    .unwrap()
}

struct Image {
    width: usize,
    space: String,
    has_mask: bool,
    pixels: Vec<u8>,
}

fn name_of(dict: &Dictionary, key: &[u8]) -> String {
    match dict.get(key) {
        Ok(Object::Name(n)) => String::from_utf8_lossy(n).into_owned(),
        Ok(Object::Array(a)) => a
            .first()
            .and_then(|o| o.as_name().ok())
            .map(|n| String::from_utf8_lossy(n).into_owned())
            .unwrap_or_default(),
        _ => String::new(),
    }
}

/// Every image of a file apart from the soft masks, with its pixels decoded when they are plain.
fn images(bytes: &[u8]) -> Vec<Image> {
    let doc = Document::load_mem(bytes).unwrap();
    let masks: Vec<_> = doc
        .objects
        .values()
        .filter_map(|o| o.as_stream().ok())
        .filter_map(|s| s.dict.get(b"SMask").ok()?.as_reference().ok())
        .collect();
    let mut found = Vec::new();
    for (id, object) in &doc.objects {
        let Object::Stream(stream) = object else {
            continue;
        };
        let dict = &stream.dict;
        if name_of(dict, b"Subtype") != "Image" || masks.contains(id) {
            continue;
        }
        let space = match dict.get(b"ColorSpace") {
            Ok(Object::Array(a)) => a
                .first()
                .and_then(|o| o.as_name().ok())
                .map(|n| String::from_utf8_lossy(n).into_owned())
                .unwrap_or_default(),
            _ => name_of(dict, b"ColorSpace"),
        };
        found.push(Image {
            width: dict.get(b"Width").and_then(|w| w.as_i64()).unwrap() as usize,
            space,
            has_mask: dict.has(b"SMask"),
            pixels: if name_of(dict, b"Filter") == "FlateDecode" {
                let mut plain = Vec::new();
                std::io::Read::read_to_end(
                    &mut flate2::read::ZlibDecoder::new(&stream.content[..]),
                    &mut plain,
                )
                .unwrap();
                plain
            } else {
                Vec::new()
            },
        });
    }
    found
}

/// The picture with the red disc, among a file's images.
fn the_disc(images: &[Image]) -> &Image {
    let mut discs = images.iter().filter(|i| i.width == SIDE as usize);
    let disc = discs.next().expect("the disc is in the file");
    assert!(discs.next().is_none(), "one disc");
    disc
}

/// What the disc's pixel at (x, y) should be over `behind`: the picture over the page colour.
fn blend(x: u32, y: u32, behind: [u8; 3]) -> [u8; 3] {
    let p = feathered().get_pixel(x, y).0;
    let a = u32::from(p[3]);
    let mut out = [0u8; 3];
    for i in 0..3 {
        out[i] = ((u32::from(p[i]) * a + u32::from(behind[i]) * (255 - a) + 127) / 255) as u8;
    }
    out
}

fn at(image: &Image, x: usize, y: usize) -> [u8; 3] {
    let o = (y * image.width + x) * 3;
    [image.pixels[o], image.pixels[o + 1], image.pixels[o + 2]]
}

fn near(a: [u8; 3], b: [u8; 3]) {
    for i in 0..3 {
        assert!(
            (i32::from(a[i]) - i32::from(b[i])).abs() <= 2,
            "{a:?} is not {b:?}"
        );
    }
}

fn one_disc(colour: Option<&str>) -> (Book, StyleSheet) {
    book_with([
        (
            colour,
            vec![frame("a", "fade.png", 100.0, 100.0, 200.0, 0.0)],
        ),
        (None, vec![]),
    ])
}

#[test]
fn a_paperback_interior_blends_a_feathered_picture_onto_the_pink_page() {
    let (book, styles) = one_disc(Some("#ffc0cb"));
    let (bytes, notes) = export(&book, &styles, PdfTarget::Print, None, false);
    assert!(notes.is_empty(), "{notes:?}");
    let all = images(&bytes);
    let disc = the_disc(&all);
    assert!(!disc.has_mask, "a flattened picture has no soft mask");
    assert_eq!(disc.space, "DeviceRGB");
    assert_eq!(disc.pixels.len(), (SIDE * SIDE * 3) as usize);
    // The edge of the disc, half faded; its middle; and a corner nothing was drawn on.
    near(at(disc, 300 + 235, 300), blend(300 + 235, 300, PINK));
    assert_ne!(blend(300 + 235, 300, PINK), PINK);
    assert_ne!(blend(300 + 235, 300, PINK), RED);
    near(at(disc, 300, 300), RED);
    near(at(disc, 2, 2), PINK);
}

#[test]
fn a_sharing_pdf_keeps_the_transparency_live_unless_asked() {
    let (book, styles) = one_disc(Some("#ffc0cb"));
    let (bytes, notes) = export(&book, &styles, PdfTarget::Sharing, None, false);
    assert!(notes.is_empty());
    assert!(the_disc(&images(&bytes)).has_mask);

    let (bytes, _) = export(&book, &styles, PdfTarget::Sharing, Some(true), false);
    let all = images(&bytes);
    let disc = the_disc(&all);
    assert!(!disc.has_mask);
    near(at(disc, 2, 2), PINK);

    // And a paperback can be made live by saying so.
    let (bytes, _) = export(&book, &styles, PdfTarget::Print, Some(false), false);
    assert!(the_disc(&images(&bytes)).has_mask);
}

#[test]
fn a_page_with_no_colour_is_blended_onto_white() {
    let (book, styles) = one_disc(None);
    let (bytes, _) = export(&book, &styles, PdfTarget::Print, None, false);
    let all = images(&bytes);
    let disc = the_disc(&all);
    assert!(!disc.has_mask);
    near(at(disc, 2, 2), [255, 255, 255]);
    near(
        at(disc, 300 + 235, 300),
        blend(300 + 235, 300, [255, 255, 255]),
    );
}

#[test]
fn a_picture_on_another_picture_stays_see_through_and_says_so() {
    let (book, styles) = book_with([
        (
            Some("#ffc0cb"),
            vec![
                frame("under", "flat.png", 80.0, 80.0, 200.0, 0.0),
                frame("over", "fade.png", 150.0, 150.0, 200.0, 0.0),
            ],
        ),
        (None, vec![]),
    ]);
    let (bytes, notes) = export(&book, &styles, PdfTarget::Print, None, false);
    let all = images(&bytes);
    assert!(the_disc(&all).has_mask);
    assert_eq!(notes.len(), 1, "{notes:?}");
    assert!(
        notes[0].starts_with("A see-through picture sits on other pictures or shapes")
            && notes[0].ends_with("(page 1)"),
        "{notes:?}"
    );
}

#[test]
fn a_picture_with_only_words_or_art_above_it_is_still_flattened() {
    let (book, styles) = book_with([
        (
            Some("#ffc0cb"),
            vec![
                frame("over", "fade.png", 150.0, 150.0, 200.0, 0.0),
                frame("above", "flat.png", 80.0, 80.0, 200.0, 0.0),
            ],
        ),
        (None, vec![]),
    ]);
    let (bytes, notes) = export(&book, &styles, PdfTarget::Print, None, false);
    assert!(notes.is_empty(), "{notes:?}");
    assert!(!the_disc(&images(&bytes)).has_mask);
}

#[test]
fn a_turned_frame_is_flattened_and_is_judged_by_its_turned_corners() {
    let (book, styles) = book_with([
        (
            Some("#ffc0cb"),
            vec![frame("a", "fade.png", 100.0, 100.0, 200.0, 30.0)],
        ),
        (None, vec![]),
    ]);
    let (bytes, notes) = export(&book, &styles, PdfTarget::Print, None, false);
    assert!(notes.is_empty(), "{notes:?}");
    let all = images(&bytes);
    let disc = the_disc(&all);
    assert!(!disc.has_mask);
    near(at(disc, 2, 2), PINK);

    // Another picture clear of the frame's upright box but inside the box of its turned corners.
    let beside = |turn: f64| {
        let (book, styles) = book_with([
            (
                Some("#ffc0cb"),
                vec![
                    frame("beside", "flat.png", 310.0, 100.0, 30.0, 0.0),
                    frame("a", "fade.png", 100.0, 100.0, 200.0, turn),
                ],
            ),
            (None, vec![]),
        ]);
        let (bytes, notes) = export(&book, &styles, PdfTarget::Print, None, false);
        (the_disc(&images(&bytes)).has_mask, notes)
    };
    let (live, notes) = beside(45.0);
    assert!(live && notes.len() == 1, "{notes:?}");
    let (live, notes) = beside(0.0);
    assert!(!live && notes.is_empty(), "{notes:?}");
}

#[test]
fn a_picture_used_on_two_pages_with_different_colours_is_two_pictures() {
    let two = |a: Option<&str>, b: Option<&str>| {
        let (book, styles) = book_with([
            (a, vec![frame("a", "fade.png", 100.0, 100.0, 200.0, 0.0)]),
            (b, vec![frame("b", "fade.png", 100.0, 100.0, 200.0, 0.0)]),
        ]);
        let (bytes, notes) = export(&book, &styles, PdfTarget::Print, None, false);
        assert!(notes.is_empty(), "{notes:?}");
        images(&bytes)
            .into_iter()
            .filter(|i| i.width == SIDE as usize)
            .collect::<Vec<_>>()
    };
    let different = two(Some("#ffc0cb"), Some("#c0e0ff"));
    assert_eq!(different.len(), 2);
    let corners: Vec<[u8; 3]> = different.iter().map(|i| at(i, 2, 2)).collect();
    assert!(corners.contains(&PINK) && corners.contains(&[192, 224, 255]));
    // The same colour twice is one picture, as ever.
    assert_eq!(two(Some("#ffc0cb"), Some("#ffc0cb")).len(), 1);
}

#[test]
fn a_file_in_ink_has_no_rgb_pictures_and_no_soft_masks() {
    let (book, styles) = one_disc(Some("#ffc0cb"));
    let (bytes, _) = export(&book, &styles, PdfTarget::Print, None, true);
    let all = images(&bytes);
    assert!(!all.is_empty());
    for image in &all {
        assert_eq!(image.space, "DeviceCMYK");
        assert!(!image.has_mask);
    }
    let doc = Document::load_mem(&bytes).unwrap();
    assert!(!doc.objects.values().any(|o| match o {
        Object::Stream(s) => s.dict.has(b"SMask"),
        Object::Dictionary(d) => d.has(b"SMask"),
        _ => false,
    }));
}

#[test]
fn the_print_shop_file_is_flattened_and_keeps_its_output_intent() {
    let (book, styles) = one_disc(Some("#ffc0cb"));
    let (bytes, notes) = export(&book, &styles, PdfTarget::PrintShop, None, false);
    assert!(notes.is_empty(), "{notes:?}");
    assert!(!the_disc(&images(&bytes)).has_mask);
    let doc = Document::load_mem(&bytes).unwrap();
    assert!(doc.catalog().unwrap().has(b"OutputIntents"));
    let info = String::from_utf8_lossy(&bytes).into_owned();
    assert!(info.contains("GTS_PDFX"), "PDF/X is declared");
}

#[test]
fn a_sharing_pdf_with_a_back_page_flattens_the_back_picture_against_the_back_colour() {
    const BLUE: [u8; 3] = [173, 216, 230];
    let (mut book, _) = book_with([(None, vec![]), (None, vec![])]);
    let cover = book
        .spreads
        .iter_mut()
        .find(|s| s.kind == SpreadKind::Cover)
        .expect("a picture book has a cover");
    cover.pages[0].bg = Some(Background::Colour {
        colour: Colour("#add8e6".into()),
    });
    // The back panel starts at the left edge of the sheet.
    cover.frames = vec![frame("back_art", "fade.png", 20.0, 100.0, 200.0, 0.0)];
    let styles = StyleSheet::from_styles(&book.styles);
    let (bytes, _) = export_pdf_noted(
        &book,
        &styles,
        &FontBook::bundled(),
        &assets(),
        &PdfOptions {
            target: PdfTarget::Sharing,
            flatten: Some(true),
            pages: folio_export::pages::PageChoice {
                cover: folio_export::pages::CoverPages::FrontAndBackEnds,
                ..Default::default()
            },
            ..PdfOptions::default()
        },
        &ImagePolicy::for_target(PdfTarget::Sharing),
        &PictureCache::default(),
    )
    .unwrap();
    let all = images(&bytes);
    let disc = the_disc(&all);
    assert!(!disc.has_mask, "the back picture is flattened");
    near(at(disc, 2, 2), BLUE);
    near(at(disc, 300 + 235, 300), blend(300 + 235, 300, BLUE));
}
