// The book's styles: the named paragraph styles and the typeface they start in
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use ts_rs::TS;

use crate::colour::{BookSwatch, Colour};

/// One named style as `styles.json` stores it. Every key is optional.
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", default)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub struct StyleDef {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_pt: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weight: Option<u16>,
    #[serde(skip_serializing_if = "is_false")]
    pub italic: bool,
    /// Drawn in the sans family rather than the look's serif.
    #[serde(skip_serializing_if = "is_false")]
    pub sans: bool,
    /// This style's own typeface, which wins over the look and `sans`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub family: Option<String>,
    /// The colour the words are drawn in; none is the ink the page uses.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub colour: Option<Colour>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub align: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_height: Option<f64>,
    /// Letter spacing in ems.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tracking: Option<f64>,
    /// First-line indent in ems.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_indent: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub space_before: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub space_after: Option<f64>,
    /// `"preserve"` keeps every line break (poetry).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wrap: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hyphenate: Option<bool>,
    /// Where a paragraph in this style begins in a flowing story: `"page"` or `"recto"` (a
    /// right-hand page). A chapter title says `"recto"`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start: Option<String>,
    /// Keys this version does not know, kept as they were so a save does not lose them.
    #[serde(flatten, skip_serializing_if = "BTreeMap::is_empty")]
    #[ts(skip)]
    pub extra: BTreeMap<String, Value>,
}

fn is_false(value: &bool) -> bool {
    !*value
}

/// The book's styles, with the look they started from. It is `styles.json` in the package.
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", default)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub struct Styles {
    /// The look the book started in (`friendly`, `modern`, `classic`).
    pub look: String,
    /// The typeface body text is drawn in.
    pub font_family: String,
    pub styles: BTreeMap<String, StyleDef>,
    /// The colours the book names, which fields can refer to as `swatch:<id>` (D-40).
    #[serde(
        default,
        skip_serializing_if = "Vec::is_empty",
        deserialize_with = "kept_swatches"
    )]
    pub swatches: Vec<BookSwatch>,
    /// Keys this version does not know (colour swatches, character styles), kept as they were.
    #[serde(flatten, skip_serializing_if = "BTreeMap::is_empty")]
    #[ts(skip)]
    pub extra: BTreeMap<String, Value>,
}

/// Reads the swatches that make sense and leaves out any that do not (one written by something else,
/// or by an older version), so a styles file never fails to open for one.
fn kept_swatches<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Vec<BookSwatch>, D::Error> {
    let raw: Vec<Value> = Vec::deserialize(d)?;
    Ok(raw
        .into_iter()
        .filter_map(|v| serde_json::from_value::<BookSwatch>(v).ok())
        .collect())
}

impl Styles {
    /// Whether there is nothing to keep: no look, no typeface, no styles and no unknown keys.
    pub fn is_empty(&self) -> bool {
        self.look.is_empty()
            && self.font_family.is_empty()
            && self.styles.is_empty()
            && self.swatches.is_empty()
            && self.extra.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn reads_what_styles_json_holds_and_writes_it_back_as_it_was() {
        let file = json!({
            "look": "classic",
            "fontFamily": "Literata",
            "swatches": [{ "id": "ink", "name": "Ink", "display": "#1c1b20", "ink": { "c": 0, "m": 0, "y": 0, "k": 100 } }],
            "styles": {
                "body": { "name": "Body", "sizePt": 10.5, "firstIndent": 1.2, "future": true },
                "h1": { "name": "Heading", "sizePt": 24.0, "weight": 700, "sans": true }
            }
        });
        let styles: Styles = serde_json::from_value(file.clone()).unwrap();
        assert_eq!(styles.font_family, "Literata");
        assert_eq!(styles.styles["body"].size_pt, Some(10.5));
        assert_eq!(styles.styles["body"].extra["future"], json!(true));
        assert_eq!(serde_json::to_value(&styles).unwrap(), file);
    }

    #[test]
    fn a_swatch_that_does_not_make_sense_is_left_out_and_the_file_still_opens() {
        let styles: Styles = serde_json::from_value(json!({
            "swatches": [
                { "name": "Old", "rgb": "#123456" },
                { "id": "ok", "name": "Fine", "display": "#abcdef" }
            ]
        }))
        .unwrap();
        assert_eq!(styles.swatches.len(), 1);
        assert_eq!(styles.swatches[0].id.0, "ok");
    }

    #[test]
    fn a_missing_key_is_left_out_not_written_as_null() {
        let styles: Styles =
            serde_json::from_value(json!({ "styles": { "body": { "name": "Body" } } })).unwrap();
        let written = serde_json::to_value(&styles).unwrap();
        assert_eq!(written["styles"]["body"], json!({ "name": "Body" }));
    }

    #[test]
    fn a_book_without_styles_has_none_to_write() {
        assert!(Styles::default().is_empty());
        assert!(!Styles {
            look: "friendly".into(),
            ..Styles::default()
        }
        .is_empty());
    }
}
