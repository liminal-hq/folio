// Looks for a newer Folio, fetches it and installs it
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT
//
// The updater plugin does the asking and checks the signature; this decides whether the copy can
// update itself and how, keeps where the check has got to for the window to show, and runs the
// automatic check when the person has turned it on (D-50). Nothing is asked of github.com until the
// person has said so, either by turning the setting on or by choosing Check for updates.
//
// An AppImage is built from the `.deb`, so the program in it reports itself as a `.deb` install.
// It is told apart by `$APPIMAGE`, asks the manifest for `appimage-<arch>` and replaces its own
// file, because the plugin would install the download as a `.deb`.

use std::path::Path;
use std::sync::Mutex;
use std::time::Duration;

use folio_protocol::{InstallKind, UpdateState};
use tauri::utils::config::BundleType;
use tauri::{AppHandle, Emitter, Manager, Runtime, State};
use tauri_plugin_folio_settings::SettingsStore;
use tauri_plugin_store::StoreExt;
use tauri_plugin_updater::{Update, UpdaterExt};

/// The store file for the last time Folio asked, apart from the settings so that a check makes no
/// settings change for every window to hear.
const STATE_FILE: &str = "updates.json";
const LAST_CHECK_KEY: &str = "lastCheckedAt";
/// The automatic check waits this long after start, so it never competes with opening a book.
const START_DELAY: Duration = Duration::from_secs(10);
/// The shortest time between automatic checks.
const CHECK_EVERY_SECS: u64 = 24 * 60 * 60;

/// Where the check has got to, and what it found.
#[derive(Default)]
pub struct Updates {
    inner: Mutex<Inner>,
}

#[derive(Default)]
struct Inner {
    state: Option<UpdateState>,
    update: Option<Update>,
    bytes: Option<Vec<u8>>,
}

impl Updates {
    fn state(&self) -> UpdateState {
        self.inner
            .lock()
            .map(|i| i.state.clone())
            .ok()
            .flatten()
            .unwrap_or(UpdateState::Idle)
    }

    fn set<R: Runtime>(&self, app: &AppHandle<R>, state: UpdateState) {
        if let Ok(mut inner) = self.inner.lock() {
            inner.state = Some(state.clone());
        }
        let _ = app.emit("update-state", state);
    }
}

/// How this copy can be updated, or why it cannot.
///
/// `appimage` is whether `$APPIMAGE` is set. A development build has no bundle type, and so cannot.
pub fn support(bundle: Option<BundleType>, appimage: bool) -> Result<InstallKind, String> {
    match bundle {
        // The program in our AppImage is the `.deb`'s, so either type is an AppImage when `$APPIMAGE` is set.
        Some(BundleType::AppImage) => Ok(InstallKind::AppImage),
        Some(BundleType::Deb) if appimage => Ok(InstallKind::AppImage),
        Some(BundleType::Deb) => Ok(InstallKind::Deb),
        Some(BundleType::Rpm) if appimage => Ok(InstallKind::AppImage),
        Some(BundleType::Rpm) => Ok(InstallKind::Rpm),
        Some(BundleType::Msi | BundleType::Nsis) => Ok(InstallKind::Windows),
        Some(_) => Err("This kind of install cannot update itself.".into()),
        None => Err("This is a development build, which does not update itself.".into()),
    }
}

/// Whether enough time has passed since the last automatic check.
pub fn due(last: Option<u64>, now: u64) -> bool {
    last.is_none_or(|last| now.saturating_sub(last) >= CHECK_EVERY_SECS)
}

fn unix_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Replaces the AppImage at `path` with `bytes`, keeping its permissions. The new file is written
/// beside it and renamed over it, so the running copy is never half-written.
pub fn replace_appimage(path: &Path, bytes: &[u8]) -> Result<(), String> {
    // An AppImage is an ELF program.
    if !bytes.starts_with(b"\x7fELF") {
        return Err("The download is not an AppImage.".into());
    }
    let name = path
        .file_name()
        .ok_or("The AppImage has no file name.")?
        .to_string_lossy();
    let next = path.with_file_name(format!(".{name}.update"));
    let result = (|| -> std::io::Result<()> {
        let permissions = std::fs::metadata(path)?.permissions();
        std::fs::write(&next, bytes)?;
        std::fs::set_permissions(&next, permissions)?;
        std::fs::rename(&next, path)
    })();
    result.map_err(|e| {
        let _ = std::fs::remove_file(&next);
        format!("Could not replace {}: {e}", path.display())
    })
}

fn appimage_path() -> Option<std::path::PathBuf> {
    std::env::var_os("APPIMAGE").map(Into::into)
}

fn this_install() -> Result<InstallKind, String> {
    support(
        tauri::utils::platform::bundle_type(),
        appimage_path().is_some(),
    )
}

/// Asks for the newest version and records the answer.
async fn check_now<R: Runtime>(app: &AppHandle<R>) -> UpdateState {
    let updates = app.state::<Updates>();
    updates.set(app, UpdateState::Checking);
    let install = match this_install() {
        Ok(install) => install,
        Err(reason) => {
            let state = UpdateState::Unsupported { reason };
            updates.set(app, state.clone());
            return state;
        }
    };
    let mut builder = app.updater_builder();
    if install == InstallKind::AppImage {
        if let Some(path) = appimage_path() {
            builder = builder
                .target(format!("appimage-{}", std::env::consts::ARCH))
                .executable_path(path);
        }
    }
    // For testing a real update against a server of one's own; release builds never have it.
    #[cfg(feature = "update-endpoint-override")]
    if let Some(url) = std::env::var("FOLIO_UPDATE_ENDPOINT")
        .ok()
        .and_then(|u| u.parse().ok())
    {
        builder = builder.endpoints(vec![url]).unwrap_or_else(|e| {
            log::warn!("the update endpoint was refused: {e}");
            app.updater_builder()
        });
    }
    let found = match builder.build() {
        Ok(updater) => updater.check().await,
        Err(e) => Err(e),
    };
    let state = match found {
        Ok(Some(update)) => {
            let state = UpdateState::Available {
                version: update.version.clone(),
                notes: update.body.clone().filter(|n| !n.trim().is_empty()),
                date: update.date.map(|d| d.to_string()),
                install,
            };
            if let Ok(mut inner) = updates.inner.lock() {
                inner.update = Some(update);
                inner.bytes = None;
            }
            state
        }
        Ok(None) => UpdateState::UpToDate,
        Err(e) => {
            log::warn!("the update check failed: {e}");
            UpdateState::Error {
                message: e.to_string(),
            }
        }
    };
    if !matches!(state, UpdateState::Error { .. }) {
        remember_check(app);
    }
    updates.set(app, state.clone());
    state
}

fn remember_check<R: Runtime>(app: &AppHandle<R>) {
    if let Ok(store) = app.store(STATE_FILE) {
        store.set(LAST_CHECK_KEY, unix_now());
        let _ = store.save();
    }
}

fn last_check<R: Runtime>(app: &AppHandle<R>) -> Option<u64> {
    app.store(STATE_FILE)
        .ok()
        .and_then(|s| s.get(LAST_CHECK_KEY))
        .and_then(|v| v.as_u64())
}

/// Starts the automatic check, if the person has turned it on and it is due. Call once the settings
/// plugin is set up.
pub fn watch<R: Runtime>(app: &AppHandle<R>) {
    app.manage(Updates::default());
    let app = app.clone();
    std::thread::spawn(move || {
        std::thread::sleep(START_DELAY);
        let on = app
            .try_state::<SettingsStore<R>>()
            .is_some_and(|s| s.get().updates.automatic);
        if on && this_install().is_ok() && due(last_check(&app), unix_now()) {
            tauri::async_runtime::block_on(check_now(&app));
        }
    });
}

/// Where the check has got to.
#[tauri::command]
pub fn update_state(updates: State<'_, Updates>) -> UpdateState {
    match (updates.state(), this_install()) {
        // Before anything is asked, a copy that cannot update says so, so the window offers nothing.
        (UpdateState::Idle, Err(reason)) => UpdateState::Unsupported { reason },
        (state, _) => state,
    }
}

/// Looks for a newer version now, whatever the setting and the last check.
#[tauri::command]
pub async fn update_check<R: Runtime>(app: AppHandle<R>) -> UpdateState {
    check_now(&app).await
}

/// Fetches the update found and checks its signature.
#[tauri::command]
pub async fn update_download<R: Runtime>(app: AppHandle<R>) -> Result<UpdateState, String> {
    let updates = app.state::<Updates>();
    let update = updates
        .inner
        .lock()
        .ok()
        .and_then(|i| i.update.clone())
        .ok_or("There is no update to download.")?;
    updates.set(
        &app,
        UpdateState::Downloading {
            received: 0,
            total: None,
        },
    );
    let mut received = 0u64;
    let progress = app.clone();
    let result = update
        .download(
            move |chunk, total| {
                received += chunk as u64;
                progress
                    .state::<Updates>()
                    .set(&progress, UpdateState::Downloading { received, total });
            },
            || {},
        )
        .await;
    let state = match result {
        Ok(bytes) => {
            if let Ok(mut inner) = updates.inner.lock() {
                inner.bytes = Some(bytes);
            }
            UpdateState::Ready {
                version: update.version,
            }
        }
        Err(e) => UpdateState::Error {
            message: e.to_string(),
        },
    };
    updates.set(&app, state.clone());
    Ok(state)
}

/// Installs what was downloaded and restarts Folio. On Windows the installer closes it.
#[tauri::command]
pub fn update_install<R: Runtime>(app: AppHandle<R>) -> Result<(), String> {
    let updates = app.state::<Updates>();
    let (update, bytes) = {
        let inner = updates.inner.lock().map_err(|_| "Updates are busy.")?;
        (inner.update.clone(), inner.bytes.clone())
    };
    let (Some(update), Some(bytes)) = (update, bytes) else {
        return Err("There is no downloaded update to install.".into());
    };
    let outcome = match (this_install()?, appimage_path()) {
        (InstallKind::AppImage, Some(path)) => replace_appimage(&path, &bytes),
        _ => update.install(&bytes).map_err(|e| e.to_string()),
    };
    match outcome {
        Ok(()) => {
            app.restart();
        }
        Err(message) => {
            updates.set(
                &app,
                UpdateState::Error {
                    message: message.clone(),
                },
            );
            Err(message)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_kind_of_install_is_updated_its_own_way() {
        assert_eq!(support(Some(BundleType::Deb), false), Ok(InstallKind::Deb));
        assert_eq!(support(Some(BundleType::Rpm), false), Ok(InstallKind::Rpm));
        assert_eq!(
            support(Some(BundleType::Nsis), false),
            Ok(InstallKind::Windows)
        );
        assert_eq!(
            support(Some(BundleType::Msi), false),
            Ok(InstallKind::Windows)
        );
    }

    #[test]
    fn a_program_from_the_deb_running_as_an_appimage_replaces_its_own_file() {
        assert_eq!(
            support(Some(BundleType::Deb), true),
            Ok(InstallKind::AppImage)
        );
        assert_eq!(
            support(Some(BundleType::AppImage), false),
            Ok(InstallKind::AppImage)
        );
    }

    #[test]
    fn a_development_build_says_why_it_does_not_update() {
        assert!(support(None, false).unwrap_err().contains("development"));
    }

    #[test]
    fn the_automatic_check_waits_a_day() {
        let day = CHECK_EVERY_SECS;
        assert!(due(None, 1000));
        assert!(!due(Some(1000), 1000 + day - 1));
        assert!(due(Some(1000), 1000 + day));
        // A clock set back is not a reason to check again.
        assert!(!due(Some(5000), 1000));
    }

    #[cfg(unix)]
    #[test]
    fn an_appimage_is_replaced_in_place_and_keeps_its_permissions() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("Folio.AppImage");
        std::fs::write(&path, b"\x7fELF old").unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
        replace_appimage(&path, b"\x7fELF new").unwrap();
        assert_eq!(std::fs::read(&path).unwrap(), b"\x7fELF new");
        assert_eq!(
            std::fs::metadata(&path).unwrap().permissions().mode() & 0o777,
            0o755
        );
        assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 1);
    }

    #[test]
    fn something_that_is_not_an_appimage_leaves_the_old_one_alone() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("Folio.AppImage");
        std::fs::write(&path, b"\x7fELF old").unwrap();
        assert!(replace_appimage(&path, b"<html>").is_err());
        assert_eq!(std::fs::read(&path).unwrap(), b"\x7fELF old");
    }
}
