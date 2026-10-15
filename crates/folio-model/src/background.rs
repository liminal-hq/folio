// What fills a page behind its frames: a colour, a gradient, a texture or a picture
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::book::Master;
use crate::colour::{Colour, GradientStop};
use crate::spread::Spread;

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
    /// A blend along a line at `angle` degrees (0 runs left to right, 90 top to bottom). A preset
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

/// Where a page's background came from, nearest first.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
#[serde(rename_all = "lowercase")]
pub enum BgSource {
    Page,
    Span,
    Master,
}

/// The background a page shows, and which of the three it is. A `Span` one is drawn once across
/// both pages and cut at each page's edge, so the page needs to know it is part of it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub struct ResolvedBackground {
    pub source: BgSource,
    pub bg: Background,
}

/// The background each page of `spread` shows: its own, else the one across the spread, else its
/// master's, else none (the paper). The TypeScript `resolveBackgrounds` is its twin, and both are
/// tested against `backgroundCases.json`.
pub fn resolve(masters: &[Master], spread: &Spread) -> Vec<Option<ResolvedBackground>> {
    spread
        .pages
        .iter()
        .map(|page| {
            let from = |source, bg: &Option<Background>| {
                bg.clone().map(|bg| ResolvedBackground { source, bg })
            };
            from(BgSource::Page, &page.bg)
                // A span needs two pages: a hand-edited book may carry one on a single page.
                .or_else(|| {
                    if spread.pages.len() > 1 {
                        from(BgSource::Span, &spread.bg_span)
                    } else {
                        None
                    }
                })
                .or_else(|| {
                    let master = masters
                        .iter()
                        .find(|m| Some(&m.id) == page.master.as_ref())?;
                    from(BgSource::Master, &master.bg)
                })
        })
        .collect()
}

impl Background {
    /// The background as it is stored, or why it is not one: colours are `#rrggbb` in lower case,
    /// a gradient has at least two stops in order along a finite line, a texture has an id, and a
    /// picture keeps a point of itself in view. `asset_ok` says whether a picture's path is one of
    /// the book's own assets.
    pub fn checked(&self, asset_ok: impl Fn(&str) -> bool) -> Result<Background, String> {
        let colour = |c: &Colour| Colour::parse(&c.0).map_err(|e| e.to_string());
        Ok(match self {
            Background::Colour { colour: c } => Background::Colour { colour: colour(c)? },
            Background::Gradient { angle, stops } => {
                if !angle.is_finite() {
                    return Err("a gradient needs a finite angle".into());
                }
                if stops.len() < 2 {
                    return Err("a gradient needs at least two colours".into());
                }
                let mut checked = Vec::with_capacity(stops.len());
                for s in stops {
                    if !(0.0..=1.0).contains(&s.at) {
                        return Err("a gradient stop sits between 0 and 1".into());
                    }
                    checked.push(GradientStop {
                        at: s.at,
                        colour: colour(&s.colour)?,
                    });
                }
                checked.sort_by(|a, b| a.at.total_cmp(&b.at));
                Background::Gradient {
                    angle: *angle,
                    stops: checked,
                }
            }
            Background::Texture { id, tint } => {
                let named = !id.is_empty()
                    && id.len() <= 32
                    && id
                        .chars()
                        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-');
                if !named {
                    return Err(format!("`{id}` is not a texture"));
                }
                Background::Texture {
                    id: id.clone(),
                    tint: tint.as_ref().map(colour).transpose()?,
                }
            }
            Background::Picture { asset, fit, focus } => {
                if !asset_ok(asset) {
                    return Err(format!("`{asset}` is not a picture in this book's assets"));
                }
                if ![focus.0, focus.1].iter().all(|v| (0.0..=1.0).contains(v)) {
                    return Err("a picture's focus sits within the page".into());
                }
                Background::Picture {
                    asset: asset.clone(),
                    fit: *fit,
                    focus: *focus,
                }
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stop(at: f64, c: &str) -> GradientStop {
        GradientStop {
            at,
            colour: Colour(c.into()),
        }
    }

    #[test]
    fn checking_normalises_colours_and_orders_gradient_stops() {
        let g = Background::Gradient {
            angle: 90.0,
            stops: vec![stop(1.0, "#FFF"), stop(0.0, "#000000")],
        };
        assert_eq!(
            g.checked(|_| true).unwrap(),
            Background::Gradient {
                angle: 90.0,
                stops: vec![stop(0.0, "#000000"), stop(1.0, "#ffffff")]
            }
        );
    }

    #[test]
    fn checking_refuses_what_cannot_be_drawn() {
        let bad = [
            Background::Colour {
                colour: Colour("red".into()),
            },
            Background::Gradient {
                angle: 0.0,
                stops: vec![stop(0.0, "#000")],
            },
            Background::Gradient {
                angle: f64::NAN,
                stops: vec![stop(0.0, "#000"), stop(1.0, "#fff")],
            },
            Background::Gradient {
                angle: 0.0,
                stops: vec![stop(0.0, "#000"), stop(1.5, "#fff")],
            },
            Background::Texture {
                id: "Linen!".into(),
                tint: None,
            },
            Background::Picture {
                asset: "../x".into(),
                fit: BgFit::Fill,
                focus: (0.5, 0.5),
            },
            Background::Picture {
                asset: "ok".into(),
                fit: BgFit::Fill,
                focus: (2.0, 0.5),
            },
        ];
        for b in bad {
            assert!(b.checked(|a| a == "ok").is_err(), "{b:?}");
        }
    }

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

#[cfg(test)]
mod resolve_tests {
    use super::*;

    /// Where the cases are kept: next to the TypeScript twin that is tested against them.
    fn cases() -> serde_json::Value {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../apps/folio/src/canvas/fixtures/backgroundCases.json");
        serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
    }

    #[test]
    fn every_shared_case_resolves_as_the_fixture_says() {
        for case in cases().as_array().unwrap() {
            let masters: Vec<Master> = serde_json::from_value(case["masters"].clone()).unwrap();
            let spread: Spread = serde_json::from_value(case["spread"].clone()).unwrap();
            let got = serde_json::to_value(resolve(&masters, &spread)).unwrap();
            assert_eq!(got, case["expected"], "{}", case["name"]);
        }
    }
}
