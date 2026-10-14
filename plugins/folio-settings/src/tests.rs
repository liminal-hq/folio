// Plugin-level tests: the store and its commands run against a mock Tauri app with real windows
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use folio_settings::{
    MemoryStorage, Mode, Settings, SettingsDocument, SettingsSnapshot, SettingsStorage,
    StorageError, ToolbarLayout, Units,
};
use serde_json::{json, Value};
use tauri::test::{mock_builder, mock_context, noop_assets, MockRuntime};
use tauri::{Listener, Manager, WebviewUrl, WebviewWindowBuilder};

use crate::{commands, init, Error, SettingsStore, EVENT};

type App = tauri::App<MockRuntime>;

fn app_with(storage: Arc<dyn SettingsStorage>) -> App {
    let app = mock_builder()
        .plugin(init(storage))
        .build(mock_context(noop_assets()))
        .expect("the mock app builds");
    for label in ["main", "preferences", "about"] {
        WebviewWindowBuilder::new(&app, label, WebviewUrl::App("index.html".into()))
            .build()
            .expect("the mock window opens");
    }
    app
}

fn store(app: &App) -> tauri::State<'_, SettingsStore<MockRuntime>> {
    app.state::<SettingsStore<MockRuntime>>()
}

/// Collects the snapshots a window hears.
fn hear(app: &App, label: &str) -> Arc<Mutex<Vec<SettingsSnapshot>>> {
    let heard = Arc::new(Mutex::new(Vec::new()));
    let sink = heard.clone();
    app.get_webview_window(label)
        .unwrap()
        .listen(EVENT, move |event| {
            sink.lock()
                .unwrap()
                .push(serde_json::from_str(event.payload()).expect("a snapshot"));
        });
    heard
}

fn expert() -> Settings {
    Settings {
        mode: Mode::Expert,
        ..Settings::default()
    }
}

fn window(app: &App) -> tauri::WebviewWindow<MockRuntime> {
    app.get_webview_window("preferences").unwrap()
}

fn update(app: &App, patch: Value) -> Result<SettingsSnapshot, Error> {
    tauri::async_runtime::block_on(commands::update_settings(window(app), store(app), patch))
}

#[test]
fn a_fresh_app_starts_at_the_defaults_and_revision_zero() {
    let app = app_with(Arc::new(MemoryStorage::default()));
    assert_eq!(
        store(&app).snapshot(),
        SettingsSnapshot {
            revision: 0,
            settings: Settings::default()
        }
    );
}

#[test]
fn saved_settings_load_and_an_out_of_range_stored_value_is_brought_into_range() {
    let storage = MemoryStorage::default();
    let mut body = expert();
    body.interface.sidebar_width = 90_000;
    storage.save(&SettingsDocument::new(body)).unwrap();
    let app = app_with(Arc::new(storage));
    let loaded = store(&app).get();
    assert_eq!(loaded.mode, Mode::Expert);
    assert_eq!(loaded.interface.sidebar_width, 560);
}

#[test]
fn update_saves_bumps_the_revision_and_returns_what_is_in_force() {
    let storage = Arc::new(MemoryStorage::default());
    let app = app_with(storage.clone());
    let answer = update(&app, json!({ "mode": "expert" })).unwrap();
    assert_eq!(answer.revision, 1);
    assert_eq!(answer.settings, expert());
    assert_eq!(storage.saved(), Some(SettingsDocument::new(expert())));
    let read =
        tauri::async_runtime::block_on(commands::get_settings(window(&app), store(&app))).unwrap();
    assert_eq!(read, answer);
}

#[test]
fn a_patch_keeps_what_another_window_changed() {
    let app = app_with(Arc::new(MemoryStorage::default()));
    update(&app, json!({ "units": "millimetres" })).unwrap();
    // A window that has not heard of the units change yet turns the menu bar on.
    let after = update(&app, json!({ "interface": { "menuBar": true } })).unwrap();
    assert_eq!(after.revision, 2);
    assert!(after.settings.interface.menu_bar);
    assert_eq!(after.settings.units, Units::Millimetres);
    assert_eq!(after.settings.interface.toolbar, ToolbarLayout::Ribbon);
}

#[test]
fn every_window_hears_each_change_with_a_rising_revision() {
    let app = app_with(Arc::new(MemoryStorage::default()));
    let heard: Vec<_> = ["main", "preferences", "about"]
        .iter()
        .map(|label| hear(&app, label))
        .collect();
    update(&app, json!({ "mode": "expert" })).unwrap();
    update(&app, json!({ "colourMode": "dark" })).unwrap();
    for window in heard {
        let revisions: Vec<u64> = window.lock().unwrap().iter().map(|s| s.revision).collect();
        assert_eq!(revisions, vec![1, 2]);
    }
}

#[test]
fn a_patch_that_changes_nothing_is_quiet() {
    let storage = Arc::new(MemoryStorage::default());
    let app = app_with(storage.clone());
    let heard = hear(&app, "main");
    let answer = update(&app, json!({ "mode": "easy" })).unwrap();
    assert_eq!(answer.revision, 0);
    assert!(heard.lock().unwrap().is_empty());
    assert_eq!(storage.saved(), None, "nothing was written");
}

#[test]
fn a_value_out_of_range_is_refused_with_its_field_and_range_and_changes_nothing() {
    let storage = Arc::new(MemoryStorage::default());
    let app = app_with(storage.clone());
    let heard = hear(&app, "main");
    let refused = update(&app, json!({ "interface": { "sidebarWidth": 50 } })).unwrap_err();
    assert!(matches!(refused, Error::Invalid(_)));
    assert_eq!(
        serde_json::to_value(&refused).unwrap(),
        json!({
            "kind": "invalid",
            "message": "interface.sidebarWidth must be between 240 and 560",
            "field": "interface.sidebarWidth",
            "min": 240,
            "max": 560,
        })
    );
    assert_eq!(store(&app).snapshot().revision, 0);
    assert_eq!(storage.saved(), None);
    assert!(heard.lock().unwrap().is_empty());
}

#[test]
fn a_patch_of_the_wrong_shape_is_refused() {
    let app = app_with(Arc::new(MemoryStorage::default()));
    let refused = update(&app, json!({ "mode": "wizard" })).unwrap_err();
    assert_eq!(serde_json::to_value(&refused).unwrap()["kind"], "invalid");
    assert_eq!(store(&app).get(), Settings::default());
}

/// A storage that cannot save.
struct Failing;

impl SettingsStorage for Failing {
    fn load(&self) -> Result<Option<SettingsDocument>, StorageError> {
        Ok(None)
    }
    fn save(&self, _: &SettingsDocument) -> Result<(), StorageError> {
        Err(StorageError::Io("the disk is full".into()))
    }
}

#[test]
fn a_save_that_fails_changes_nothing_and_says_so() {
    let app = app_with(Arc::new(Failing));
    let heard = hear(&app, "main");
    let refused = update(&app, json!({ "mode": "expert" })).unwrap_err();
    assert!(matches!(refused, Error::Storage(_)));
    let wire = serde_json::to_value(&refused).unwrap();
    assert_eq!(wire["kind"], "storage");
    assert!(wire["message"]
        .as_str()
        .unwrap()
        .contains("the disk is full"));
    assert_eq!(store(&app).get(), Settings::default());
    assert!(heard.lock().unwrap().is_empty());
}

/// A storage that cannot load.
struct Unreadable;

impl SettingsStorage for Unreadable {
    fn load(&self) -> Result<Option<SettingsDocument>, StorageError> {
        Err(StorageError::Io("no such file".into()))
    }
    fn save(&self, _: &SettingsDocument) -> Result<(), StorageError> {
        Ok(())
    }
}

#[test]
fn settings_that_cannot_be_read_start_the_app_at_the_defaults() {
    let app = app_with(Arc::new(Unreadable));
    assert_eq!(store(&app).get(), Settings::default());
}

#[test]
fn reset_puts_every_setting_back_and_is_announced() {
    let app = app_with(Arc::new(MemoryStorage::default()));
    update(&app, json!({ "mode": "expert", "units": "millimetres" })).unwrap();
    let heard = hear(&app, "main");
    let after = tauri::async_runtime::block_on(commands::reset_settings(window(&app), store(&app)))
        .unwrap();
    assert_eq!(after.settings, Settings::default());
    assert_eq!(after.revision, 2);
    assert_eq!(heard.lock().unwrap().len(), 1);
}

#[test]
fn a_hook_hears_changes_but_not_refusals_or_no_ops() {
    let app = app_with(Arc::new(MemoryStorage::default()));
    let seen = Arc::new(Mutex::new(Vec::new()));
    let sink = seen.clone();
    store(&app).on_change(move |settings| sink.lock().unwrap().push(settings.clone()));
    update(&app, json!({})).unwrap();
    update(&app, json!({ "interface": { "sidebarWidth": 1 } })).unwrap_err();
    update(&app, json!({ "mode": "expert" })).unwrap();
    assert_eq!(*seen.lock().unwrap(), vec![expert()]);
}

#[test]
fn a_hook_runs_after_the_new_settings_are_readable() {
    let app = app_with(Arc::new(MemoryStorage::default()));
    let handle = app.handle().clone();
    let consistent = Arc::new(AtomicBool::new(false));
    let flag = consistent.clone();
    store(&app).on_change(move |settings| {
        let now = handle.state::<SettingsStore<MockRuntime>>().get();
        flag.store(now == *settings, Ordering::SeqCst);
    });
    update(&app, json!({ "mode": "expert" })).unwrap();
    assert!(consistent.load(Ordering::SeqCst));
}

#[test]
fn the_status_command_reports_the_plugin_available() {
    let status = tauri::async_runtime::block_on(commands::get_status());
    assert!(status.available);
    assert_eq!(status.features, vec!["settings", "events"]);
}
