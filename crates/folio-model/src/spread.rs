// Spreads, their pages and guides
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::background::Background;
use crate::frame::Frame;
use crate::ids::{FrameId, MasterId, PageId, SpreadId};
use crate::units::Pt;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
#[serde(rename_all = "lowercase")]
pub enum SpreadKind {
    Cover,
    Single,
    Spread,
    Opener,
    Closer,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
#[serde(rename_all = "camelCase")]
pub struct Spread {
    pub id: SpreadId,
    pub kind: SpreadKind,
    pub pages: Vec<Page>,
    /// z-order = vector order
    pub frames: Vec<Frame>,
    #[serde(default)]
    pub guides: Vec<Guide>,
    #[serde(default)]
    pub master_overrides: Vec<MasterOverride>,
    /// A background across both pages, through the gutter. A page's own background wins over it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bg_span: Option<Background>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub struct Page {
    pub id: PageId,
    pub master: Option<MasterId>,
    /// This page's own background, which wins over the spread's and the master's.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bg: Option<Background>,
}
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub struct Guide {
    pub axis: Axis,
    pub pos: Pt,
    pub locked: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
#[serde(rename_all = "lowercase")]
pub enum Axis {
    X,
    Y,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
#[serde(rename_all = "camelCase")]
pub struct MasterOverride {
    pub page_id: PageId,
    pub hide: Vec<FrameId>,
}
