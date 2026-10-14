// Reports the application's name, version and identity to the pages
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use folio_protocol::AppInfo;
use tauri::{AppHandle, Runtime};

/// Assembles the report from the pieces the app knows.
pub fn describe(name: &str, version: &str, identifier: &str, platform: &str) -> AppInfo {
    AppInfo {
        name: name.to_string(),
        version: version.to_string(),
        identifier: identifier.to_string(),
        platform: platform.to_string(),
    }
}

/// What the application reports about itself, for the About window and the logs.
#[tauri::command]
pub fn app_info<R: Runtime>(app: AppHandle<R>) -> AppInfo {
    let config = app.config();
    describe(
        config.product_name.as_deref().unwrap_or("Folio"),
        &app.package_info().version.to_string(),
        &config.identifier,
        std::env::consts::OS,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn describes_the_app() {
        let info = describe("Folio", "1.2.3", "ca.liminalhq.folio", "linux");
        assert_eq!(info.name, "Folio");
        assert_eq!(info.identifier, "ca.liminalhq.folio");
    }

    #[test]
    fn the_command_reports_the_configured_identity() {
        let app = tauri::test::mock_app();
        let info = app_info(app.handle().clone());
        assert_eq!(info.platform, std::env::consts::OS);
        assert_eq!(info.name, "Folio");
    }
}
