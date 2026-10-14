// The document model's first slice: lengths in points and the kinds of book. The rest of the model
// in `spec/types/project.rs` arrives with the document work.
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// A length in points (1/72 in), the unit every stored length uses. Folios show inches or
/// millimetres, never points, unless the user picks them.
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/protocol/src/generated/",
    type = "number"
)]
pub struct Pt(pub f64);

impl Pt {
    /// Points in one inch.
    pub const PER_INCH: f64 = 72.0;

    pub fn from_in(inches: f64) -> Self {
        Pt(inches * Self::PER_INCH)
    }

    pub fn from_mm(millimetres: f64) -> Self {
        Pt(millimetres * Self::PER_INCH / 25.4)
    }

    pub fn to_in(self) -> f64 {
        self.0 / Self::PER_INCH
    }

    pub fn to_mm(self) -> f64 {
        self.0 * 25.4 / Self::PER_INCH
    }
}

/// The kind of book, which selects a preset bundle of trim, bleed, grid, styles and export defaults.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub enum BookKind {
    Picture,
    Novel,
    Textbook,
    Illustrated,
    Poetry,
    Memoir,
}

/// Whether a book's pages are laid out by hand (`Fixed`) or flow from a story (`Flowing`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub enum LayoutMode {
    Fixed,
    Flowing,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_between_points_inches_and_millimetres() {
        assert_eq!(Pt::from_in(8.5), Pt(612.0));
        assert!((Pt::from_mm(25.4).to_in() - 1.0).abs() < 1e-12);
        assert!((Pt::from_in(1.0).to_mm() - 25.4).abs() < 1e-12);
    }

    #[test]
    fn a_length_is_a_bare_number_on_the_wire() {
        assert_eq!(serde_json::to_string(&Pt(36.0)).unwrap(), "36.0");
    }

    #[test]
    fn kinds_use_lowercase_names() {
        assert_eq!(
            serde_json::to_string(&BookKind::Picture).unwrap(),
            "\"picture\""
        );
        assert_eq!(
            serde_json::to_string(&LayoutMode::Flowing).unwrap(),
            "\"flowing\""
        );
    }
}
