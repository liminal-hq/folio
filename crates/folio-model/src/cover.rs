// Cover maths for KDP paperbacks: spine, wrap and barcode area
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::units::{Pt, Rect, Size};

// ── Export maths ──
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
#[serde(rename_all = "lowercase")]
pub enum Paper {
    #[default]
    White,
    Cream,
    Colour,
}

/// KDP paperback spine factor, inches per page. ⚠ Verify against KDP's current cover calculator.
pub fn spine_factor(p: Paper) -> f64 {
    match p {
        Paper::White => 0.002252,
        Paper::Cream => 0.0025,
        Paper::Colour => 0.002347,
    }
}

#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub struct CoverMaths {
    pub spine: Pt,
    pub edge: Pt,
    pub width: Pt,
    pub height: Pt,
    pub spine_text_allowed: bool,
    pub barcode: Rect,
}

/// Who makes the barcode on the back of the paperback cover.
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum BarcodeChoice {
    /// Folio draws the barcode when the paperback ISBN is a good one, and leaves the box clear for
    /// KDP to fill when it is not.
    #[default]
    Auto,
    /// The box is kept clear, and KDP adds its own barcode.
    Kdp,
    /// Folio draws an EAN-13 from the ISBN, with a five-digit price add-on when `addon` is set
    /// (`90000` for no price; `5` and four digits for US dollars, `6` and four for Canadian).
    Own {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        addon: Option<String>,
    },
}

impl BarcodeChoice {
    /// Whether a price add-on is five digits, as the EAN-5 it is drawn as needs.
    pub fn addon_is_valid(addon: &str) -> bool {
        addon.len() == 5 && addon.chars().all(|c| c.is_ascii_digit())
    }
}

/// KDP puts text on the spine of a book of this many pages or more. Its own pages disagree between
/// 79 and 80, so Folio takes the stricter (D-34).
pub const SPINE_TEXT_FROM: u32 = 80;

/// How many pages KDP counts for the spine: the interior, in twos, and at least 24.
pub fn spine_pages(book: &crate::book::Book) -> u32 {
    let interior: u32 = book
        .spreads
        .iter()
        .skip(1)
        .map(|s| s.pages.len() as u32)
        .sum();
    (interior + interior % 2).max(24)
}

pub fn paperback_cover(trim: Size, pages: u32, paper: Paper) -> CoverMaths {
    let spine = Pt::from_in(pages as f64 * spine_factor(paper));
    let edge = Pt::from_in(0.125);
    let width = Pt(edge.0 * 2.0 + trim.w.0 * 2.0 + spine.0);
    let height = Pt(edge.0 * 2.0 + trim.h.0);
    // 2 × 1.2 in barcode box, 0.25 in in from the back cover's bottom-right trim corner.
    let barcode = Rect {
        x: Pt(edge.0 + trim.w.0 - Pt::from_in(2.25).0),
        y: Pt(edge.0 + trim.h.0 - Pt::from_in(1.45).0),
        w: Pt::from_in(2.0),
        h: Pt::from_in(1.2),
    };
    CoverMaths {
        spine,
        edge,
        width,
        height,
        spine_text_allowed: pages >= SPINE_TEXT_FROM,
        barcode,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_32_page_square_picture_book_has_a_spine_too_thin_for_text() {
        let cover = paperback_cover(
            Size {
                w: Pt::from_in(8.5),
                h: Pt::from_in(8.5),
            },
            32,
            Paper::White,
        );
        assert!((cover.spine.to_in() - 0.072064).abs() < 1e-6);
        assert!((cover.width.to_in() - 17.322064).abs() < 1e-6);
        assert!((cover.height.to_in() - 8.75).abs() < 1e-9);
        assert!(!cover.spine_text_allowed);
    }

    #[test]
    fn the_spine_takes_text_from_80_pages() {
        let trim = Size {
            w: Pt::from_in(5.0),
            h: Pt::from_in(8.0),
        };
        assert!(!paperback_cover(trim, 79, Paper::Cream).spine_text_allowed);
        assert!(paperback_cover(trim, 80, Paper::Cream).spine_text_allowed);
    }

    #[test]
    fn the_barcode_sits_in_the_back_covers_bottom_right_corner() {
        let trim = Size {
            w: Pt::from_in(6.0),
            h: Pt::from_in(9.0),
        };
        let cover = paperback_cover(trim, 100, Paper::White);
        // The box is 2 x 1.2 in and ends 0.25 in in from the back cover's trim corner.
        let right = cover.barcode.x.0 + cover.barcode.w.0;
        let bottom = cover.barcode.y.0 + cover.barcode.h.0;
        assert!((right - (cover.edge.0 + trim.w.0 - Pt::from_in(0.25).0)).abs() < 1e-9);
        assert!((bottom - (cover.edge.0 + trim.h.0 - Pt::from_in(0.25).0)).abs() < 1e-9);
    }
}
