// A spread as a press would print it: each page drawn, then sent through the press's profile
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use crate::error::ExportError;
use crate::pdf::Assets;
use crate::raster::render_page;
use folio_colour::{Intent, Profile, Proof};
use folio_layout::{FontBook, Layouter, StyleSheet};
use folio_model::{Book, SpreadId};

/// One page, as drawn and as the press would print it.
pub struct ProofPage {
    /// The page as drawn, a PNG.
    pub original: Vec<u8>,
    /// The page through the press's profile, a PNG.
    pub proofed: Vec<u8>,
    /// How much of the page, in per cent, is a colour the press cannot reproduce.
    pub outside: f64,
}

/// Draws the pages of one spread `width` pixels wide each, original and proofed for the press whose
/// ICC profile is `press`. `None` when the book has no such spread.
pub fn proof_spread(
    book: &Book,
    styles: &StyleSheet,
    fonts: &FontBook,
    assets: &dyn Assets,
    spread: &SpreadId,
    press: &[u8],
    width: u32,
) -> Result<Option<Vec<ProofPage>>, ExportError> {
    let profile = Profile::from_icc(press).map_err(|e| ExportError::Pdf(e.to_string()))?;
    let proof =
        Proof::new(&profile, Intent::Perceptual).map_err(|e| ExportError::Pdf(e.to_string()))?;
    let Some(scene) = Layouter::new(book, styles, fonts).scene_spread(spread)? else {
        return Ok(None);
    };
    let mut pages = Vec::new();
    for page in &scene.pages {
        let height = (f64::from(width) * page.height / page.width)
            .round()
            .max(1.0) as u32;
        let mut pixmap = render_page(page, fonts, assets, width, height)?;
        let original = pixmap
            .encode_png()
            .map_err(|e| ExportError::Pdf(e.to_string()))?;
        // The pixmap holds premultiplied pixels and the page is opaque, so the colours are as drawn.
        let outside = proof.pixels(pixmap.data_mut());
        let total = (width * height) as f64;
        pages.push(ProofPage {
            original,
            proofed: pixmap
                .encode_png()
                .map_err(|e| ExportError::Pdf(e.to_string()))?,
            outside: outside as f64 * 100.0 / total,
        });
    }
    Ok(Some(pages))
}

/// One of the book's swatches, as it is drawn and as the press would print it.
pub struct ProofColour {
    pub name: String,
    /// The colour drawn, `#rrggbb`.
    pub display: String,
    /// The colour through the press's profile, `#rrggbb`.
    pub proofed: String,
    /// Whether the press cannot reproduce it.
    pub outside: bool,
}

/// The book's swatches through the press whose ICC profile is `press`.
pub fn proof_colours(book: &Book, press: &[u8]) -> Result<Vec<ProofColour>, ExportError> {
    let profile = Profile::from_icc(press).map_err(|e| ExportError::Pdf(e.to_string()))?;
    let proof =
        Proof::new(&profile, Intent::Perceptual).map_err(|e| ExportError::Pdf(e.to_string()))?;
    Ok(book
        .styles
        .swatches
        .iter()
        .filter_map(|swatch| {
            let (r, g, b) = swatch.display.rgb().map(|[r, g, b]| (r, g, b))?;
            let shown = proof.colour((r, g, b));
            Some(ProofColour {
                name: swatch.name.clone(),
                display: format!("#{r:02x}{g:02x}{b:02x}"),
                proofed: format!("#{:02x}{:02x}{:02x}", shown.rgb.0, shown.rgb.1, shown.rgb.2),
                outside: shown.out_of_gamut,
            })
        })
        .collect())
}
