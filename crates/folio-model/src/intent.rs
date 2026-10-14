// Intents: the changes the UI asks the core to make
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::book::{Grid, PageNumbers};
use crate::frame::{Fit, Frame, StyleOverrides};
use crate::ids::{FrameId, PageId, SpreadId, StyleId};
use crate::units::{Margins, Pt, Rect, Size};

// ── Intents (UI → core) ──
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
#[serde(tag = "type")]
pub enum Intent {
    MoveFrame {
        spread: SpreadId,
        frame: FrameId,
        #[serde(rename = "box")]
        rect: Rect,
    },
    /// Puts a copy of a frame on the same spread, a little down and to the right, in front.
    DuplicateFrame {
        spread: SpreadId,
        frame: FrameId,
    },
    /// Locks a frame against moving and deleting, or lets it go again.
    LockFrame {
        spread: SpreadId,
        frame: FrameId,
        locked: bool,
    },
    /// Puts a picture in a picture frame (an asset path from the package) and says how it sits in
    /// the frame. Either part left out stays as it was.
    SetImage {
        frame: FrameId,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        asset: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        fit: Option<Fit>,
    },
    /// Puts page numbers on every page after the cover, or takes them off.
    SetPageNumbers {
        position: PageNumbers,
    },
    /// Hides a master page's frame on one page, or shows it again.
    HideMasterItem {
        spread: SpreadId,
        page: PageId,
        frame: FrameId,
        hide: bool,
    },
    SetText {
        frame: FrameId,
        text: String,
    },
    SetStyle {
        frame: FrameId,
        style: StyleId,
    },
    OverrideStyle {
        frame: FrameId,
        overrides: StyleOverrides,
    },
    AddFrame {
        spread: SpreadId,
        frame: Frame,
    },
    DeleteFrame {
        spread: SpreadId,
        frame: FrameId,
    },
    Reorder {
        spread: SpreadId,
        frame: FrameId,
        to: ReorderTo,
    },
    AddSpread {
        after: SpreadId,
        duplicate: bool,
    },
    DeleteSpread {
        spread: SpreadId,
    },
    MoveSpread {
        spread: SpreadId,
        to: u32,
    },
    SetBookGeometry {
        trim: Option<Size>,
        margins: Option<Margins>,
        bleed: Option<Pt>,
        grid: Option<Grid>,
    },
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
#[serde(rename_all = "lowercase")]
pub enum ReorderTo {
    Front,
    Back,
    Index(u32),
}
