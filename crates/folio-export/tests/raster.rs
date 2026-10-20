// Tests for the picture of the cover: the size a store asks for, the cover's colour and its words
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use folio_export::job::{export_target, Target};
use folio_export::pdf::NoAssets;
use folio_layout::{FontBook, StyleSheet};
use folio_model::{BookKind, Paper};
use folio_presets::{new_book, NewBookSpec};

fn cover_jpeg(kind: BookKind, colour: &str) -> image::RgbImage {
    let made = new_book(&NewBookSpec {
        kind,
        cover_colour: colour.into(),
        ..NewBookSpec::default()
    })
    .unwrap();
    let styles = StyleSheet::from_styles(&made.book.styles);
    let files = export_target(
        Target::EbookCover,
        Paper::White,
        &made.book,
        &styles,
        &FontBook::bundled(),
        &NoAssets,
    )
    .unwrap();
    assert_eq!(files.len(), 1);
    assert_eq!(
        files[0].file_name,
        "the-otter-who-couldnt-sleep-ebook-cover.jpg"
    );
    assert!(files[0].bytes.starts_with(&[0xff, 0xd8]), "a JPEG");
    image::load_from_memory(&files[0].bytes).unwrap().to_rgb8()
}

#[test]
fn the_cover_picture_is_1600_by_2560_and_under_the_five_megabytes_stores_allow() {
    let picture = cover_jpeg(BookKind::Picture, "#5b2a4a");
    assert_eq!((picture.width(), picture.height()), (1600, 2560));
}

#[test]
fn it_is_the_covers_colour_with_the_title_drawn_over_it() {
    let picture = cover_jpeg(BookKind::Picture, "#1f4e5f");
    let near = |p: &image::Rgb<u8>, want: (u8, u8, u8)| {
        (i16::from(p[0]) - i16::from(want.0)).abs() < 40
            && (i16::from(p[1]) - i16::from(want.1)).abs() < 40
            && (i16::from(p[2]) - i16::from(want.2)).abs() < 40
    };
    // The corners are the cover's own colour.
    assert!(near(picture.get_pixel(10, 10), (0x1f, 0x4e, 0x5f)));
    assert!(near(picture.get_pixel(1589, 2549), (0x1f, 0x4e, 0x5f)));
    // The title is drawn in the page's ink, which is darker than the colour behind it.
    let darker = picture
        .pixels()
        .filter(|p| u32::from(p[0]) + u32::from(p[1]) + u32::from(p[2]) < 90)
        .count();
    assert!(
        darker > 2_000,
        "the title and byline are drawn ({darker} dark pixels)"
    );
}

#[test]
fn a_light_cover_gets_a_thin_grey_border_and_a_dark_one_does_not() {
    let light = cover_jpeg(BookKind::Picture, "#f7f3ea");
    let edge = light.get_pixel(1, 1);
    assert!(edge[0] < 200 && edge[0] > 90, "a grey edge: {edge:?}");
    let dark = cover_jpeg(BookKind::Picture, "#1f4e5f");
    let edge = dark.get_pixel(1, 1);
    assert!(edge[0] < 80, "no border on a dark cover: {edge:?}");
}
