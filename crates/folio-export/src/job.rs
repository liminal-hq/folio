// What can be exported, what each file is called, and making one
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use folio_layout::{FontBook, StyleSheet};
use folio_model::Book;

use crate::error::ExportError;
use crate::pdf::{export_pdf, Assets, PdfOptions, PdfTarget};

/// A kind of file to make from a book.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Target {
    /// A PDF to read on a screen or send round.
    SharingPdf,
    /// The interior of a paperback, as Kindle Direct Publishing asks for it.
    KdpInterior,
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
    /// The file this target makes for a book called `title`.
    pub fn file_name(self, title: &str) -> String {
        let slug = slug(title);
        match self {
            Self::SharingPdf => format!("{slug}-share.pdf"),
            Self::KdpInterior => format!("{slug}-paperback-interior.pdf"),
        }
    }
}

/// Makes the file for `target`.
pub fn export_target(
    target: Target,
    book: &Book,
    styles: &StyleSheet,
    fonts: &FontBook,
    assets: &dyn Assets,
) -> Result<Exported, ExportError> {
    let pdf = |kind| export_pdf(book, styles, fonts, assets, &PdfOptions { target: kind });
    let bytes = match target {
        Target::SharingPdf => pdf(PdfTarget::Sharing)?,
        Target::KdpInterior => pdf(PdfTarget::Print)?,
    };
    Ok(Exported {
        target,
        file_name: target.file_name(&book.title),
        bytes,
    })
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
    fn each_target_names_its_file() {
        assert_eq!(Target::SharingPdf.file_name("Pip"), "pip-share.pdf");
        assert_eq!(
            Target::KdpInterior.file_name("Pip"),
            "pip-paperback-interior.pdf"
        );
    }
}
