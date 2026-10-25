// Writes a sample book of a kind to a PDF, to look at: `cargo run -p folio-export --example sample -- picture out.pdf [print|shop]`
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use folio_export::pdf::{export_pdf, NoAssets, PdfOptions, PdfTarget};
use folio_layout::{FontBook, StyleSheet};
use folio_model::BookKind;
use folio_presets::{new_book, NewBookSpec};

fn main() {
    let mut args = std::env::args().skip(1);
    let kind = match args.next().as_deref() {
        Some("novel") => BookKind::Novel,
        Some("textbook") => BookKind::Textbook,
        Some("illustrated") => BookKind::Illustrated,
        Some("poetry") => BookKind::Poetry,
        Some("memoir") => BookKind::Memoir,
        _ => BookKind::Picture,
    };
    let out = args.next().unwrap_or_else(|| "sample.pdf".into());
    let target = match args.next().as_deref() {
        Some("print") => PdfTarget::Print,
        Some("shop") => PdfTarget::PrintShop,
        _ => PdfTarget::Sharing,
    };
    let made = new_book(&NewBookSpec {
        kind,
        ..NewBookSpec::default()
    })
    .expect("a sample book");
    let styles = StyleSheet::from_styles(&made.book.styles);
    let fonts = FontBook::bundled();
    let bytes = export_pdf(
        &made.book,
        &styles,
        &fonts,
        &NoAssets,
        &PdfOptions {
            target,
            ink: None,
            crop_marks: target == PdfTarget::PrintShop,
        },
    )
    .expect("the book exports");
    std::fs::write(&out, bytes).expect("the file is written");
    println!("wrote {out}");
}
