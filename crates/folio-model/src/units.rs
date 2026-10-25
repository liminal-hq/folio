// Lengths in points, and the sizes, rectangles and margins built from them
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// A length in points (1/72 in), the unit every stored length uses. Folios show inches or
/// millimetres, never points, unless the user picks them.
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
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

#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub struct Size {
    pub w: Pt,
    pub h: Pt,
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub struct Rect {
    pub x: Pt,
    pub y: Pt,
    pub w: Pt,
    pub h: Pt,
}
impl Rect {
    /// The smallest upright rectangle holding this one after it is turned `degrees` clockwise about
    /// its middle: where a turned frame is for snapping and for print checks.
    pub fn turned_bounds(&self, degrees: f64) -> Rect {
        let (sin, cos) = degrees.to_radians().sin_cos();
        let (w, h) = (self.w.0, self.h.0);
        let (bw, bh) = (w * cos.abs() + h * sin.abs(), w * sin.abs() + h * cos.abs());
        let (cx, cy) = (self.x.0 + w / 2.0, self.y.0 + h / 2.0);
        // A quarter turn leaves the sums exact; a stray 1e-15 would only be noise on the page.
        let tidy = |n: f64| (n * 1e9).round() / 1e9;
        Rect {
            x: Pt(tidy(cx - bw / 2.0)),
            y: Pt(tidy(cy - bh / 2.0)),
            w: Pt(tidy(bw)),
            h: Pt(tidy(bh)),
        }
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub struct Margins {
    pub top: Pt,
    pub bottom: Pt,
    pub inside: Pt,
    pub outside: Pt,
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
    fn a_turned_rectangle_is_held_by_the_box_around_its_corners() {
        let r = Rect {
            x: Pt(100.0),
            y: Pt(50.0),
            w: Pt(40.0),
            h: Pt(10.0),
        };
        assert_eq!(r.turned_bounds(0.0), r);
        // A quarter turn swaps the sides about the same middle.
        assert_eq!(
            r.turned_bounds(90.0),
            Rect {
                x: Pt(115.0),
                y: Pt(35.0),
                w: Pt(10.0),
                h: Pt(40.0),
            }
        );
        assert_eq!(r.turned_bounds(-90.0), r.turned_bounds(270.0));
        // A half turn is the same box, and 45° widens both sides to (w + h) / √2.
        assert_eq!(r.turned_bounds(180.0), r);
        let d = r.turned_bounds(45.0);
        assert!((d.w.0 - 50.0 / 2f64.sqrt()).abs() < 1e-6 && (d.h.0 - d.w.0).abs() < 1e-6);
        assert!((d.x.0 + d.w.0 / 2.0 - 120.0).abs() < 1e-6);
    }

    #[test]
    fn a_length_is_a_bare_number_on_the_wire() {
        assert_eq!(serde_json::to_string(&Pt(36.0)).unwrap(), "36.0");
    }
}
