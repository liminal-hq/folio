// What the app reports about itself to every window.
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// The application's name, version and identity, for the About window and the logs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub struct AppInfo {
    /// The product name, "Folio".
    pub name: String,
    /// The application version, from the Cargo manifest.
    pub version: String,
    /// The bundle identifier, which also names the data and log directories.
    pub identifier: String,
    /// The operating system the app is running on (`linux`, `windows`, ...).
    pub platform: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialises_with_camel_case_keys() {
        let info = AppInfo {
            name: "Folio".into(),
            version: "0.0.0".into(),
            identifier: "ca.liminalhq.folio".into(),
            platform: "linux".into(),
        };
        let json = serde_json::to_value(&info).expect("serialises");
        assert_eq!(json["identifier"], "ca.liminalhq.folio");
        assert_eq!(json["platform"], "linux");
    }
}
