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
            Frame::Group { base, .. } => base,
        }
    }

    pub fn base(&self) -> &FrameBase {
        match self {
            Frame::Text(f) => &f.base,
            Frame::Image(f) => &f.base,
            Frame::Shape(f) => &f.base,
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
}
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
#[serde(rename_all = "lowercase")]
pub enum Shape {
    Rect,
    /// A rectangle with rounded corners (stored as `rounded` before D-24).
    #[serde(alias = "rounded")]
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
