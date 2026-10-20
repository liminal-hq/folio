// Writes the eBook cover of a sample book to a JPEG, to look at: `cargo run -p folio-export --example sample_ebook_cover -- out.jpg [#colour]`
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use folio_export::job::{export_target, Target};
use folio_export::pdf::NoAssets;
use folio_layout::{FontBook, StyleSheet};
use folio_model::{BookKind, Paper};
use folio_presets::{new_book, NewBookSpec};

fn main() {
    let mut args = std::env::args().skip(1);
    let out = args.next().unwrap_or_else(|| "cover.jpg".into());
    let colour = args.next().unwrap_or_else(|| "#5b2a4a".into());
    let made = new_book(&NewBookSpec {
        kind: BookKind::Picture,
        cover_colour: colour,
        ..NewBookSpec::default()
    })
    .expect("a sample book");
    let styles = StyleSheet::from_styles(&made.book.styles);
    let files = export_target(
        Target::EbookCover,
        Paper::White,
        &made.book,
        &styles,
        &FontBook::bundled(),
        &NoAssets,
    )
    .expect("the cover exports");
    std::fs::write(&out, &files[0].bytes).expect("the file is written");
    println!("wrote {out} ({} bytes)", files[0].bytes.len());
}
