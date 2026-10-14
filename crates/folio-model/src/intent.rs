// Intents: the changes the UI asks the core to make
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::book::Grid;
use crate::frame::{Frame, StyleOverrides};
use crate::ids::{FrameId, SpreadId, StyleId};
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
