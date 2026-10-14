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
        },
        // The other kinds arrive with the child windows.
        WindowKind::About | WindowKind::Preferences | WindowKind::BookTools => WindowSpec {
            title: APP_TITLE,
            size: (800.0, 600.0),
            min_size: None,
            resizable: true,
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
            .decorations(false)
            .transparent(true)
            .shadow(true)
            .center();
    if let Some((width, height)) = spec.min_size {
        builder = builder.min_inner_size(width, height);
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
    fn a_window_is_built_with_the_label_of_its_kind() {
        let app = tauri::test::mock_app();
        let window = build_window(app.handle(), WindowKind::Main).expect("builds");
        assert_eq!(window.label(), "main");
        let again = open_or_focus(app.handle(), WindowKind::Main).expect("focuses");
        assert_eq!(again.label(), "main");
        assert_eq!(app.webview_windows().len(), 1);
    }
}
