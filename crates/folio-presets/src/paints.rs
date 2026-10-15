// The colours and gradients the pickers offer, as data
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT
//
// Page backgrounds and shape fills draw from one palette (D-23), written here once so the UI keeps
// no copy (D-18). A gradient preset is explicit stops, so what is stored in a book never depends on
// a preset's name.

use folio_model::{Colour, GradientStop};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

const PAINTS_JSON: &str = include_str!("../data/paints.json");

/// A named colour.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub struct Swatch {
    pub name: String,
    pub colour: Colour,
}

/// A named gradient: `angle` is in degrees clockwise from "up" (0 runs from the bottom to the top,
/// 180 from the top to the bottom).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub struct GradientPreset {
    pub name: String,
    pub angle: f64,
    pub stops: Vec<GradientStop>,
}

/// A named texture. The id is one of `folio_model::TEXTURE_IDS`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub struct TexturePreset {
    pub id: String,
    pub name: String,
}

/// What the colour and gradient pickers offer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub struct PaintPresets {
    pub swatches: Vec<Swatch>,
    pub gradients: Vec<GradientPreset>,
    pub textures: Vec<TexturePreset>,
}

/// The palette, in the order it is offered.
pub fn paint_presets() -> PaintPresets {
    serde_json::from_str(PAINTS_JSON).expect("data/paints.json is valid")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_colour_is_a_valid_hex_colour_and_stops_run_in_order() {
        let paints = paint_presets();
        assert_eq!(paints.swatches.len(), 8);
        assert_eq!(paints.gradients.len(), 5);
        for swatch in &paints.swatches {
            assert_eq!(Colour::parse(&swatch.colour.0).as_ref(), Ok(&swatch.colour));
        }
        let ids: Vec<_> = paints.textures.iter().map(|t| t.id.as_str()).collect();
        assert_eq!(
            ids,
            folio_model::TEXTURE_IDS,
            "every texture is offered, in order"
        );
        for gradient in &paints.gradients {
            assert!(gradient.stops.len() >= 2, "{}", gradient.name);
            assert!(gradient.stops.windows(2).all(|w| w[0].at <= w[1].at));
            for stop in &gradient.stops {
                assert!((0.0..=1.0).contains(&stop.at));
                assert_eq!(Colour::parse(&stop.colour.0).as_ref(), Ok(&stop.colour));
            }
        }
    }
}
