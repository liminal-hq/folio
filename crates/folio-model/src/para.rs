// A paragraph's own look: alignment, spacing, line spacing and indents that win over its style's
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::frame::Align;

/// The most space, indent or first-line indent a paragraph may be given, in points.
pub const PARA_MAX_LENGTH: f64 = 720.0;
/// The least and most a line may be spaced, as a multiple of the type size.
pub const PARA_LINE_HEIGHT: (f64, f64) = (0.5, 5.0);

/// What one paragraph sets for itself over its style. A field left out takes the style's value.
///
/// Lengths are in points, and line spacing is a multiple of the type size. Space before and space
/// after add together; the space before is left off at the top of a column.
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
#[serde(rename_all = "camelCase", default)]
pub struct ParaLook {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub align: Option<Align>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub space_before: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub space_after: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub line_height: Option<f64>,
    /// How far the first line is moved in, over the style's.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub first_indent: Option<f64>,
    /// How far every line is moved in from the left.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub indent: Option<f64>,
    /// How far every line is moved in from the right.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub indent_right: Option<f64>,
}

/// A field of a [`ParaLook`], named so a change can take its value away.
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
#[serde(rename_all = "camelCase")]
pub enum ParaField {
    Align,
    SpaceBefore,
    SpaceAfter,
    LineHeight,
    FirstIndent,
    Indent,
    IndentRight,
}

/// A change to the look of some paragraphs: the fields of `clear` go back to their style's, then
/// those `set` has are set.
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
#[serde(rename_all = "camelCase", default)]
pub struct ParaChange {
    pub set: ParaLook,
    pub clear: Vec<ParaField>,
}

fn length_ok(v: Option<f64>) -> bool {
    v.is_none_or(|v| v.is_finite() && (0.0..=PARA_MAX_LENGTH).contains(&v))
}

impl ParaLook {
    /// Whether the paragraph sets nothing for itself.
    pub fn is_empty(&self) -> bool {
        *self == ParaLook::default()
    }

    /// Why this look cannot be set, if it cannot: lengths are from nothing up to
    /// [`PARA_MAX_LENGTH`] points and line spacing is within [`PARA_LINE_HEIGHT`].
    pub fn check(&self) -> Result<(), String> {
        let lengths = [
            self.space_before,
            self.space_after,
            self.first_indent,
            self.indent,
            self.indent_right,
        ];
        if !lengths.into_iter().all(length_ok) {
            return Err(format!(
                "spaces and indents are from 0 to {PARA_MAX_LENGTH} points"
            ));
        }
        let (low, high) = PARA_LINE_HEIGHT;
        if self
            .line_height
            .is_some_and(|h| !h.is_finite() || h < low || h > high)
        {
            return Err(format!(
                "line spacing is from {low} to {high} times the size"
            ));
        }
        Ok(())
    }

    /// Makes `change`.
    pub fn apply(&mut self, change: &ParaChange) {
        for field in &change.clear {
            match field {
                ParaField::Align => self.align = None,
                ParaField::SpaceBefore => self.space_before = None,
                ParaField::SpaceAfter => self.space_after = None,
                ParaField::LineHeight => self.line_height = None,
                ParaField::FirstIndent => self.first_indent = None,
                ParaField::Indent => self.indent = None,
                ParaField::IndentRight => self.indent_right = None,
            }
        }
        let set = &change.set;
        self.align = set.align.or(self.align);
        self.space_before = set.space_before.or(self.space_before);
        self.space_after = set.space_after.or(self.space_after);
        self.line_height = set.line_height.or(self.line_height);
        self.first_indent = set.first_indent.or(self.first_indent);
        self.indent = set.indent.or(self.indent);
        self.indent_right = set.indent_right.or(self.indent_right);
    }
}

impl ParaChange {
    /// Whether this changes nothing.
    pub fn is_empty(&self) -> bool {
        self.set.is_empty() && self.clear.is_empty()
    }

    /// Why this change cannot be made, if it cannot.
    pub fn check(&self) -> Result<(), String> {
        self.set.check()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_change_sets_fields_and_clears_others() {
        let mut look = ParaLook {
            space_after: Some(8.0),
            indent: Some(18.0),
            ..ParaLook::default()
        };
        look.apply(&ParaChange {
            set: ParaLook {
                space_before: Some(4.0),
                ..ParaLook::default()
            },
            clear: vec![ParaField::Indent],
        });
        assert_eq!(look.space_before, Some(4.0));
        assert_eq!(look.space_after, Some(8.0));
        assert_eq!(look.indent, None);
    }

    #[test]
    fn lengths_outside_the_range_are_refused() {
        let bad = |look: ParaLook| look.check().is_err();
        assert!(bad(ParaLook {
            space_after: Some(-1.0),
            ..ParaLook::default()
        }));
        assert!(bad(ParaLook {
            indent: Some(f64::NAN),
            ..ParaLook::default()
        }));
        assert!(bad(ParaLook {
            line_height: Some(9.0),
            ..ParaLook::default()
        }));
        assert!(ParaLook {
            line_height: Some(1.5),
            space_before: Some(12.0),
            ..ParaLook::default()
        }
        .check()
        .is_ok());
    }

    #[test]
    fn an_empty_look_is_written_as_nothing_and_read_from_nothing() {
        assert_eq!(serde_json::to_string(&ParaLook::default()).unwrap(), "{}");
        let look: ParaLook = serde_json::from_str(r#"{"spaceAfter":6}"#).unwrap();
        assert_eq!(look.space_after, Some(6.0));
    }
}
