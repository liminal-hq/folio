// Opens dialog windows, blocks their openers and reports how each one ended
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use serde_json::Value;
use std::sync::{Arc, Mutex};
use tauri::{
    AppHandle, Emitter, Manager, Runtime, WebviewUrl, WebviewWindow, WebviewWindowBuilder,
    WindowEvent,
};

use crate::error::Error;
use crate::models::{
    DialogSpec, Finished, ModalState, Opened, Posted, FINISHED_EVENT, MESSAGE_EVENT, MODAL_EVENT,
};
use crate::platform;
use crate::registry::{Opening, Registry};

/// Called with a dialog's label and its new logical size when its window is resized.
pub type OnResized = Arc<dyn Fn(&str, f64, f64) + Send + Sync>;

/// What the app can ask of the plugin when it registers it.
#[derive(Clone, Default)]
pub struct Options {
    /// Told when a dialog window is resized, so the app can remember its size.
    pub on_resized: Option<OnResized>,
}

/// The plugin's state.
pub struct Dialogs {
    pub(crate) registry: Mutex<Registry>,
    pub(crate) options: Options,
}

impl Dialogs {
    pub fn new(options: Options) -> Self {
        Self {
            registry: Mutex::new(Registry::default()),
            options,
        }
    }

    fn registry(&self) -> std::sync::MutexGuard<'_, Registry> {
        self.registry.lock().expect("the registry is not poisoned")
    }
}

/// Opens the dialog window `label` for `opener` (none for a dialog nothing blocks or belongs to),
/// or brings it forward when it is already open. Build windows from an `async` command, not a
/// synchronous one: a synchronous command runs on the main thread, and building a webview there
/// waits on that thread.
pub fn open<R: Runtime>(
    app: &AppHandle<R>,
    opener: Option<&WebviewWindow<R>>,
    label: &str,
    spec: &DialogSpec,
    payload: Value,
) -> Result<Opened, Error> {
    let state = app.state::<Dialogs>();
    let opener_label = opener.map(|w| w.label().to_owned());
    let opening = state
        .registry()
        .begin(label, opener_label.as_deref(), spec.modal, payload);
    let request_id = match opening {
        Opening::Refused { modal } => return Err(Error::ModalOpen(modal)),
        Opening::Existing { request_id } => {
            if let Some(window) = app.get_webview_window(label) {
                let _ = window.unminimize();
                let _ = window.show();
                let _ = window.set_focus();
            }
            return Ok(Opened {
                request_id,
                existing: true,
            });
        }
        Opening::New { request_id } => request_id,
    };

    let window = match build(app, opener, label, spec) {
        Ok(window) => window,
        Err(error) => {
            state.registry().take(label);
            return Err(Error::Internal(error.to_string()));
        }
    };

    let watched = window.clone();
    let on_resized = state.options.on_resized.clone();
    let app_for_events = app.clone();
    let label_for_events = label.to_owned();
    window.on_window_event(move |event| match event {
        // Before the window is destroyed, so an owner that was disabled is enabled and focused
        // while the dialog still exists.
        WindowEvent::CloseRequested { .. } | WindowEvent::Destroyed => {
            ended(&app_for_events, &label_for_events, None);
        }
        WindowEvent::Resized(size) => {
            if let Some(callback) = &on_resized {
                let scale = watched.scale_factor().unwrap_or(1.0);
                callback(
                    &label_for_events,
                    size.width as f64 / scale,
                    size.height as f64 / scale,
                );
            }
        }
        _ => {}
    });

    if let (true, Some(opener)) = (spec.modal, opener) {
        let native = platform::block(&window, opener);
        let _ = app.emit_to(
            opener.label(),
            MODAL_EVENT,
            ModalState {
                open: true,
                label: label.to_owned(),
                native,
            },
        );
    }
    log::debug!("dialog {label}: opened (request {request_id})");
    Ok(Opened {
        request_id,
        existing: false,
    })
}

fn build<R: Runtime>(
    app: &AppHandle<R>,
    opener: Option<&WebviewWindow<R>>,
    label: &str,
    spec: &DialogSpec,
) -> tauri::Result<WebviewWindow<R>> {
    let url = spec.url.as_deref().unwrap_or("index.html");
    let mut builder = WebviewWindowBuilder::new(app, label, WebviewUrl::App(url.into()))
        .title(&spec.title)
        .inner_size(spec.width, spec.height)
        .resizable(spec.resizable)
        // A dialog is never maximised or minimised, resizable or not; it is a child of its opener.
        .maximizable(false)
        .minimizable(false)
        .skip_taskbar(true);
    if let (Some(width), Some(height)) = (spec.min_width, spec.min_height) {
        builder = builder.min_inner_size(width, height);
    }
    if spec.chromeless {
        builder = builder.decorations(false).transparent(true).shadow(true);
    }
    if let Some(opener) = opener {
        builder = builder.parent(opener)?;
    }
    builder = match centred_over(opener, spec) {
        Some((x, y)) => builder.position(x, y),
        None => builder.center(),
    };
    builder.build()
}

/// Where a dialog of the spec's size sits centred over its opener, in logical pixels. None where
/// the compositor places windows, or when the opener's place is not known.
fn centred_over<R: Runtime>(
    opener: Option<&WebviewWindow<R>>,
    spec: &DialogSpec,
) -> Option<(f64, f64)> {
    if !platform::places_windows() {
        return None;
    }
    let opener = opener?;
    let scale = opener.scale_factor().ok()?;
    let at = opener.outer_position().ok()?.to_logical::<f64>(scale);
    let size = opener.outer_size().ok()?.to_logical::<f64>(scale);
    Some((
        at.x + (size.width - spec.width) / 2.0,
        at.y + (size.height - spec.height) / 2.0,
    ))
}

/// Ends the dialog `label`: releases its opener and tells it how it ended, once. `result` is
/// none when the dialog was closed rather than finished.
pub fn ended<R: Runtime>(app: &AppHandle<R>, label: &str, result: Option<Value>) {
    let Some(state) = app.try_state::<Dialogs>() else {
        return;
    };
    let Some(closing) = state.registry().take(label) else {
        return;
    };
    if let Some(name) = closing.opener.as_deref() {
        if let Some(opener) = app.get_webview_window(name) {
            if closing.modal {
                platform::unblock(&opener);
                let _ = app.emit_to(
                    name,
                    MODAL_EVENT,
                    ModalState {
                        open: false,
                        label: label.to_owned(),
                        native: true,
                    },
                );
            }
            // Focus goes back to the page that asked, window and webview both.
            let _ = opener.show();
            let _ = opener.set_focus();
        }
        let _ = app.emit_to(
            name,
            FINISHED_EVENT,
            Finished {
                request_id: closing.request_id,
                label: label.to_owned(),
                cancelled: result.is_none(),
                result,
            },
        );
    }
    log::debug!("dialog {label}: ended (request {})", closing.request_id);
}

/// Posts a message from the dialog `label` to its opener while the dialog stays open.
pub fn post<R: Runtime>(app: &AppHandle<R>, label: &str, message: Value) -> Result<(), Error> {
    let state = app.state::<Dialogs>();
    let (request_id, opener) = state.registry().opener_of(label).ok_or(Error::NotADialog)?;
    if let Some(opener) = opener {
        let _ = app.emit_to(
            &opener,
            MESSAGE_EVENT,
            Posted {
                request_id,
                label: label.to_owned(),
                message,
            },
        );
    }
    Ok(())
}
