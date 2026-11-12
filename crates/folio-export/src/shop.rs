// What a print shop's PDF/X-4 can be asked for beyond the whole book
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use crate::pdf::InkOptions;

/// How much bleed the print shop's file has all round.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum ShopBleed {
    /// The book's own bleed.
    #[default]
    Book,
    /// This many points, whatever the book has: more than the book's shows the page's background
    /// and pictures that run to it, and nothing where the book has none.
    Custom(f64),
}

/// The room round the bleed for crop marks, in points, that a shop's file has when marks are on.
pub const DEFAULT_SLUG: f64 = 24.0;

/// The most and least room round the bleed that holds crop marks.
pub const SLUG_RANGE: (f64, f64) = (18.0, 72.0);

/// The print shop's options.
#[derive(Debug, Clone, Default)]
pub struct ShopOptions {
    pub bleed: ShopBleed,
    /// Room round the bleed for crop marks, in points; `None` is `DEFAULT_SLUG`.
    pub slug: Option<f64>,
    /// Make the file in ink through this press profile, with a CMYK output intent. A shop's file
    /// is sRGB with an sRGB output intent unless it has a profile to say what the ink is.
    pub ink: Option<InkOptions>,
}

impl ShopOptions {
    /// The bleed in points for a book whose own is `book`.
    pub fn bleed_in_points(&self, book: f64) -> f64 {
        match self.bleed {
            ShopBleed::Book => book.max(0.0),
            ShopBleed::Custom(points) => points.clamp(0.0, 36.0),
        }
    }

    /// The room for crop marks in points.
    pub fn slug_in_points(&self) -> f64 {
        self.slug
            .unwrap_or(DEFAULT_SLUG)
            .clamp(SLUG_RANGE.0, SLUG_RANGE.1)
    }
}
