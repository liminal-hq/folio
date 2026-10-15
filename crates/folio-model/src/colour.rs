// A colour as the hex string it is stored as, and the check an intent applies to it
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// A colour: `#rrggbb`, lower case. It is stored as the string so a book stays readable, and so a
/// hand-edited or older book still opens: reading never fails on a colour. An intent checks it with
/// [`Colour::parse`] before it stores one.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/protocol/src/generated/",
    type = "string"
)]
#[serde(transparent)]
pub struct Colour(pub String);

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
    /// A colour from `#rgb` or `#rrggbb` (either case), stored as `#rrggbb` in lower case.
    pub fn parse(text: &str) -> Result<Colour, BadColour> {
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

    /// The red, green and blue of the colour, each 0–255, or `None` if it is not a colour.
    pub fn rgb(&self) -> Option<[u8; 3]> {
        let c = Colour::parse(&self.0).ok()?;
        let n = u32::from_str_radix(&c.0[1..], 16).ok()?;
        Some([(n >> 16) as u8, (n >> 8) as u8, n as u8])
    }
}

/// One colour along a gradient, `at` being how far along it is, 0 to 1.
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
