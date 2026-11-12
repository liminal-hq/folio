// Linux: the dialog is a transient of its opener, and Wayland compositors place it themselves
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT
//
// The dialog is built with its opener as parent, which is `gtk_window_set_transient_for` and, on
// Wayland, `xdg_toplevel.set_parent`: it stays above its opener and goes with it when the opener is
// minimised or closed. It is deliberately not a GTK modal and the opener is not disabled. GNOME's
// Mutter attaches a modal dialog to its parent, so dragging the dialog's title bar also dragged
// the main window, and blocking the opener made the app feel locked up.

/// Wayland compositors place windows; asking for a position is ignored.
pub fn places_windows() -> bool {
    std::env::var_os("WAYLAND_DISPLAY").is_none()
}
