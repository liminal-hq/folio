// The settings file: a key-value view over `tauri-plugin-store` that sets an unreadable file aside
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT
//
// The file lives in the app data directory (`ca.liminalhq.folio`). `tauri-plugin-store` ignores a
// file it cannot parse and the next save would overwrite it, so a file that is not a JSON object is
// copied to `{name}.corrupt-{unix}` first, once per run, and the settings fall back to the copy of
// the run before (see `folio_settings::Persistence`).

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use folio_settings::KeyValue;
use serde_json::Value;
use tauri::{AppHandle, Manager, Runtime};
use tauri_plugin_store::StoreExt;

/// The store file, relative to the app data directory.
pub const SETTINGS_FILE: &str = "settings.json";

/// A store file opened through `tauri-plugin-store`.
pub struct FileKeyValue<R: Runtime> {
    store: Arc<tauri_plugin_store::Store<R>>,
    path: PathBuf,
    unreadable: bool,
}

fn unix_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Copies `path` to `{path}.corrupt-{unix}`, a new name each time.
fn copy_aside(path: &Path) -> Option<String> {
    let name = path.file_name()?.to_string_lossy().into_owned();
    let now = unix_now();
    for attempt in 0..100u32 {
        let suffix = if attempt == 0 {
            format!("{name}.corrupt-{now}")
        } else {
            format!("{name}.corrupt-{now}-{attempt}")
        };
        let target = path.with_file_name(&suffix);
        if !target.exists() {
            return std::fs::copy(path, &target).ok().map(|_| suffix);
        }
    }
    None
}

/// Files already copied aside in this run, with the name of the copy, so opening a file twice does
/// not leave two copies.
static SET_ASIDE: Mutex<Vec<(PathBuf, Option<String>)>> = Mutex::new(Vec::new());

/// `copy_aside`, once per path per run: later calls report the first call's copy.
fn copy_aside_once(path: &Path) -> Option<String> {
    let mut done = SET_ASIDE.lock().unwrap_or_else(|e| e.into_inner());
    if let Some((_, name)) = done.iter().find(|(seen, _)| seen == path) {
        return name.clone();
    }
    let name = copy_aside(path);
    done.push((path.to_path_buf(), name.clone()));
    name
}

impl<R: Runtime> FileKeyValue<R> {
    /// Opens the store file `file` in the app data directory.
    pub fn open_file(app: &AppHandle<R>, file: &str) -> Result<Self, String> {
        let dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
        let path = dir.join(file);
        let unreadable = std::fs::read(&path).is_ok_and(|bytes| {
            !matches!(
                serde_json::from_slice::<Value>(&bytes),
                Ok(Value::Object(_))
            )
        });
        if unreadable {
            log::warn!("the store file {} is not a JSON object", path.display());
            match copy_aside_once(&path) {
                Some(name) => log::warn!("kept a copy of the store file as `{name}`"),
                None => log::warn!("could not keep a copy of the unreadable store file"),
            }
        }
        let store = app.store(file).map_err(|e| e.to_string())?;
        Ok(Self {
            store,
            path,
            unreadable,
        })
    }
}

impl<R: Runtime> KeyValue for FileKeyValue<R> {
    fn get(&self, key: &str) -> Option<Value> {
        self.store.get(key)
    }

    fn set(&self, key: &str, value: Value) {
        self.store.set(key, value);
    }

    fn save(&self) -> Result<(), String> {
        self.store.save().map_err(|e| e.to_string())
    }

    fn set_aside(&self) -> Option<String> {
        copy_aside(&self.path)
    }

    fn was_unreadable(&self) -> bool {
        self.unreadable
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_file_opened_twice_is_copied_aside_once() {
        let dir = tempfile::tempdir().expect("a temporary directory");
        let path = dir.path().join("settings.json");
        std::fs::write(&path, b"not json").unwrap();
        let first = copy_aside_once(&path);
        let second = copy_aside_once(&path);
        assert!(first.is_some());
        assert_eq!(first, second);
        let copies = std::fs::read_dir(dir.path())
            .unwrap()
            .filter(|e| {
                e.as_ref()
                    .unwrap()
                    .file_name()
                    .to_string_lossy()
                    .contains(".corrupt-")
            })
            .count();
        assert_eq!(copies, 1);
    }
}
