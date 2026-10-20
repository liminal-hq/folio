// Writes a sample book of a kind to an EPUB, to check with EPUBCheck: `cargo run -p folio-export --example sample_epub -- novel out.epub [kobo|fixed]`
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use folio_export::epub::{export_epub, EpubFlavour, EpubOptions};
use folio_export::fixed::export_fixed_epub;
use folio_export::pdf::NoAssets;
use folio_layout::{FontBook, StyleSheet};
use folio_model::BookKind;
use folio_presets::{new_book, NewBookSpec};

fn main() {
    let mut args = std::env::args().skip(1);
    let kind = match args.next().as_deref() {
        Some("picture") => BookKind::Picture,
        Some("textbook") => BookKind::Textbook,
        Some("illustrated") => BookKind::Illustrated,
        Some("poetry") => BookKind::Poetry,
        Some("memoir") => BookKind::Memoir,
        _ => BookKind::Novel,
    };
    let out = args.next().unwrap_or_else(|| "sample.epub".into());
    let mode = args.next();
    let flavour = if mode.as_deref() == Some("kobo") {
        EpubFlavour::Kobo
    } else {
        EpubFlavour::Standard
    };
    let made = new_book(&NewBookSpec {
        kind,
        ..NewBookSpec::default()
    })
    .expect("a sample book");
    if mode.as_deref() == Some("fixed") {
        let styles = StyleSheet::from_styles(&made.book.styles);
        let bytes = export_fixed_epub(&made.book, &styles, &FontBook::bundled(), &NoAssets)
            .expect("the book exports");
        std::fs::write(&out, bytes).expect("the file is written");
        println!("wrote {out}");
        return;
    }
    let bytes = export_epub(
        &made.book,
        &FontBook::bundled(),
        &NoAssets,
        &EpubOptions {
            flavour,
            embed_fonts: true,
        },
    )
    .expect("the book exports");
    std::fs::write(&out, bytes).expect("the file is written");
    println!("wrote {out}");
}
