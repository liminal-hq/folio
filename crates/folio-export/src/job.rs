// What can be exported, what each file is called, and making one
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use folio_layout::{FontBook, StyleSheet};
use folio_model::{Book, Paper};

use crate::cover::cover_sheet;
use crate::epub::{export_epub, EpubFlavour, EpubOptions};
use crate::error::ExportError;
use crate::fixed::export_fixed_epub;
use crate::pdf::{export_pdf, Assets, InkOptions, PdfOptions, PdfTarget};
use crate::raster::page_to_jpeg;

/// A kind of file to make from a book.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Target {
    /// A PDF to read on a screen or send round.
    SharingPdf,
    /// The interior of a paperback, as Kindle Direct Publishing asks for it.
    KdpInterior,
    /// Every page for a print shop: bleed all round, crop marks, PDF/X-4.
    PrintShopPdf,
    /// The paperback cover for Kindle Direct Publishing: back, spine and front on one sheet.
    KdpCover,
    /// The cover as a picture for the book's page in a store: 1600 × 2560 pixels.
    EbookCover,
    /// A reflowable EPUB for Kindle Direct Publishing, which converts it for Kindle.
    KindleEpub,
    /// A fixed-layout EPUB: every page as laid out, for a picture book.
    FixedEpub,
    /// A reflowable EPUB for Kobo Writing Life, and the Kobo flavour of it beside it.
    KoboEpub,
}

/// A file made from a book, not yet written anywhere.
#[derive(Debug, Clone)]
pub struct Exported {
    pub target: Target,
    pub file_name: String,
    pub bytes: Vec<u8>,
}

/// `title` as a file name stem: lower case, letters and digits, one hyphen between words.
pub fn slug(title: &str) -> String {
    let mut out = String::new();
    let mut gap = false;
    for c in title.chars() {
        if c.is_alphanumeric() {
            if gap && !out.is_empty() {
                out.push('-');
            }
            gap = false;
            out.extend(c.to_lowercase());
        } else if c == '\'' || c == '’' {
            // "Couldn't" is one word.
        } else {
            gap = true;
        }
    }
    if out.is_empty() {
        "book".into()
    } else {
        out
    }
}

impl Target {
    /// The files this target makes for a book called `title`.
    pub fn file_names(self, title: &str) -> Vec<String> {
        let slug = slug(title);
        match self {
            Self::SharingPdf => vec![format!("{slug}-share.pdf")],
            Self::KdpInterior => vec![format!("{slug}-paperback-interior.pdf")],
            Self::PrintShopPdf => vec![format!("{slug}-print-pdfx4.pdf")],
            Self::KdpCover => vec![format!("{slug}-paperback-cover.pdf")],
            Self::EbookCover => vec![format!("{slug}-ebook-cover.jpg")],
            Self::KindleEpub => vec![format!("{slug}-kindle.epub")],
            // The same book twice: a Kobo picks its renderer for a sideloaded file by the name, and only
            // the `.fxl.kepub.epub` one is its fixed-layout renderer.
            Self::FixedEpub => vec![
                format!("{slug}-fixed.epub"),
                format!("{slug}-fixed.fxl.kepub.epub"),
            ],
            Self::KoboEpub => vec![format!("{slug}.epub"), format!("{slug}.kepub.epub")],
        }
    }
}

/// Makes the files for `target`.
pub fn export_target(
    target: Target,
    paper: Paper,
    book: &Book,
    styles: &StyleSheet,
    fonts: &FontBook,
    assets: &dyn Assets,
    ink: Option<&InkOptions>,
) -> Result<Vec<Exported>, ExportError> {
    // Everything but a file in ink draws a swatch as the colour it shows as; a file in ink keeps the
    // references, so each swatch can print as its exact ink.
    let mut shown = book.clone();
    folio_model::resolve_swatches(&mut shown);
    let (resolved, book_in_ink) = (&shown, book);
    let book = resolved;
    let pdf = |kind, ink: Option<InkOptions>| {
        export_pdf(
            if ink.is_some() { book_in_ink } else { book },
            styles,
            fonts,
            assets,
            &PdfOptions {
                target: kind,
                ink,
                crop_marks: kind == PdfTarget::PrintShop,
            },
        )
    };
    let epub = |flavour| {
        export_epub(
            book,
            fonts,
            assets,
            &EpubOptions {
                flavour,
                embed_fonts: true,
            },
        )
    };
    let names = target.file_names(&book.title);
    let made: Vec<Vec<u8>> = match target {
        Target::SharingPdf => vec![pdf(PdfTarget::Sharing, None)?],
        Target::KdpInterior => vec![pdf(PdfTarget::Print, ink.cloned())?],
        Target::PrintShopPdf => vec![pdf(PdfTarget::PrintShop, None)?],
        Target::KdpCover => {
            let sheet = cover_sheet(book_in_ink, paper).ok_or(ExportError::NoPages)?;
            vec![export_pdf(
                &sheet,
                styles,
                fonts,
                assets,
                &PdfOptions {
                    target: PdfTarget::Cover,
                    // KDP wants the cover in CMYK, untagged, with no spot colours: through the
                    // chosen press profile when there is one, by a plain separation when not.
                    ink: Some(ink.cloned().unwrap_or_default()),
                    crop_marks: false,
                },
            )?]
        }
        Target::EbookCover => {
            let front = folio_model::front_cover_view(book);
            let cover = front.spreads.first().ok_or(ExportError::NoPages)?;
            let layouter = folio_layout::Layouter::new(&front, styles, fonts);
            let scene = layouter
                .scene_spread(&cover.id)?
                .and_then(|s| s.pages.into_iter().next())
                .ok_or(ExportError::NoPages)?;
            vec![page_to_jpeg(&scene, fonts, assets, 1600, 2560)?]
        }
        Target::KindleEpub => vec![epub(EpubFlavour::Standard)?],
        Target::FixedEpub => {
            let fixed = export_fixed_epub(book, styles, fonts, assets)?;
            vec![fixed.clone(), fixed]
        }
        Target::KoboEpub => vec![epub(EpubFlavour::Standard)?, epub(EpubFlavour::Kobo)?],
    };
    Ok(names
        .into_iter()
        .zip(made)
        .map(|(file_name, bytes)| Exported {
            target,
            file_name,
            bytes,
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_title_becomes_a_lower_case_slug() {
        assert_eq!(
            slug("The Otter Who Couldn't Sleep"),
            "the-otter-who-couldnt-sleep"
        );
        assert_eq!(slug("  What? Who: me/you  "), "what-who-me-you");
        assert_eq!(slug("Crème brûlée"), "crème-brûlée");
        assert_eq!(slug("???"), "book");
    }

    #[test]
    fn each_target_names_its_files() {
        assert_eq!(Target::SharingPdf.file_names("Pip"), ["pip-share.pdf"]);
        assert_eq!(
            Target::KdpInterior.file_names("Pip"),
            ["pip-paperback-interior.pdf"]
        );
        assert_eq!(Target::KindleEpub.file_names("Pip"), ["pip-kindle.epub"]);
        assert_eq!(
            Target::FixedEpub.file_names("Pip"),
            ["pip-fixed.epub", "pip-fixed.fxl.kepub.epub"]
        );
        assert_eq!(
            Target::KdpCover.file_names("Pip"),
            ["pip-paperback-cover.pdf"]
        );
        assert_eq!(
            Target::KoboEpub.file_names("Pip"),
            ["pip.epub", "pip.kepub.epub"]
        );
    }
}
