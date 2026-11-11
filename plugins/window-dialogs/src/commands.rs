// Implements the IPC commands of the window-dialogs plugin
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use serde_json::Value;
use tauri::{AppHandle, Runtime, State, WebviewWindow};

use crate::error::Error;
use crate::models::{DialogRequest, DialogSpec, Opened};
use crate::session::{self, Dialogs};

/// Opens the dialog window `label` for the window that asked, with `payload` for the dialog to
/// read. The opener hears how it ended on `window-dialogs://finished`.
#[tauri::command]
pub async fn open_dialog<R: Runtime>(
    app: AppHandle<R>,
    window: WebviewWindow<R>,
    label: String,
    spec: DialogSpec,
    payload: Value,
) -> Result<Opened, Error> {
    session::open(&app, Some(&window), &label, &spec, payload)
}

/// What the dialog window that asks was opened for. The dialog asks when it starts, so there is no
/// race between its opening and its payload arriving.
#[tauri::command]
pub async fn dialog_request<R: Runtime>(
    window: WebviewWindow<R>,
    state: State<'_, Dialogs>,
) -> Result<DialogRequest, Error> {
    let registry = state.registry.lock().expect("the registry is not poisoned");
    if let Some(ms) = registry.age_ms(window.label()) {
        log::debug!(
            "dialog {}: asked for its request after {ms} ms",
            window.label()
        );
    }
    registry.request(window.label()).ok_or(Error::NotADialog)
}

/// Says the dialog has painted its content; returns the milliseconds since it was opened.
#[tauri::command]
pub async fn dialog_shown<R: Runtime>(
    window: WebviewWindow<R>,
    state: State<'_, Dialogs>,
) -> Result<u64, Error> {
    let registry = state.registry.lock().expect("the registry is not poisoned");
    let ms = registry.age_ms(window.label()).ok_or(Error::NotADialog)?;
    log::info!(
        "dialog {}: first paint {ms} ms after opening",
        window.label()
    );
    Ok(ms as u64)
}

/// Ends the dialog with a result: the opener gets it, and the window closes.
#[tauri::command]
pub async fn finish_dialog<R: Runtime>(
    app: AppHandle<R>,
    window: WebviewWindow<R>,
    result: Value,
) -> Result<(), Error> {
    session::ended(&app, window.label(), Some(result));
    window.close().map_err(|e| Error::Internal(e.to_string()))
}

/// Sends a message from the dialog to its opener while the dialog stays open.
#[tauri::command]
pub async fn post_to_opener<R: Runtime>(
    app: AppHandle<R>,
    window: WebviewWindow<R>,
    message: Value,
) -> Result<(), Error> {
    session::post(&app, window.label(), message)
}

/// Brings the dialog forward and focuses it. The opener's blocker calls this when it is clicked.
#[tauri::command]
pub async fn focus_dialog<R: Runtime>(
    app: AppHandle<R>,
    window: WebviewWindow<R>,
    state: State<'_, Dialogs>,
) -> Result<(), Error> {
    let label = state
        .registry
        .lock()
        .expect("the registry is not poisoned")
        .modal_of(window.label())
        .map(str::to_owned);
    if let Some(dialog) = label.and_then(|l| tauri::Manager::get_webview_window(&app, &l)) {
        let _ = dialog.unminimize();
        let _ = dialog.show();
        let _ = dialog.set_focus();
    }
    Ok(())
}
