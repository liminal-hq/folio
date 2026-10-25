// The book's styles: what `styles.json` says, resolved into what the layout needs
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::collections::BTreeMap;

use folio_model::{Align as ModelAlign, StyleDef, StyleId, StyleOverrides, Styles};
use serde_json::Value;

use crate::fonts::{FALLBACK_SANS, FALLBACK_SERIF};
use crate::paragraph::{Align, TextStyle, Wrap};

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
    /// The sheet a book's styles make: its typeface for body text and its named styles.
    pub fn from_styles(styles: &Styles) -> Self {
        Self {
            family: if styles.font_family.is_empty() {
                FALLBACK_SERIF.to_string()
            } else {
                styles.font_family.clone()
            },
            styles: styles.styles.clone(),
        }
    }

    /// Reads the shape of `styles.json`: `{ "fontFamily": "...", "styles": { id: {...} } }`.
    /// Anything unreadable gives an empty sheet, and the layout uses defaults for it.
    pub fn from_value(value: &Value) -> Self {
        Self::from_styles(&serde_json::from_value(value.clone()).unwrap_or_default())
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
        let family = match (&def.family, def.sans) {
            (Some(own), _) => own.as_str(),
            (None, true) => FALLBACK_SANS,
            (None, false) => &self.family,
        };
        let mut text = TextStyle::new(family, size);
        text.weight = def.weight.unwrap_or(400);
        text.colour = def.colour.clone();
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
            if let Some(font) = &o.font {
                text.family = font.clone();
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn sheet() -> StyleSheet {
        StyleSheet::from_value(&json!({
            "fontFamily": "Literata",
            "styles": {
                "body": { "sizePt": 11 },
                "label": { "sans": true },
                "mono": { "sans": true, "family": "Source Code Pro" }
            }
        }))
    }

    #[test]
    fn a_style_is_drawn_in_the_look_unless_it_is_sans_or_names_its_own_typeface() {
        let sheet = sheet();
        let family = |id: &str| sheet.resolve(&StyleId(id.into()), None).text.family;
        assert_eq!(family("body"), "Literata");
        assert_eq!(family("label"), FALLBACK_SANS);
        assert_eq!(family("mono"), "Source Code Pro");
    }

    #[test]
    fn a_sheet_without_a_typeface_uses_the_default_serif() {
        let sheet = StyleSheet::from_value(&json!({ "styles": {} }));
        assert_eq!(
            sheet.resolve(&StyleId("body".into()), None).text.family,
            FALLBACK_SERIF
        );
    }
}
