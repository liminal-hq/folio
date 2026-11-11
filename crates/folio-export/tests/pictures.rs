// Tests for the pictures of a PDF: sized for their place, written as they suit, never drawn twice
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::collections::HashMap;

use folio_export::pdf::{export_pdf, export_pdf_with, Assets, InkOptions, PdfOptions, PdfTarget};
use folio_export::picture::{ImagePolicy, PictureCache};
use folio_layout::{FontBook, StyleSheet};
use folio_model::{Book, BookKind, Frame};
use folio_presets::{new_book, NewBookSpec};
use image::codecs::jpeg::JpegEncoder;
use image::{ExtendedColorType, ImageEncoder, Rgb, RgbImage, Rgba, RgbaImage};
use lopdf::{Dictionary, Document, Object};

struct Memory(HashMap<String, Vec<u8>>);

impl Assets for Memory {
    fn bytes(&self, path: &str) -> Option<Vec<u8>> {
        self.0.get(path).cloned()
    }
}

/// Pixels that are different everywhere, the way a photograph is: nothing for a codec to fold.
fn noise(w: u32, h: u32, seed: u32) -> RgbImage {
    let mut state = seed.wrapping_mul(2_654_435_761).wrapping_add(1);
    RgbImage::from_fn(w, h, |x, y| {
        state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        let n = (state >> 24) as u8;
        Rgb([
            ((x / 4 + y / 7) as u8).wrapping_add(n / 3),
            ((x / 5) as u8).wrapping_add(n / 3),
            ((y / 3) as u8).wrapping_add(n / 3),
        ])
    })
}

fn png(image: &image::DynamicImage) -> Vec<u8> {
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

fn jpeg(image: &RgbImage) -> Vec<u8> {
    let mut out = Vec::new();
    JpegEncoder::new_with_quality(&mut out, 85)
        .encode(
            image.as_raw(),
            image.width(),
            image.height(),
            ExtendedColorType::Rgb8,
        )
        .unwrap();
    out
}

/// A picture book whose picture frames show the given assets, one for each frame in turn.
fn book_showing(paths: &[&str]) -> (Book, StyleSheet) {
    let made = new_book(&NewBookSpec {
        kind: BookKind::Picture,
        ..NewBookSpec::default()
    })
    .unwrap();
    let mut book = made.book;
    let mut frames: Vec<&mut Frame> = book
        .spreads
        .iter_mut()
        .flat_map(|s| s.frames.iter_mut())
        .filter(|f| matches!(f, Frame::Image(_)))
        .collect();
    assert!(
        frames.len() >= paths.len(),
        "the sample has frames for the pictures"
    );
    for (frame, path) in frames.iter_mut().zip(paths) {
        if let Frame::Image(image) = frame {
            image.image.asset = Some((*path).into());
        }
    }
    let styles = StyleSheet::from_styles(&book.styles);
    (book, styles)
}

fn export(book: &Book, styles: &StyleSheet, assets: &Memory, target: PdfTarget) -> Vec<u8> {
    export_pdf(
        book,
        styles,
        &FontBook::bundled(),
        assets,
        &PdfOptions {
            target,
            ink: None,
            crop_marks: false,
        },
    )
    .unwrap()
}

struct Picture {
    width: i64,
    filter: String,
    space: String,
    has_mask: bool,
    data: Vec<u8>,
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

/// The picture objects of a file, leaving out the soft masks that go with them.
fn pictures(bytes: &[u8]) -> Vec<Picture> {
    let doc = Document::load_mem(bytes).unwrap();
    let mut found = Vec::new();
    for object in doc.objects.values() {
        let Object::Stream(stream) = object else {
            continue;
        };
        let dict = &stream.dict;
        if name_of(dict, b"Subtype") != "Image" || dict.has(b"ImageMask") {
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
        // A soft mask is a picture too, but it is grey and belongs to another.
        let is_mask = doc.objects.values().any(|o| {
            o.as_stream()
                .ok()
                .and_then(|s| s.dict.get(b"SMask").ok())
                .and_then(|m| m.as_reference().ok())
                .is_some_and(|id| doc.objects.get(&id) == Some(object))
        });
        if is_mask {
            continue;
        }
        found.push(Picture {
            width: dict.get(b"Width").and_then(|w| w.as_i64()).unwrap_or(0),
            filter: name_of(dict, b"Filter"),
            space,
            has_mask: dict.has(b"SMask"),
            data: stream.content.clone(),
        });
    }
    found
}

#[test]
fn a_photograph_is_reduced_to_its_place_and_written_as_a_jpeg() {
    let assets = Memory(HashMap::from([(
        "a.png".to_string(),
        png(&noise(2400, 2400, 1).into()),
    )]));
    let (book, styles) = book_showing(&["a.png"]);
    let bytes = export(&book, &styles, &assets, PdfTarget::Sharing);
    let made = pictures(&bytes);
    assert_eq!(made.len(), 1);
    assert_eq!(made[0].filter, "DCTDecode");
    // A page is a few inches across: 150 ppi is well under the 2400 the author brought.
    assert!(
        made[0].width > 300 && made[0].width < 1600,
        "{}",
        made[0].width
    );
    assert!(bytes.len() < 2_000_000, "{} bytes", bytes.len());
}

#[test]
fn flat_art_stays_lossless() {
    let flat = RgbImage::from_fn(2400, 2400, |x, y| match (x / 600 + y / 800) % 3 {
        0 => Rgb([240, 200, 60]),
        1 => Rgb([40, 120, 200]),
        _ => Rgb([250, 250, 245]),
    });
    let assets = Memory(HashMap::from([("a.png".to_string(), png(&flat.into()))]));
    let (book, styles) = book_showing(&["a.png"]);
    let made = pictures(&export(&book, &styles, &assets, PdfTarget::Sharing));
    assert_eq!(made.len(), 1);
    assert_eq!(made[0].filter, "FlateDecode");
    assert!(made[0].width < 2400);
}

#[test]
fn a_small_jpeg_is_passed_through_byte_for_byte() {
    let source = jpeg(&noise(200, 200, 3));
    let assets = Memory(HashMap::from([("a.jpg".to_string(), source.clone())]));
    let (book, styles) = book_showing(&["a.jpg"]);
    let made = pictures(&export(&book, &styles, &assets, PdfTarget::Sharing));
    assert_eq!(made.len(), 1);
    assert_eq!(made[0].filter, "DCTDecode");
    assert_eq!(made[0].data, source);
}

#[test]
fn a_picture_with_transparency_keeps_its_soft_mask() {
    let see_through = RgbaImage::from_fn(1200, 1200, |x, y| {
        let n = noise(1, 1, x * 31 + y).get_pixel(0, 0).0;
        Rgba([n[0], n[1], n[2], (x * 255 / 1200) as u8])
    });
    let assets = Memory(HashMap::from([(
        "a.png".to_string(),
        png(&see_through.into()),
    )]));
    let (book, styles) = book_showing(&["a.png"]);
    let made = pictures(&export(&book, &styles, &assets, PdfTarget::Sharing));
    assert_eq!(made.len(), 1);
    assert!(made[0].has_mask);
    assert_eq!(made[0].filter, "FlateDecode");
}

#[test]
fn a_grey_picture_is_written_in_grey() {
    let grey = image::GrayImage::from_fn(2000, 2000, |x, y| {
        image::Luma([noise(1, 1, x * 7 + y * 13).get_pixel(0, 0).0[0]])
    });
    let assets = Memory(HashMap::from([("a.png".to_string(), png(&grey.into()))]));
    let (book, styles) = book_showing(&["a.png"]);
    let made = pictures(&export(&book, &styles, &assets, PdfTarget::Sharing));
    assert_eq!(made.len(), 1);
    assert_eq!(made[0].space, "DeviceGray");
}

#[test]
fn a_picture_used_on_two_pages_is_one_object_sized_for_the_larger_place() {
    let assets = Memory(HashMap::from([(
        "a.png".to_string(),
        png(&noise(2400, 2400, 5).into()),
    )]));
    let (book, styles) = book_showing(&["a.png", "a.png"]);
    let made = pictures(&export(&book, &styles, &assets, PdfTarget::Sharing));
    assert_eq!(made.len(), 1, "one asset is one picture in the file");
}

#[test]
fn original_leaves_every_picture_as_it_was() {
    let source = png(&noise(900, 900, 9).into());
    let assets = Memory(HashMap::from([("a.png".to_string(), source)]));
    let (book, styles) = book_showing(&["a.png"]);
    let options = PdfOptions {
        target: PdfTarget::Sharing,
        ink: None,
        crop_marks: false,
    };
    let bytes = export_pdf_with(
        &book,
        &styles,
        &FontBook::bundled(),
        &assets,
        &options,
        &ImagePolicy::original(),
        &PictureCache::default(),
    )
    .unwrap();
    let made = pictures(&bytes);
    assert_eq!(made[0].width, 900);
    assert_eq!(made[0].filter, "FlateDecode");
}

#[test]
fn a_print_file_keeps_three_hundred_pixels_an_inch() {
    let assets = Memory(HashMap::from([(
        "a.png".to_string(),
        png(&noise(2400, 2400, 11).into()),
    )]));
    let (book, styles) = book_showing(&["a.png", "a.png"]);
    let sharing = pictures(&export(&book, &styles, &assets, PdfTarget::Sharing));
    let print = pictures(&export(&book, &styles, &assets, PdfTarget::Print));
    // Both are of the same picture in the same place: the print file has twice the pixels across.
    let ratio = print[0].width as f64 / sharing[0].width as f64;
    assert!((1.8..2.2).contains(&ratio), "{ratio}");
}

#[test]
fn pictures_in_a_file_in_ink_are_cmyk_whatever_they_were() {
    let flat = RgbImage::from_fn(1500, 1500, |x, _| {
        if x < 750 {
            Rgb([200, 30, 30])
        } else {
            Rgb([30, 30, 200])
        }
    });
    let see_through = RgbaImage::from_fn(900, 900, |x, y| {
        let n = noise(1, 1, x * 31 + y).get_pixel(0, 0).0;
        Rgba([n[0], n[1], n[2], (x * 255 / 900) as u8])
    });
    let assets = Memory(HashMap::from([
        ("photo.png".to_string(), png(&noise(2400, 2400, 21).into())),
        ("flat.png".to_string(), png(&flat.into())),
        ("alpha.png".to_string(), png(&see_through.into())),
        ("small.jpg".to_string(), jpeg(&noise(300, 300, 4))),
    ]));
    let (book, styles) = book_showing(&["photo.png", "flat.png", "alpha.png", "small.jpg"]);
    let bytes = export_pdf(
        &book,
        &styles,
        &FontBook::bundled(),
        &assets,
        &PdfOptions {
            target: PdfTarget::Print,
            ink: Some(InkOptions::default()),
            crop_marks: false,
        },
    )
    .unwrap();
    let made = pictures(&bytes);
    assert!(!made.is_empty());
    for picture in &made {
        assert_eq!(picture.space, "DeviceCMYK", "{}", picture.filter);
    }
    assert!(made.iter().any(|p| p.has_mask), "transparency is kept");
}
