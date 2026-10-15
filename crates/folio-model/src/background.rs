// What fills a page behind its frames: a colour, a gradient, a texture or a picture
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::colour::{Colour, GradientStop};

/// How a picture fills the page: covering it (and cropped at the edges) or whole inside it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
#[serde(rename_all = "lowercase")]
pub enum BgFit {
    Fill,
    Fit,
}

/// A page's background. A page shows its own, else the one across its spread, else its master's,
/// else the paper (D-23). It reaches into the bleed on the outer edge of the page.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Background {
    /// One flat colour.
    Colour { colour: Colour },
    /// A blend along a line at `angle` degrees clockwise from up, as in CSS (0 runs bottom to top, 90
    /// left to right, 180 top to bottom). A preset
    /// is stored as the stops it expands to, so a book never depends on the palette it was made with.
    Gradient {
        angle: f64,
        stops: Vec<GradientStop>,
    },
    /// One of the textures Folio ships, by id, optionally tinted.
    Texture {
        id: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        tint: Option<Colour>,
    },
    /// A picture from the book's assets, with the point of it to keep in view when it is cropped.
    Picture {
        asset: String,
        fit: BgFit,
        #[serde(default = "centre")]
        focus: (f64, f64),
    },
}

fn centre() -> (f64, f64) {
    (0.5, 0.5)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_kind_is_tagged_by_its_type() {
        let colour = Background::Colour {
            colour: Colour("#fdf6e3".into()),
        };
        assert_eq!(
            serde_json::to_value(&colour).unwrap(),
            serde_json::json!({ "type": "colour", "colour": "#fdf6e3" })
        );
        let texture: Background =
            serde_json::from_value(serde_json::json!({ "type": "texture", "id": "linen" }))
                .unwrap();
        assert_eq!(
            texture,
            Background::Texture {
                id: "linen".into(),
                tint: None
            }
        );
    }

    #[test]
    fn a_picture_with_no_focus_is_centred() {
        let picture: Background = serde_json::from_value(
            serde_json::json!({ "type": "picture", "asset": "a1", "fit": "fill" }),
        )
        .unwrap();
        assert_eq!(
            picture,
            Background::Picture {
                asset: "a1".into(),
                fit: BgFit::Fill,
                focus: (0.5, 0.5)
            }
        );
    }
}
