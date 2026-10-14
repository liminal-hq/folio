// The availability status every native plugin reports to the app.
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// What a native plugin reports through its `get_status` command, so the UI can hide options that
/// do not work on the current system.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub struct PluginStatus {
    /// Whether the plugin's concern works on this system.
    pub available: bool,
    /// A short human-readable explanation when `available` is false.
    pub reason: Option<String>,
    /// Individually-detected capabilities, by stable identifier.
    pub features: Vec<String>,
}

impl PluginStatus {
    pub fn available(features: Vec<String>) -> Self {
        Self {
            available: true,
            reason: None,
            features,
        }
    }

    pub fn unavailable(reason: impl Into<String>) -> Self {
        Self {
            available: false,
            reason: Some(reason.into()),
            features: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_available_plugin_has_no_reason() {
        let status = PluginStatus::available(vec!["settings".into()]);
        assert!(status.available && status.reason.is_none());
    }

    #[test]
    fn an_unavailable_plugin_says_why() {
        let status = PluginStatus::unavailable("needs a portal");
        assert!(!status.available);
        assert_eq!(status.reason.as_deref(), Some("needs a portal"));
    }
}
