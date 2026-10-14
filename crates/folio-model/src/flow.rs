// Text flows: ordered chains of frames that long text runs through
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::ids::{FlowId, FrameId};

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
#[serde(rename_all = "camelCase")]
pub struct Flow {
    pub id: FlowId,
    pub source: String,
    pub frames: Vec<FrameId>,
    #[serde(default = "yes")]
    pub auto_add_pages: bool,
}
fn yes() -> bool {
    true
}
