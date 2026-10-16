// What a laid-out spread looks like to the UI: lines of positioned text per frame, and the checks
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use folio_model::{FrameId, SpreadId};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// A word, or a piece of one, drawn in one font. `x` is from the line's left edge.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
#[serde(rename_all = "camelCase")]
pub struct LayoutRun {
    pub text: String,
    pub x: f64,
    pub width: f64,
    /// The family actually used, which may differ from the one asked for.
    pub family: String,
    pub size: f64,
    pub weight: u16,
    pub italic: bool,
    /// Extra space after every glyph, in ems.
    pub tracking: f64,
    /// A hyphen drawn at the end of a line; it is not part of the text.
    pub hyphen: bool,
    /// The misspelt parts of `text`, each as where it starts and ends in UTF-16 code units.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub miss: Vec<(u32, u32)>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
#[serde(rename_all = "camelCase")]
pub struct LayoutLine {
    /// Left edge of the line and the top of its box, from the frame's top-left, in points.
    pub x: f64,
    pub top: f64,
    pub baseline: f64,
    pub width: f64,
    pub height: f64,
    pub runs: Vec<LayoutRun>,
}

/// A rectangle in points, from a frame's top-left.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub struct LayoutRect {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
#[serde(rename_all = "camelCase")]
pub struct FrameLayout {
    pub frame: FrameId,
    pub lines: Vec<LayoutLine>,
    /// The frame's text carries on past its last line, in a frame with no room.
    pub overset: bool,
    /// For a frame that comes from a master page: where its top-left sits on the spread, since the
    /// spread has no such frame to say. `frame` is then the master frame's id and the page's, joined
    /// by `@`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub origin: Option<Origin>,
    /// For a shape: the room its words have, from its top-left, where an editor goes. The lines
    /// are already placed inside it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_box: Option<LayoutRect>,
    /// For a shape: how its style sets the words, so an editor sets them the same way.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_align: Option<folio_model::Align>,
}

/// A point on a spread, in points from the top-left of the first page's trim.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub struct Origin {
    pub x: f64,
    pub y: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
#[serde(rename_all = "camelCase")]
pub enum PreflightKind {
    /// Text runs on past the last frame of its chain.
    OversetText,
    /// A text frame reaches outside the safe margin.
    TextOutsideMargin,
    /// A picture touches the trim edge but stops short of the bleed.
    PictureShortOfBleed,
    /// A preserved poem line was too long and wrapped.
    PoemLineTooLong,
    /// A picture frame or background names a picture that is not in the book.
    PictureMissing,
    /// A picture is drawn at under 300 pixels per inch: a warning.
    LowResolution,
    /// A picture is drawn at under 150 pixels per inch: an error.
    VeryLowResolution,
    /// A page has a background but the book has no bleed, so a trimmed edge may show paper.
    BleedNeeded,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
#[serde(rename_all = "camelCase")]
pub struct PreflightItem {
    pub kind: PreflightKind,
    pub spread: SpreadId,
    /// The frame at fault; none when it is a page's background.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub frame: Option<FrameId>,
    /// The page at fault, for a background.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub page: Option<folio_model::PageId>,
}

impl PreflightItem {
    /// A problem with one frame of `spread`.
    pub fn of_frame(kind: PreflightKind, spread: &SpreadId, frame: &FrameId) -> Self {
        Self {
            kind,
            spread: spread.clone(),
            frame: Some(frame.clone()),
            page: None,
        }
    }

    /// A problem with the background of one page of `spread`.
    pub fn of_page(kind: PreflightKind, spread: &SpreadId, page: &folio_model::PageId) -> Self {
        Self {
            kind,
            spread: spread.clone(),
            frame: None,
            page: Some(page.clone()),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
#[serde(rename_all = "camelCase")]
pub struct SpreadLayout {
    pub spread: SpreadId,
    pub frames: Vec<FrameLayout>,
    pub preflight: Vec<PreflightItem>,
}
