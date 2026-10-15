// The book's styles: what `styles.json` says, resolved into what the layout needs
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::collections::BTreeMap;

use folio_model::{Align as ModelAlign, StyleId, StyleOverrides};
use serde::Deserialize;
use serde_json::Value;

use crate::fonts::{FALLBACK_SANS, FALLBACK_SERIF};
use crate::paragraph::{Align, TextStyle, Wrap};

/// One named style as the file stores it. Every key is optional.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct StyleDef {
    pub name: String,
    pub size_pt: Option<f64>,
    pub weight: Option<u16>,
    pub italic: bool,
    /// Drawn in the sans family rather than the look's serif.
    pub sans: bool,
    pub align: Option<String>,
    pub line_height: Option<f64>,
    /// Letter spacing in ems.
    pub tracking: Option<f64>,
    /// First-line indent in ems.
    pub first_indent: Option<f64>,
    pub space_before: Option<f64>,
    pub space_after: Option<f64>,
    /// `"preserve"` keeps every line break (poetry).
    pub wrap: Option<String>,
    pub hyphenate: Option<bool>,
}

/// A style ready to lay text out with.
#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedStyle {
    pub text: TextStyle,
    pub align: Align,
    pub first_indent: f64,
    pub space_before: f64,
    pub space_after: f64,
    pub wrap: Wrap,
    pub hyphenate: bool,
}

/// The sizes used when a style is missing or leaves one out.
const DEFAULT_SIZE: f64 = 12.0;
const DEFAULT_LINE_HEIGHT: f64 = 1.3;

#[derive(Debug, Clone)]
pub struct StyleSheet {
    family: String,
    styles: BTreeMap<String, StyleDef>,
}

impl Default for StyleSheet {
    fn default() -> Self {
        Self {
            family: FALLBACK_SERIF.to_string(),
            styles: BTreeMap::new(),
        }
    }
}

impl StyleSheet {
    /// Reads the shape of `styles.json`: `{ "fontFamily": "...", "styles": { id: {...} } }`.
    /// Anything unreadable is left out, and the layout uses defaults for it.
    pub fn from_value(value: &Value) -> Self {
        let family = value
            .get("fontFamily")
            .and_then(Value::as_str)
            .unwrap_or(FALLBACK_SERIF)
            .to_string();
        let styles = value
            .get("styles")
            .and_then(Value::as_object)
            .map(|map| {
                map.iter()
                    .filter_map(|(id, v)| {
                        serde_json::from_value::<StyleDef>(v.clone())
                            .ok()
                            .map(|d| (id.clone(), d))
                    })
                    .collect()
            })
            .unwrap_or_default();
        Self { family, styles }
    }

    /// This sheet with the styles of `fallback` it does not have: a book that never used a style
    /// (a bubble's, say) still gets the one Folio ships, and a book that defines its own keeps it.
    pub fn with_fallback(mut self, fallback: &StyleSheet) -> Self {
        for (id, def) in &fallback.styles {
            self.styles.entry(id.clone()).or_insert_with(|| def.clone());
        }
        self
    }

    pub fn contains(&self, id: &StyleId) -> bool {
        self.styles.contains_key(&id.0)
    }

    /// The style `id` with `overrides` on top. A missing style resolves to the defaults.
    pub fn resolve(&self, id: &StyleId, overrides: Option<&StyleOverrides>) -> ResolvedStyle {
        let def = self.styles.get(&id.0).cloned().unwrap_or_default();
        let size = def.size_pt.unwrap_or(DEFAULT_SIZE);
        let mut text = TextStyle::new(
            if def.sans {
                FALLBACK_SANS
            } else {
                &self.family
            },
            size,
        );
        text.weight = def.weight.unwrap_or(400);
        text.italic = def.italic;
        text.tracking_em = def.tracking.unwrap_or(0.0);
        text.line_height = def.line_height.unwrap_or(DEFAULT_LINE_HEIGHT);
        let mut align = match def.align.as_deref() {
            Some("center") => Align::Center,
            Some("right") => Align::Right,
            Some("justify") => Align::Justify,
            _ => Align::Left,
        };
        let preserve = def.wrap.as_deref() == Some("preserve");
        if let Some(o) = overrides {
            if let Some(size) = o.size {
                text.size_pt = size.0;
            }
            if let Some(a) = o.align {
                align = match a {
                    ModelAlign::Left => Align::Left,
                    ModelAlign::Center => Align::Center,
                    ModelAlign::Right => Align::Right,
                    ModelAlign::Justify => Align::Justify,
                };
            }
            if let Some(bold) = o.bold {
                text.weight = if bold { 700 } else { 400 };
            }
            if let Some(italic) = o.italic {
                text.italic = italic;
            }
        }
        ResolvedStyle {
            first_indent: def.first_indent.unwrap_or(0.0) * text.size_pt,
            space_before: def.space_before.unwrap_or(0.0),
            space_after: def.space_after.unwrap_or(0.0),
            wrap: if preserve {
                Wrap::Preserve
            } else {
                Wrap::Reflow
            },
            hyphenate: def.hyphenate.unwrap_or(!preserve),
            align,
            text,
        }
    }
}
