// The settings state: one document, one writer, a revision and a change event to every window
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::sync::{Arc, Mutex, MutexGuard};

use folio_settings::{Settings, SettingsDocument, SettingsSnapshot, SettingsStorage};
use serde_json::Value;
use tauri::{AppHandle, Emitter, Runtime};

use crate::error::Error;
use crate::EVENT;

/// Runs with the new settings after every change, outside the plugin's locks. The app subscribes
/// to push what other state depends on without the plugin knowing.
pub type ChangeHook = Arc<dyn Fn(&Settings) + Send + Sync>;

fn locked<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    // A poisoned lock only means a hook panicked; the state is still consistent.
    mutex.lock().unwrap_or_else(|e| e.into_inner())
}

/// The application settings every window reads. Rust owns them: a window asks for a change and
/// hears the answer as an event carrying the new revision.
pub struct SettingsStore<R: Runtime> {
    app: AppHandle<R>,
    storage: Arc<dyn SettingsStorage>,
    current: Mutex<SettingsSnapshot>,
    /// Held across a whole change, so two changes never interleave their save, event and hooks.
    writer: Mutex<()>,
    hooks: Mutex<Vec<ChangeHook>>,
}

impl<R: Runtime> SettingsStore<R> {
    /// Loads what the storage holds. Settings that cannot be read are never fatal: the defaults
    /// apply, and an out-of-range value from a hand-edited file is brought into range.
    pub fn new(app: AppHandle<R>, storage: Arc<dyn SettingsStorage>) -> Self {
        let settings = match storage.load() {
            Ok(Some(document)) => document.body.clamped(),
            Ok(None) => Settings::default(),
            Err(why) => {
                log::warn!("could not load the settings, using the defaults: {why}");
                Settings::default()
            }
        };
        Self {
            app,
            storage,
            current: Mutex::new(SettingsSnapshot {
                revision: 0,
                settings,
            }),
            writer: Mutex::new(()),
            hooks: Mutex::new(Vec::new()),
        }
    }

    /// The settings in force and the revision they are at.
    pub fn snapshot(&self) -> SettingsSnapshot {
        locked(&self.current).clone()
    }

    pub fn get(&self) -> Settings {
        self.snapshot().settings
    }

    /// Runs `hook` after every change. It is not run for the settings loaded at start-up: the
    /// caller reads those with `get`.
    pub fn on_change(&self, hook: impl Fn(&Settings) + Send + Sync + 'static) {
        locked(&self.hooks).push(Arc::new(hook));
    }

    /// Lays `patch` (a JSON merge patch) over the settings in force: validated, saved, then in
    /// force and announced to every window. The merge happens under the writer lock, so a change
    /// another window just made is kept. A value out of range, or a save that fails, changes
    /// nothing; neither does a patch that leaves the settings as they are (no revision, no event).
    pub fn update(&self, patch: &Value) -> Result<SettingsSnapshot, Error> {
        let _writer = locked(&self.writer);
        let settings = self.get().patched(patch)?;
        self.commit(settings)
    }

    /// Puts every setting back to its default.
    pub fn reset(&self) -> Result<SettingsSnapshot, Error> {
        let _writer = locked(&self.writer);
        self.commit(Settings::default())
    }

    /// Saves, applies and announces `settings`; the caller holds the writer lock.
    fn commit(&self, settings: Settings) -> Result<SettingsSnapshot, Error> {
        let before = self.snapshot();
        if before.settings == settings {
            return Ok(before);
        }
        self.storage
            .save(&SettingsDocument::new(settings.clone()))
            .map_err(|e| Error::Storage(e.to_string()))?;
        let after = SettingsSnapshot {
            revision: before.revision + 1,
            settings: settings.clone(),
        };
        *locked(&self.current) = after.clone();
        if let Err(e) = self.app.emit(EVENT, after.clone()) {
            log::warn!("could not announce the settings change: {e}");
        }
        let hooks: Vec<ChangeHook> = locked(&self.hooks).clone();
        for hook in hooks {
            hook(&settings);
        }
        Ok(after)
    }
}
