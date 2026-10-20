// Showing a file in the system's file manager
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::path::Path;
use std::process::Command;

/// Opens the file manager at `path`: selecting the file on Windows, and on Linux showing the
/// folder that holds it, since file managers there have no common way to select.
pub fn reveal(path: &Path) -> Result<(), String> {
    let folder = if path.is_dir() {
        path
    } else {
        path.parent().unwrap_or(path)
    };
    let result = if cfg!(windows) {
        Command::new("explorer")
            .arg(format!("/select,{}", path.display()))
            .spawn()
    } else if cfg!(target_os = "macos") {
        Command::new("open").arg(folder).spawn()
    } else {
        Command::new("xdg-open").arg(folder).spawn()
    };
    result
        .map(drop)
        .map_err(|e| format!("{}: {e}", folder.display()))
}
