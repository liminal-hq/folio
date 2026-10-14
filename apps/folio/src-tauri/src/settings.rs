// Wires the settings plugin to its file
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT
//
// The document is saved in `settings.json` under the key `settings`, with the run before under
// `settingsPrevious`. A file that cannot be opened leaves the settings in memory for this run, with
// a warning: they must never stop the app starting.

use std::sync::Arc;

use folio_settings::{MemoryStorage, Persistence, SettingsStorage};
use tauri::{AppHandle, Wry};

use crate::storage::{FileKeyValue, SETTINGS_FILE};

/// The settings plugin's storage, made once the app exists.
pub fn storage(app: &AppHandle<Wry>) -> Arc<dyn SettingsStorage> {
    match FileKeyValue::open_file(app, SETTINGS_FILE) {
        Ok(kv) => Arc::new(Persistence::new(kv)),
        Err(e) => {
            log::warn!("could not open the settings file, settings will not be saved: {e}");
            Arc::new(MemoryStorage::default())
        }
    }
}
