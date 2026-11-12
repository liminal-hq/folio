// Linux modality: the opener is made insensitive, the dialog is its transient, and nothing is a GTK modal
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT
//
// The dialog is built with its opener as parent, which is `gtk_window_set_transient_for` and, on
// Wayland, `xdg_toplevel.set_parent`: the dialog stays above its opener and goes with it when it
// is minimised or closed. It is deliberately not made a GTK modal. GNOME's Mutter *attaches* a
// modal dialog to its parent (the "attach modal dialogs" behaviour), so dragging the dialog's
// title bar dragged the main window with it. The opener is instead made insensitive with
// `gtk_widget_set_sensitive(false)`, which drops its pointer and keyboard input without the
// compositor treating the dialog as modal. The page also shows its own blocker (and dims), which
// is what covers a compositor that ignores the transient.
// A modeless dialog (`DialogSpec::modal == false`) never reaches this module: it is only a
// transient child.

use gtk::prelude::WidgetExt;
use std::sync::mpsc;
use std::time::Duration;
use tauri::{Runtime, WebviewWindow};

fn set_sensitive<R: Runtime>(window: &WebviewWindow<R>, sensitive: bool) -> bool {
    let (tx, rx) = mpsc::channel();
    let target = window.clone();
    let queued = window.run_on_main_thread(move || {
        let done = target
            .gtk_window()
            .map(|window| window.set_sensitive(sensitive))
            .is_ok();
        let _ = tx.send(done);
    });
    queued.is_ok() && rx.recv_timeout(Duration::from_secs(2)).unwrap_or(false)
}

/// Blocks `opener`'s input. True when GTK took it.
pub fn block<R: Runtime>(_dialog: &WebviewWindow<R>, opener: &WebviewWindow<R>) -> bool {
    set_sensitive(opener, false)
}

/// Lets `opener` take input again.
pub fn unblock<R: Runtime>(opener: &WebviewWindow<R>) {
    let target = opener.clone();
    let _ = opener.run_on_main_thread(move || {
        if let Ok(window) = target.gtk_window() {
            window.set_sensitive(true);
        }
    });
}

/// Wayland compositors place windows; asking for a position is ignored.
pub fn places_windows() -> bool {
    std::env::var_os("WAYLAND_DISPLAY").is_none()
}
