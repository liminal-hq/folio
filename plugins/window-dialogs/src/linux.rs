// Linux modality: a GTK modal transient, with Wayland compositors placing the dialog themselves
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT
//
// The dialog is built with its opener as parent, which is `gtk_window_set_transient_for` and, on
// Wayland, `xdg_toplevel.set_parent`. `gtk_window_set_modal` then makes GTK drop pointer and
// keyboard input for the app's other windows while the dialog is open. GNOME attaches the dialog
// to its parent; KWin and wlroots treat it as an ordinary transient (GTK 3 has no
// `xdg_dialog_v1`), which is why the page also shows its own blocker.

use gtk::prelude::GtkWindowExt;
use std::sync::mpsc;
use std::time::Duration;
use tauri::{Runtime, WebviewWindow};

/// Makes `dialog` a GTK modal. True when GTK took it.
pub fn block<R: Runtime>(dialog: &WebviewWindow<R>, _opener: &WebviewWindow<R>) -> bool {
    let (tx, rx) = mpsc::channel();
    let target = dialog.clone();
    let queued = dialog.run_on_main_thread(move || {
        let taken = target
            .gtk_window()
            .map(|window| window.set_modal(true))
            .is_ok();
        let _ = tx.send(taken);
    });
    queued.is_ok() && rx.recv_timeout(Duration::from_secs(2)).unwrap_or(false)
}

/// GTK lifts the block when the modal closes; nothing to undo.
pub fn unblock<R: Runtime>(_opener: &WebviewWindow<R>) {}

/// Wayland compositors place windows; asking for a position is ignored.
pub fn places_windows() -> bool {
    std::env::var_os("WAYLAND_DISPLAY").is_none()
}
