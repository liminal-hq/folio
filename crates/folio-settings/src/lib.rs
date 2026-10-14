// Folio's application settings: the typed document, its validation and its storage
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT
//
// This crate owns everything the Preferences window edits and the remembered interface state
// (mode, toolbar, sidebar). Per-book settings live inside the `.folio` file and never here. It has
// no `tauri` dependency: the plugin in `plugins/folio-settings` is the adapter, and the app
// supplies the key-value file the document is saved in.

mod model;
mod storage;

pub use model::{
    push_recent, ColourMode, InterfaceSettings, JaxHelp, MarkSetting, MarksSettings, Mode,
    Settings, SettingsError, SettingsSnapshot, SidebarLayout, ToolbarLayout, Units, MARK_COLOURS,
    RECENT_BOOKS_MAX, SIDEBAR_WIDTH_MAX, SIDEBAR_WIDTH_MIN,
};
pub use storage::{
    KeyValue, MemoryStorage, Persistence, SettingsDocument, SettingsStorage, StorageError,
    DOCUMENT_VERSION, PREVIOUS_KEY, SETTINGS_KEY,
};
