// Frames: text, pictures, shapes and groups, and what they carry
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::colour::Colour;
use crate::ids::{FlowId, FrameId, MasterId, StyleId};
use crate::units::{Pt, Rect};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Frame {
    Text(TextFrame),
    Image(ImageFrame),
    Shape(ShapeFrame),
    Table(TableFrame),
    Group {
        #[serde(flatten)]
        base: FrameBase,
        children: Vec<Frame>,
    },
}
impl Frame {
    pub fn base_mut(&mut self) -> &mut FrameBase {
        match self {
            Frame::Text(f) => &mut f.base,
            Frame::Image(f) => &mut f.base,
            Frame::Shape(f) => &mut f.base,
            Frame::Table(f) => &mut f.base,
            Frame::Group { base, .. } => base,
        }
    }

    pub fn base(&self) -> &FrameBase {
        match self {
            Frame::Text(f) => &f.base,
            Frame::Image(f) => &f.base,
            Frame::Shape(f) => &f.base,
            Frame::Table(f) => &f.base,
            Frame::Group { base, .. } => base,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
#[serde(rename_all = "camelCase")]
pub struct FrameBase {
    pub id: FrameId,
    #[serde(rename = "box")]
    pub rect: Rect,
    #[serde(default)]
    pub rotation: f64,
    #[serde(default)]
    pub locked: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from_master: Option<MasterId>,
    /// Alt text for pictures; accessible name for others.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
#[serde(rename_all = "camelCase")]
pub struct TextFrame {
    #[serde(flatten)]
    pub base: FrameBase,
    pub style: StyleId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<InlineContent>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub flow: Option<FlowSlot>,
    #[serde(default = "one")]
    pub columns: u8,
    #[serde(default)]
    pub baseline_lock: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub overrides: Option<StyleOverrides>,
}
fn one() -> u8 {
    1
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub struct InlineContent {
    pub inline: Vec<Run>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub struct Run {
    pub text: String,
    #[serde(default)]
    pub style: Option<StyleId>,
    #[serde(default)]
    pub bold: bool,
    #[serde(default)]
    pub italic: bool,
    /// A typeface for these words alone, over the style's.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub font: Option<String>,
    /// A colour for these words alone, over the style's.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub colour: Option<Colour>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub struct FlowSlot {
    pub id: FlowId,
    pub index: u32,
}

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub struct StyleOverrides {
    pub size: Option<Pt>,
    pub align: Option<Align>,
    pub bold: Option<bool>,
    pub italic: Option<bool>,
    /// A typeface for the whole box, over its style's.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub font: Option<String>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
#[serde(rename_all = "lowercase")]
pub enum Align {
    Left,
    Center,
    Right,
    Justify,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
#[serde(rename_all = "camelCase")]
pub struct ImageFrame {
    #[serde(flatten)]
    pub base: FrameBase,
    pub image: ImageRef,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub placeholder_label: Option<String>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub struct ImageRef {
    pub asset: Option<String>,
    pub fit: Fit,
    pub focus: (f64, f64),
    pub crop: Option<Rect>,
}
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
#[serde(rename_all = "lowercase")]
pub enum Fit {
    Fill,
    Fit,
    Stretch,
    Manual,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
#[serde(rename_all = "camelCase")]
pub struct ShapeFrame {
    #[serde(flatten)]
    pub base: FrameBase,
    pub shape: Shape,
    pub fill: Option<Colour>,
    pub style: Option<StyleId>,
    /// The outline colour. `None` draws no outline.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stroke: Option<Colour>,
    /// The outline's width in points; `None` is the default hairline. The drawn width does not scale
    /// with the frame.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stroke_width: Option<Pt>,
    /// How round the corners of a `Round` shape are, in points; `None` is the default.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub radius: Option<Pt>,
    /// A line runs from the top-left of its box to the bottom-right, or from the bottom-left to the
    /// top-right when this is set.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub flip: bool,
    /// The words in the shape, if it has any. They are laid out inside the shape's body (see
    /// [`Shape::text_inset`]) in `style`, or the kind's own style when there is none.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<InlineContent>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub overrides: Option<StyleOverrides>,
}
/// The most rows or columns a table can have.
pub const TABLE_MAX_SIDE: usize = 50;

/// How a table's rules are drawn.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
#[serde(rename_all = "lowercase")]
pub enum TableLook {
    /// A rule under every row.
    #[default]
    Lined,
    /// A rule around every cell.
    Grid,
    /// A rule under the header and a tint on every other row.
    Recipe,
}

/// One cell of a table: its words, if it has any.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub struct TableCell {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<InlineContent>,
}

/// A table is one frame: a grid of cells whose words are laid out by `folio-layout` in the `cell`
/// style (D-24). Rows are equally tall; columns take `cols` shares of the width.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
#[serde(rename_all = "camelCase")]
pub struct TableFrame {
    #[serde(flatten)]
    pub base: FrameBase,
    /// The width of each column as a share of the table's width; they add up to 1.
    pub cols: Vec<f64>,
    /// The rows, each with one cell per column.
    pub rows: Vec<Vec<TableCell>>,
    /// How many rows from the top are headers, set bold.
    #[serde(default)]
    pub head: u8,
    #[serde(default)]
    pub look: TableLook,
    /// The style the cells are set in; `None` is the kind's `cell`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub style: Option<StyleId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub overrides: Option<StyleOverrides>,
}

impl TableFrame {
    /// Whether the table is one a book can hold: between one and `TABLE_MAX_SIDE` rows and columns,
    /// every row as long as there are columns, column shares that are positive and add up to 1, and
    /// no more header rows than rows.
    pub fn check(&self) -> Result<(), String> {
        let (r, c) = (self.rows.len(), self.cols.len());
        if r == 0 || c == 0 || r > TABLE_MAX_SIDE || c > TABLE_MAX_SIDE {
            return Err(format!(
                "a table has 1 to {TABLE_MAX_SIDE} rows and columns"
            ));
        }
        if self.rows.iter().any(|row| row.len() != c) {
            return Err("every row of a table has one cell for each column".into());
        }
        if self.cols.iter().any(|w| !w.is_finite() || *w <= 0.0)
            || (self.cols.iter().sum::<f64>() - 1.0).abs() > 1e-6
        {
            return Err("column widths are positive and add up to 1".into());
        }
        if usize::from(self.head) > r {
            return Err("a table cannot have more header rows than rows".into());
        }
        Ok(())
    }

    /// A table of `rows` by `columns` empty cells in equal columns.
    pub fn blank(base: FrameBase, rows: usize, columns: usize) -> TableFrame {
        TableFrame {
            base,
            cols: vec![1.0 / columns as f64; columns],
            rows: vec![vec![TableCell::default(); columns]; rows],
            head: 0,
            look: TableLook::Lined,
            style: None,
            overrides: None,
        }
    }
}

/// How far in from each side of a shape's box its words start, as a share of the box (D-24).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TextInset {
    pub left: f64,
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
}

impl Shape {
    /// Whether words can go in the shape: a line has no inside.
    pub fn holds_text(self) -> bool {
        self != Shape::Line
    }

    /// Where words sit inside the kind, so a bubble's text stays in its body and out of its tail.
    pub fn text_inset(self) -> TextInset {
        let (left, top, right, bottom) = match self {
            Shape::Rect | Shape::Line => (0.0, 0.0, 0.0, 0.0),
            Shape::Round => (0.04, 0.04, 0.04, 0.04),
            Shape::Ellipse => (0.15, 0.15, 0.15, 0.15),
            Shape::Speech => (0.08, 0.08, 0.08, 0.38),
            Shape::Thought => (0.18, 0.14, 0.18, 0.4),
            Shape::Shout => (0.2, 0.2, 0.2, 0.2),
        };
        TextInset {
            left,
            top,
            right,
            bottom,
        }
    }

    /// The style words in the kind use when the shape names none: bubbles speak in `bubble`, and
    /// every other shape (the note box) in `callout`.
    pub fn default_style(self) -> &'static str {
        match self {
            Shape::Speech | Shape::Thought | Shape::Shout => "bubble",
            _ => "callout",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
#[serde(rename_all = "lowercase")]
pub enum Shape {
    Rect,
    /// A rectangle with rounded corners.
    Round,
    Ellipse,
    /// A straight line across the box's diagonal. Its box may have no height or no width.
    Line,
    /// A speech bubble with a tail at the bottom left.
    Speech,
    /// A thought bubble: a cloud with two small circles trailing off it.
    Thought,
    /// A jagged shout bubble.
    Shout,
}
