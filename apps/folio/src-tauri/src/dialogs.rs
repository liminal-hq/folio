// Opens the app's dialogs as windows of their own, through the window-dialogs plugin
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT
//
// The plugin knows nothing of Folio's dialogs. This module is where a `DialogKind` becomes a
// window: its title, its size (the one it was last left at, for the ones that can be resized) and
// whether it blocks the window that opened it.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use folio_protocol::DialogKind;
use serde_json::{json, Value};
use tauri::{AppHandle, Manager, Runtime, WebviewWindow};
use tauri_plugin_folio_settings::SettingsStore;
use tauri_plugin_window_dialogs::{models::DialogSpec, OnResized, Options};

/// A modal dialog that cannot be resized.
fn fixed(title: &str, width: f64, height: f64) -> DialogSpec {
    DialogSpec {
        title: title.into(),
        width,
        height,
        min_width: None,
        min_height: None,
        resizable: false,
        modal: true,
        chromeless: true,
        url: None,
    }
}

/// A modal dialog that can be resized, down to a minimum, and remembers its size.
fn resizable(title: &str, width: f64, height: f64, min_width: f64, min_height: f64) -> DialogSpec {
    DialogSpec {
        resizable: true,
        min_width: Some(min_width),
        min_height: Some(min_height),
        ..fixed(title, width, height)
    }
}

/// How a kind of dialog is made, in logical pixels.
pub fn spec_for(kind: DialogKind) -> DialogSpec {
    match kind {
        DialogKind::Export => resizable("Export book", 640.0, 720.0, 480.0, 420.0),
        DialogKind::BookDetails => fixed("Book details", 600.0, 760.0),
        DialogKind::Barcode => fixed("Barcode", 560.0, 680.0),
        DialogKind::BookSize => fixed("Book size", 460.0, 380.0),
        DialogKind::Margins => fixed("Margins", 460.0, 440.0),
        DialogKind::SpreadTemplates => resizable("Spread templates", 900.0, 680.0, 560.0, 420.0),
        DialogKind::ImportReport => resizable("Import report", 700.0, 640.0, 480.0, 360.0),
        DialogKind::CoverSheet => resizable("Cover sheet", 1000.0, 760.0, 640.0, 480.0),
        DialogKind::SoftProof => resizable("Soft proof", 900.0, 760.0, 560.0, 420.0),
        // The page behind shows each try at once, so the window is kept small enough to move aside.
        DialogKind::PageBackground => fixed("Page background", 540.0, 560.0),
        // The book stays usable while the list is open: "Show me" moves its canvas.
        DialogKind::CheckBook => DialogSpec {
            modal: false,
            ..resizable("Check book", 620.0, 700.0, 440.0, 360.0)
        },
    }
}

/// The spec with the size the dialog was left at, when there is one and the dialog can be resized.
pub fn spec_with_size(kind: DialogKind, remembered: Option<(u32, u32)>) -> DialogSpec {
    let mut spec = spec_for(kind);
    if let (true, Some((width, height))) = (spec.resizable, remembered) {
        spec.width = f64::from(width).max(spec.min_width.unwrap_or(0.0));
        spec.height = f64::from(height).max(spec.min_height.unwrap_or(0.0));
    }
    spec
}

/// Opens the dialog of `kind` for the window that asked, with `payload` for it to read. The
/// command is `async` for the reason `windows::open_window` is: a webview cannot be built from a
/// synchronous command on Windows.
#[tauri::command]
pub async fn open_dialog<R: Runtime>(
    app: AppHandle<R>,
    window: WebviewWindow<R>,
    kind: DialogKind,
    payload: Value,
) -> Result<(), String> {
    let remembered = app
        .try_state::<SettingsStore<R>>()
        .and_then(|settings| settings.get().dialogs.sizes.get(kind.label()).copied())
        .map(|size| (size.width, size.height));
    tauri_plugin_window_dialogs::open(
        &app,
        Some(&window),
        kind.label(),
        &spec_with_size(kind, remembered),
        payload,
    )
    .map(drop)
    .map_err(|e| e.to_string())
}

/// The latest size reported for each dialog, with a count that tells a newer report from an older.
type Latest = Arc<Mutex<HashMap<String, (u64, f64, f64)>>>;

/// How long a dialog must stop being resized before its size is saved.
const RESIZE_SETTLE: Duration = Duration::from_millis(600);

/// The plugin options: a dialog window's size is saved when it has stopped changing.
pub fn options<R: Runtime>(app: &AppHandle<R>) -> Options {
    let latest: Latest = Arc::default();
    let app = app.clone();
    let on_resized: OnResized = Arc::new(move |label, width, height| {
        if DialogKind::from_label(label).is_none() {
            return;
        }
        let generation = {
            let mut latest = latest.lock().expect("the sizes are not poisoned");
            let generation = latest.get(label).map_or(1, |(n, _, _)| n + 1);
            latest.insert(label.to_owned(), (generation, width, height));
            generation
        };
        let (app, latest, label) = (app.clone(), latest.clone(), label.to_owned());
        std::thread::spawn(move || {
            std::thread::sleep(RESIZE_SETTLE);
            let still = latest
                .lock()
                .expect("the sizes are not poisoned")
                .get(&label)
                .is_some_and(|(n, _, _)| *n == generation);
            if !still {
                return;
            }
            remember(&app, &label, width, height);
        });
    });
    Options {
        on_resized: Some(on_resized),
    }
}

/// Saves the size of a dialog window in the settings (a size that is not sensible is not saved).
fn remember<R: Runtime>(app: &AppHandle<R>, label: &str, width: f64, height: f64) {
    let Some(settings) = app.try_state::<SettingsStore<R>>() else {
        return;
    };
    let (width, height) = (width.round() as u32, height.round() as u32);
    let patch = json!({ "dialogs": { "sizes": { label: { "width": width, "height": height } } } });
    if let Err(error) = settings.update(&patch) {
        log::debug!("did not remember the size of {label}: {error}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_dialog_that_can_be_resized_opens_at_the_size_it_was_left_at() {
        let spec = spec_with_size(DialogKind::Export, Some((900, 800)));
        assert_eq!((spec.width, spec.height), (900.0, 800.0));
    }

    #[test]
    fn a_remembered_size_is_never_below_the_minimum() {
        let spec = spec_with_size(DialogKind::Export, Some((300, 300)));
        assert_eq!((spec.width, spec.height), (480.0, 420.0));
    }

    #[test]
    fn without_a_remembered_size_the_default_applies() {
        let spec = spec_with_size(DialogKind::Export, None);
        assert_eq!((spec.width, spec.height), (640.0, 720.0));
    }

    #[test]
    fn every_kind_has_a_title_and_only_check_book_leaves_the_opener_usable() {
        for kind in DialogKind::ALL {
            let spec = spec_for(kind);
            assert!(!spec.title.is_empty());
            assert_eq!(spec.modal, kind != DialogKind::CheckBook, "{kind:?}");
        }
    }
}
