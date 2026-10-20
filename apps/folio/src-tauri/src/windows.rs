// Creates the app's windows: every window is built here, never from `tauri.conf.json`
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT
//
// `tauri.conf.json` keeps `"windows": []`, so decorations, transparency, shadow and size limits
// are chosen for every window in one place. Windows are frameless and transparent, and shadowed
// (`shadow: true`): Windows 11 draws the shadow and the rounded corners, while on Linux Tauri
// cannot, so the page's window frame draws its own inside a small transparent margin (D-13).

use folio_protocol::WindowKind;
use tauri::{AppHandle, Manager, Runtime, WebviewUrl, WebviewWindow, WebviewWindowBuilder};

/// How a kind of window is made, in logical pixels.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WindowSpec {
    pub title: &'static str,
    pub size: (f64, f64),
    pub min_size: Option<(f64, f64)>,
    pub resizable: bool,
    /// Keeps the window off the taskbar: it belongs to the main window, not the desktop.
    pub skip_taskbar: bool,
    /// Opens as a child of the main window, so it stays above it and closes with it.
    pub child_of_main: bool,
}

/// The title Folio's windows carry before the page names them (the page sets the real one).
const APP_TITLE: &str = "Folio";

/// The spec for a kind of window.
pub fn spec_for(kind: WindowKind) -> WindowSpec {
    match kind {
        WindowKind::Main => WindowSpec {
            title: APP_TITLE,
            size: (1440.0, 900.0),
            min_size: Some((1024.0, 680.0)),
            resizable: true,
            skip_taskbar: false,
            child_of_main: false,
        },
        WindowKind::About => WindowSpec {
            title: "About Folio",
            size: (560.0, 760.0),
            min_size: None,
            resizable: false,
            skip_taskbar: true,
            child_of_main: false,
        },
        WindowKind::Preferences => WindowSpec {
            title: "Preferences",
            size: (860.0, 620.0),
            min_size: Some((560.0, 440.0)),
            resizable: true,
            skip_taskbar: false,
            child_of_main: false,
        },
        WindowKind::BookTools => WindowSpec {
            title: "Book Tools",
            size: (1180.0, 760.0),
            min_size: Some((900.0, 600.0)),
            resizable: true,
            skip_taskbar: false,
            child_of_main: true,
        },
        WindowKind::WizardNew => WindowSpec {
            title: "New book",
            size: (880.0, 640.0),
            min_size: None,
            resizable: false,
            skip_taskbar: true,
            child_of_main: true,
        },
    }
}

/// Builds the window for `kind`, centred and shown.
pub fn build_window<R: Runtime>(
    app: &AppHandle<R>,
    kind: WindowKind,
) -> tauri::Result<WebviewWindow<R>> {
    let spec = spec_for(kind);
    let mut builder =
        WebviewWindowBuilder::new(app, kind.label(), WebviewUrl::App("index.html".into()))
            .title(spec.title)
            .inner_size(spec.size.0, spec.size.1)
            .resizable(spec.resizable)
            // A fixed window cannot be maximised either: the button would do nothing.
            .maximizable(spec.resizable)
            .skip_taskbar(spec.skip_taskbar)
            .decorations(false)
            .transparent(true)
            .shadow(true)
            .center();
    if let Some((width, height)) = spec.min_size {
        builder = builder.min_inner_size(width, height);
    }
    if spec.child_of_main {
        // On Wayland a window's place cannot be set, so a child is placed by the compositor over
        // its parent. Without a main window there is nothing to belong to.
        if let Some(main) = app.get_webview_window(WindowKind::Main.label()) {
            builder = builder.parent(&main)?;
        }
    }
    builder.build()
}

/// Focuses the window for `kind` if it is open, and otherwise builds it.
pub fn open_or_focus<R: Runtime>(
    app: &AppHandle<R>,
    kind: WindowKind,
) -> tauri::Result<WebviewWindow<R>> {
    if let Some(window) = app.get_webview_window(kind.label()) {
        window.unminimize()?;
        window.show()?;
        window.set_focus()?;
        return Ok(window);
    }
    build_window(app, kind)
}

/// Opens a window of `kind`, or focuses it when it is already open. Reopening never makes a second.
///
/// The command is `async` on purpose: a synchronous command runs on the main thread, and on Windows
/// building a webview from there waits on that same thread, so the new window never leaves
/// `about:blank`.
#[tauri::command]
pub async fn open_window<R: Runtime>(app: AppHandle<R>, kind: WindowKind) -> Result<(), String> {
    open_or_focus(&app, kind)
        .map(drop)
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_main_window_opens_larger_than_its_minimum() {
        let spec = spec_for(WindowKind::Main);
        let (min_width, min_height) = spec.min_size.expect("the main window has a minimum");
        assert!(spec.size.0 >= min_width && spec.size.1 >= min_height);
        assert_eq!(spec.size, (1440.0, 900.0));
        assert_eq!(spec.min_size, Some((1024.0, 680.0)));
    }

    #[test]
    fn about_is_a_fixed_window_that_stays_off_the_taskbar() {
        let spec = spec_for(WindowKind::About);
        assert_eq!(spec.size, (560.0, 760.0));
        assert!(!spec.resizable && spec.skip_taskbar);
    }

    #[test]
    fn book_tools_belong_to_the_main_window() {
        let spec = spec_for(WindowKind::BookTools);
        assert!(spec.child_of_main);
        assert_eq!(spec.size, (1180.0, 760.0));
        assert_eq!(spec.min_size, Some((900.0, 600.0)));
    }

    #[test]
    fn the_new_book_wizard_is_a_fixed_child_of_the_main_window() {
        let spec = spec_for(WindowKind::WizardNew);
        assert_eq!(spec.size, (880.0, 640.0));
        assert!(!spec.resizable && spec.skip_taskbar && spec.child_of_main);
    }

    #[test]
    fn every_kind_opens_a_window_with_its_own_label_and_reopening_makes_no_second() {
        let app = tauri::test::mock_app();
        // Book tools come before the main window: the mock runtime cannot parent a window to
        // another, which a real Book tools window does when the main window is open.
        for kind in [
            WindowKind::BookTools,
            WindowKind::WizardNew,
            WindowKind::About,
            WindowKind::Preferences,
            WindowKind::Main,
        ] {
            tauri::async_runtime::block_on(open_window(app.handle().clone(), kind)).expect("opens");
            tauri::async_runtime::block_on(open_window(app.handle().clone(), kind))
                .expect("focuses");
            assert!(app.get_webview_window(kind.label()).is_some());
        }
        assert_eq!(app.webview_windows().len(), 5);
    }

    #[test]
    fn a_window_is_built_with_the_label_of_its_kind() {
        let app = tauri::test::mock_app();
        let window = build_window(app.handle(), WindowKind::Main).expect("builds");
        assert_eq!(window.label(), "main");
        let again = open_or_focus(app.handle(), WindowKind::Main).expect("focuses");
        assert_eq!(again.label(), "main");
        assert_eq!(app.webview_windows().len(), 1);
    }
}
