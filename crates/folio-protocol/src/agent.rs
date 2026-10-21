// What the app says about agents that can connect to it
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// An agent program Folio can be added to.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub struct AgentProgram {
    /// What `agent_install` takes: `claude-code`, `cursor` and so on.
    pub id: String,
    pub label: String,
}

/// Whether agents can reach the open books, and what is needed to add Folio to an agent program.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub struct AgentStatus {
    /// The app is listening, so an agent works on the books open in its windows.
    pub listening: bool,
    /// Where it listens: a socket or pipe.
    pub endpoint: String,
    /// The `folio-mcp` program that goes with the app; `None` when it is not beside it, which
    /// leaves nothing to add to an agent program.
    pub binary: Option<String>,
    pub programs: Vec<AgentProgram>,
}
