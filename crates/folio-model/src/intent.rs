// Intents: the changes the UI asks the core to make
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::book::{Grid, PageNumbers};
use crate::frame::{Fit, Frame, StyleOverrides};
use crate::ids::{FrameId, MasterId, PageId, SpreadId, StyleId};
use crate::spread::Axis;
use crate::units::{Margins, Pt, Rect, Size};

/// A frame and the box it is to have.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub struct FrameMove {
    pub frame: FrameId,
    #[serde(rename = "box")]
    pub rect: Rect,
}

/// Where a frame or a guide lives: on a spread, or on a master page that every page using it shows.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
#[serde(rename_all = "camelCase")]
pub enum Surface {
    Spread(SpreadId),
    Master(MasterId),
}

// ── Intents (UI → core) ──
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
#[serde(tag = "type")]
pub enum Intent {
    MoveFrame {
        on: Surface,
        frame: FrameId,
        #[serde(rename = "box")]
        rect: Rect,
    },
    /// Moves or resizes several frames together, as one change that merges with the next of the
    /// same frames within the coalescing window, so a drag or a nudge of a selection is one undo step.
    MoveFrames {
        on: Surface,
        moves: Vec<FrameMove>,
    },
    /// Puts a copy of a frame on the same spread, a little down and to the right, in front.
    DuplicateFrame {
        on: Surface,
        frame: FrameId,
    },
    /// Locks a frame against moving and deleting, or lets it go again.
    LockFrame {
        on: Surface,
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
    /// Makes master page A (and B when the page numbers are outside) if the book has none, and gives
    /// every page after the cover without a master the one its side uses, so a master can be edited.
    EnsureMasters,
    /// Hides a master page's frame on one page, or shows it again.
    HideMasterItem {
        spread: SpreadId,
        page: PageId,
        frame: FrameId,
        hide: bool,
    },
    /// Puts a ruler guide on a spread, or on a master page so it shows on every page using it.
    AddGuide {
        on: Surface,
        axis: Axis,
        pos: Pt,
    },
    /// Moves a ruler guide, named by its place in the spread's list. A locked guide stays put.
    MoveGuide {
        on: Surface,
        index: usize,
        pos: Pt,
    },
    /// Locks a ruler guide where it is, or frees it.
    LockGuide {
        on: Surface,
        index: usize,
        locked: bool,
    },
    /// Takes a ruler guide off its spread. A locked guide stays.
    RemoveGuide {
        on: Surface,
        index: usize,
    },
    /// Moves a ruler guide from a spread onto the masters its pages use, so it shows on every page
    /// using them. A horizontal guide goes on each of them; a vertical one on the master of the page
    /// it is over (as a position on that page), and mirrored onto the facing master when there is one.
    PromoteGuide {
        spread: SpreadId,
        index: usize,
    },
    /// Moves a guide from a master back onto one spread that uses it, so it shows only there.
    DemoteGuide {
        master: MasterId,
        index: usize,
        spread: SpreadId,
    },
    /// Takes the unlocked guides off one spread, or off every spread when none is named.
    ClearGuides {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        spread: Option<SpreadId>,
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
        on: Surface,
        frame: Frame,
    },
    DeleteFrame {
        on: Surface,
        frame: FrameId,
    },
    Reorder {
        on: Surface,
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
