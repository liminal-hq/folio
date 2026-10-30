// What the app says about looking for, fetching and installing a newer Folio
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// How this copy of Folio was installed, which decides how it is updated.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub enum InstallKind {
    /// An AppImage: the file is replaced.
    AppImage,
    /// A Debian package: installed through the system's privilege prompt.
    Deb,
    /// An RPM package: installed through the system's privilege prompt.
    Rpm,
    /// A Windows installer, which closes Folio to run.
    Windows,
}

/// Where an update check has got to. Sent as the `update-state` event and by `update_state`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "camelCase")]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub enum UpdateState {
    /// Nothing has been asked yet.
    Idle,
    Checking,
    /// This is the newest version.
    UpToDate,
    /// A newer version exists.
    #[serde(rename_all = "camelCase")]
    Available {
        version: String,
        /// What the release says changed, as plain text.
        notes: Option<String>,
        /// When it was published, as an RFC 3339 date.
        date: Option<String>,
        install: InstallKind,
    },
    #[serde(rename_all = "camelCase")]
    Downloading {
        /// Bytes fetched so far. A download is far below the 2^53 a number holds exactly.
        #[ts(type = "number")]
        received: u64,
        #[ts(type = "number | null")]
        total: Option<u64>,
    },
    /// Downloaded and checked; installing it restarts Folio.
    #[serde(rename_all = "camelCase")]
    Ready {
        version: String,
    },
    Error {
        message: String,
    },
    /// This copy cannot update itself, and why.
    Unsupported {
        reason: String,
    },
}
