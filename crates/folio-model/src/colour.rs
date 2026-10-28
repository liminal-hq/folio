// A colour as the hex string it is stored as, and the check an intent applies to it
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use serde::{Deserialize, Serialize};

use crate::ids::SwatchId;
use ts_rs::TS;

/// A colour: `#rrggbb`, lower case. It is stored as the string so a book stays readable, and so a
/// hand-edited or older book still opens: reading never fails on a colour. An intent checks it with
/// [`Colour::parse`] before it stores one.
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/protocol/src/generated/",
    type = "string"
)]
#[serde(transparent)]
pub struct Colour(pub String);

/// Ink as printed, in whole percents of each plate (0 to 100): cyan, magenta, yellow and black.
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub struct Ink {
    pub c: u8,
    pub m: u8,
    pub y: u8,
    pub k: u8,
}

impl Ink {
    /// Whether every plate is a percent.
    pub fn is_valid(&self) -> bool {
        [self.c, self.m, self.y, self.k].iter().all(|p| *p <= 100)
    }

    /// The total ink laid down, in percent: 400 is every plate solid.
    pub fn total(&self) -> u32 {
        u32::from(self.c) + u32::from(self.m) + u32::from(self.y) + u32::from(self.k)
    }

    /// An sRGB colour for showing the ink on a screen: the plain inverse, `(1 - plate)(1 - k)`.
    /// There is no press profile until export, so this is a guide and brighter than print.
    pub fn to_rgb(&self) -> [u8; 3] {
        let k = 1.0 - f64::from(self.k) / 100.0;
        let plate = |p: u8| (255.0 * (1.0 - f64::from(p) / 100.0) * k).round() as u8;
        [plate(self.c), plate(self.m), plate(self.y)]
    }
}

/// A colour a book names and keeps: shown as `display` everywhere, and printed as `ink` when it has
/// one. A field that refers to it as `swatch:<id>` follows it when it is changed (D-40).
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub struct BookSwatch {
    pub id: SwatchId,
    pub name: String,
    /// The sRGB colour the screen, an EPUB and an RGB file use.
    pub display: Colour,
    /// The exact ink a file made in ink prints it as.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ink: Option<Ink>,
}

/// A colour as a file is made from it: sRGB, and exact ink when a swatch gave it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Resolved {
    pub rgb: [u8; 3],
    pub ink: Option<Ink>,
}

/// The book's swatches, for turning a colour field into a [`Resolved`].
pub struct Swatches<'a>(pub &'a [BookSwatch]);

impl Swatches<'_> {
    /// What `colour` is: hex as it says, a swatch as the book defines it, and mid-grey for a
    /// reference to a swatch the book does not have, so a book never fails to open for one.
    pub fn resolve(&self, colour: &Colour) -> Resolved {
        match colour.swatch_id() {
            Some(id) => match self.0.iter().find(|s| s.id.0 == id) {
                Some(swatch) => Resolved {
                    rgb: swatch.display.rgb().unwrap_or([128; 3]),
                    ink: swatch.ink,
                },
                None => Resolved {
                    rgb: [128; 3],
                    ink: None,
                },
            },
            None => Resolved {
                rgb: colour.rgb().unwrap_or([0; 3]),
                ink: None,
            },
        }
    }
}

/// What was wrong with a colour that was not accepted.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BadColour(pub String);

impl std::fmt::Display for BadColour {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "`{}` is not a colour: write it as #rrggbb", self.0)
    }
}

impl std::error::Error for BadColour {}

impl Colour {
    /// A colour from `#rgb` or `#rrggbb` (either case), stored as `#rrggbb` in lower case, or a
    /// reference to a book swatch, `swatch:<id>` with an id of lower case letters, digits and `-`.
    pub fn parse(text: &str) -> Result<Colour, BadColour> {
        if let Some(id) = text.strip_prefix("swatch:") {
            let ok = !id.is_empty()
                && id.len() <= 64
                && id
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-');
            return if ok {
                Ok(Colour(text.to_owned()))
            } else {
                Err(BadColour(text.to_owned()))
            };
        }
        let digits = text.strip_prefix('#').unwrap_or("");
        if !digits.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(BadColour(text.to_owned()));
        }
        match digits.len() {
            6 => Ok(Colour(format!("#{}", digits.to_ascii_lowercase()))),
            3 => {
                let long: String = digits.chars().flat_map(|c| [c, c]).collect();
                Ok(Colour(format!("#{}", long.to_ascii_lowercase())))
            }
            _ => Err(BadColour(text.to_owned())),
        }
    }

    /// The id of the swatch this refers to, when it is a `swatch:<id>` reference.
    pub fn swatch_id(&self) -> Option<&str> {
        self.0.strip_prefix("swatch:").filter(|id| !id.is_empty())
    }

    /// The red, green and blue of the colour, each 0–255, or `None` if it is not a hex colour: a
    /// swatch reference has to be resolved against the book's swatches first.
    pub fn rgb(&self) -> Option<[u8; 3]> {
        if self.swatch_id().is_some() {
            return None;
        }
        let c = Colour::parse(&self.0).ok()?;
        let n = u32::from_str_radix(&c.0[1..], 16).ok()?;
        Some([(n >> 16) as u8, (n >> 8) as u8, n as u8])
    }
}

/// One colour along a gradient, `at` being how far along it is, 0 to 1.
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub struct GradientStop {
    pub at: f64,
    pub colour: Colour,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn swatch_references_parse_and_hex_does_not_read_them() {
        let c = Colour::parse("swatch:brand-red").unwrap();
        assert_eq!(c.swatch_id(), Some("brand-red"));
        assert_eq!(c.rgb(), None);
        for bad in ["swatch:", "swatch:Brand", "swatch:a b", "swatch:a/b"] {
            assert!(Colour::parse(bad).is_err(), "{bad}");
        }
        assert_eq!(Colour("#112233".into()).swatch_id(), None);
    }

    #[test]
    fn a_swatch_resolves_to_its_display_and_ink_and_an_unknown_one_to_grey() {
        let swatches = vec![BookSwatch {
            id: SwatchId("brand".into()),
            name: "Brand".into(),
            display: Colour("#c8102e".into()),
            ink: Some(Ink {
                c: 0,
                m: 100,
                y: 90,
                k: 10,
            }),
        }];
        let all = Swatches(&swatches);
        let paint = all.resolve(&Colour("swatch:brand".into()));
        assert_eq!(paint.rgb, [0xc8, 0x10, 0x2e]);
        assert_eq!(paint.ink.map(|i| i.m), Some(100));
        assert_eq!(all.resolve(&Colour("swatch:gone".into())).rgb, [128; 3]);
        let hex = all.resolve(&Colour("#102030".into()));
        assert_eq!((hex.rgb, hex.ink), ([0x10, 0x20, 0x30], None));
    }

    #[test]
    fn ink_shows_as_the_plain_inverse_and_says_how_much_ink_it_is() {
        assert_eq!(
            Ink {
                c: 0,
                m: 0,
                y: 0,
                k: 0
            }
            .to_rgb(),
            [255, 255, 255]
        );
        assert_eq!(
            Ink {
                c: 0,
                m: 0,
                y: 0,
                k: 100
            }
            .to_rgb(),
            [0, 0, 0]
        );
        assert_eq!(
            Ink {
                c: 100,
                m: 0,
                y: 0,
                k: 0
            }
            .to_rgb(),
            [0, 255, 255]
        );
        let rich = Ink {
            c: 60,
            m: 40,
            y: 40,
            k: 100,
        };
        assert_eq!(rich.total(), 240);
        assert!(rich.is_valid());
        assert!(!Ink {
            c: 101,
            m: 0,
            y: 0,
            k: 0
        }
        .is_valid());
    }

    #[test]
    fn parses_long_and_short_forms_to_lower_case() {
        assert_eq!(Colour::parse("#1F3A5F").unwrap().0, "#1f3a5f");
        assert_eq!(Colour::parse("#Fa0").unwrap().0, "#ffaa00");
    }

    #[test]
    fn refuses_what_is_not_a_colour() {
        for bad in [
            "", "#", "1f3a5f", "#12345", "#1234567", "#gggggg", "red", "#12 456",
        ] {
            assert!(Colour::parse(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn reads_the_channels() {
        assert_eq!(Colour("#1f3a5f".into()).rgb(), Some([0x1f, 0x3a, 0x5f]));
        assert_eq!(Colour("nope".into()).rgb(), None);
    }

    #[test]
    fn is_the_bare_string_on_the_wire() {
        assert_eq!(
            serde_json::to_string(&Colour("#000000".into())).unwrap(),
            "\"#000000\""
        );
    }
}
