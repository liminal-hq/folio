// Frames: text, pictures, shapes and groups, and what they carry
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::colour::Colour;
use crate::ids::{FlowId, FrameId, MasterId, StyleId};
use crate::para::ParaLook;
use crate::units::{Pt, Rect};

#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
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
    /// The frames that are drawn: this one, or for a group the frames in it, however deep. A group
    /// is only a wrapper, its children being placed on the page like any other frame.
    pub fn leaves<'a>(&'a self, out: &mut Vec<&'a Frame>) {
        match self {
            Frame::Group { children, .. } => children.iter().for_each(|c| c.leaves(out)),
            other => out.push(other),
        }
    }

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

#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
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
    /// Decoration: a picture or the words of a frame that add nothing for a reader who cannot see
    /// the page (a pumpkin behind a page number, a page number itself). A tagged PDF marks it as an
    /// artifact, so a screen reader skips it, and preflight does not ask for a description of it.
    /// An empty description does not mean decorative; this does.
    #[serde(default, skip_serializing_if = "is_false")]
    #[ts(optional, as = "Option<bool>")]
    pub decorative: bool,
}

#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
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

#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub struct InlineContent {
    pub inline: Vec<Run>,
    /// The look of each paragraph (a paragraph begins at a line break), by place; paragraphs past
    /// the end of the list, and every one when there is no list, take their style's look.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub looks: Option<Vec<ParaLook>>,
}

impl InlineContent {
    /// Words with no paragraph looks of their own.
    pub fn of(inline: Vec<Run>) -> Self {
        Self {
            inline,
            looks: None,
        }
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, TS)]
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
    /// A line under these words.
    #[serde(default, skip_serializing_if = "is_false")]
    pub underline: bool,
    /// A line through these words.
    #[serde(default, skip_serializing_if = "is_false")]
    pub strike: bool,
    /// A size in points for these words alone, over the style's.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size: Option<f64>,
    /// Space added after every letter of these words, in thousandths of an em, over the style's.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tracking: Option<i32>,
    /// The language of these words alone (a BCP 47 tag: `fr-CA`), over the book's. It picks the
    /// dictionary spelling checks them against, how they are hyphenated and which quotes smart
    /// punctuation sets.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
    /// Where these words link to: a web address, `mailto:` or `tel:` address, or `#spread-3` for
    /// the third spread of the book. A link is drawn underlined in the link colour by the page, and
    /// never merges with the words beside it that link elsewhere or not at all.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub href: Option<String>,
}

fn is_false(value: &bool) -> bool {
    !*value
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub struct FlowSlot {
    pub id: FlowId,
    pub index: u32,
}

#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
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
    /// Where the words sit in the box when they do not fill it: at the top (a text box's way),
    /// in the middle (a shape's) or at the bottom.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub valign: Option<VAlign>,
}

/// Where words sit up and down in a box that has room to spare.
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
#[serde(rename_all = "lowercase")]
pub enum VAlign {
    Top,
    Middle,
    Bottom,
}

impl VAlign {
    /// How much of the room to spare goes above the words.
    pub fn share(self) -> f64 {
        match self {
            Self::Top => 0.0,
            Self::Middle => 0.5,
            Self::Bottom => 1.0,
        }
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
#[serde(rename_all = "lowercase")]
pub enum Align {
    Left,
    Center,
    Right,
    Justify,
}

#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
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
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub struct ImageRef {
    pub asset: Option<String>,
    pub fit: Fit,
    pub focus: (f64, f64),
    pub crop: Option<Rect>,
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
#[serde(rename_all = "lowercase")]
pub enum Fit {
    Fill,
    Fit,
    Stretch,
    Manual,
}

#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
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
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
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
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub struct TableCell {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<InlineContent>,
}

/// A table is one frame: a grid of cells whose words are laid out by `folio-layout` in the `cell`
/// style (D-24). Rows are equally tall; columns take `cols` shares of the width.
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
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

#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
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

/// The frames of a spread or master that are drawn, groups opened out into their children, in
/// stacking order (the last is in front).
pub fn leaves(frames: &[Frame]) -> Vec<&Frame> {
    let mut out = Vec::with_capacity(frames.len());
    frames.iter().for_each(|f| f.leaves(&mut out));
    out
}

impl Frame {
    /// Moves a frame and everything in it by `dx` and `dy`.
    pub fn translate(&mut self, dx: f64, dy: f64) {
        let base = self.base_mut();
        base.rect.x = Pt(base.rect.x.0 + dx);
        base.rect.y = Pt(base.rect.y.0 + dy);
        if let Frame::Group { children, .. } = self {
            children.iter_mut().for_each(|c| c.translate(dx, dy));
        }
    }

    /// Gives a frame a new box. A group's children keep their places in it: they move with it and
    /// are stretched with it, each box scaled by how much the group's box grew or shrank.
    pub fn set_rect(&mut self, rect: Rect) {
        let old = self.base().rect;
        self.base_mut().rect = rect;
        if let Frame::Group { children, .. } = self {
            let sx = if old.w.0 > 0.0 {
                rect.w.0 / old.w.0
            } else {
                1.0
            };
            let sy = if old.h.0 > 0.0 {
                rect.h.0 / old.h.0
            } else {
                1.0
            };
            for child in children {
                let at = child.base().rect;
                child.set_rect(Rect {
                    x: Pt(rect.x.0 + (at.x.0 - old.x.0) * sx),
                    y: Pt(rect.y.0 + (at.y.0 - old.y.0) * sy),
                    w: Pt(at.w.0 * sx),
                    h: Pt(at.h.0 * sy),
                });
            }
        }
    }
}
