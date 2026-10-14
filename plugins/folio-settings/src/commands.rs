// The plugin's commands: read the settings, change them, and report that the plugin works
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use folio_protocol::PluginStatus;
use folio_settings::SettingsSnapshot;
use serde_json::Value;
use tauri::{Runtime, State, WebviewWindow};

use crate::error::Error;
use crate::state::SettingsStore;

/// Reports that the settings work. They need nothing from the system, so this is always available.
#[tauri::command]
pub async fn get_status() -> PluginStatus {
    PluginStatus::available(vec!["settings".to_owned(), "events".to_owned()])
}

/// The settings in force and their revision.
#[tauri::command]
pub async fn get_settings<R: Runtime>(
    _window: WebviewWindow<R>,
    store: State<'_, SettingsStore<R>>,
) -> Result<SettingsSnapshot, Error> {
    Ok(store.snapshot())
}

/// Lays a JSON merge patch over the settings in force, saves the result and announces it to every
/// window. A window sends only what it changed, so one holding an older copy of the document
/// cannot turn another window's change back. Returns what is now in force; a value out of range
/// rejects with `{ kind: "invalid", message, field, min, max }` and changes nothing.
#[tauri::command]
pub async fn update_settings<R: Runtime>(
    _window: WebviewWindow<R>,
    store: State<'_, SettingsStore<R>>,
    patch: Value,
) -> Result<SettingsSnapshot, Error> {
    store.update(&patch)
}

/// Puts every setting back to its default.
#[tauri::command]
pub async fn reset_settings<R: Runtime>(
    _window: WebviewWindow<R>,
    store: State<'_, SettingsStore<R>>,
) -> Result<SettingsSnapshot, Error> {
    store.reset()
}
