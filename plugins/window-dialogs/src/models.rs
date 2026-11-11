// Defines the serialisable models and event names of the window-dialogs plugin
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Sent to the opener when a dialog is done, with a `Finished`: its result, or a cancellation.
pub const FINISHED_EVENT: &str = "window-dialogs://finished";
/// Sent to the opener when it is blocked or released by a dialog, with a `ModalState`.
pub const MODAL_EVENT: &str = "window-dialogs://modal";
/// Sent to the opener when a dialog posts a message to it, with a `Posted`.
pub const MESSAGE_EVENT: &str = "window-dialogs://message";

/// How a dialog window is made, in logical pixels.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DialogSpec {
    pub title: String,
    pub width: f64,
    pub height: f64,
    #[serde(default)]
    pub min_width: Option<f64>,
    #[serde(default)]
    pub min_height: Option<f64>,
    #[serde(default)]
    pub resizable: bool,
    /// Blocks the opener while the dialog is open.
    #[serde(default = "yes")]
    pub modal: bool,
    /// Frameless, transparent and shadowed, for pages that draw their own title bar and frame.
    #[serde(default = "yes")]
    pub chromeless: bool,
    /// The page to load, relative to the app's front end; defaults to `index.html`.
    #[serde(default)]
    pub url: Option<String>,
}

fn yes() -> bool {
    true
}

/// What opening a dialog came to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Opened {
    pub request_id: u64,
    /// True when the dialog was already open, and was brought forward rather than made again.
    pub existing: bool,
}

/// What a dialog window is asked to do: the payload its opener sent.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DialogRequest {
    pub request_id: u64,
    pub label: String,
    pub payload: Value,
    pub modal: bool,
}

/// How a dialog ended. `result` is `None` when it was cancelled (its window was closed).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Finished {
    pub request_id: u64,
    pub label: String,
    pub cancelled: bool,
    pub result: Option<Value>,
}

/// Whether the opener is blocked, and whether the platform blocks it (`native`) or the page has to.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModalState {
    pub open: bool,
    pub label: String,
    pub native: bool,
}

/// A message a dialog posted to its opener while it stays open.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Posted {
    pub request_id: u64,
    pub label: String,
    pub message: Value,
}
