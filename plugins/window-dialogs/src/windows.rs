// Windows modality: the dialog is owned by its opener, and the opener is disabled while it is open
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT
//
// `parent` makes the opener the dialog's owner window, so the dialog stays above it. Disabling
// the opener with `EnableWindow` is what blocks it. It must be enabled and focused *before* the
// dialog is destroyed, or Windows activates another application's window.

use tauri::{Runtime, WebviewWindow};

/// Disables the opener. True when it was.
pub fn block<R: Runtime>(_dialog: &WebviewWindow<R>, opener: &WebviewWindow<R>) -> bool {
    opener.set_enabled(false).is_ok()
}

/// Enables the opener again.
pub fn unblock<R: Runtime>(opener: &WebviewWindow<R>) {
    let _ = opener.set_enabled(true);
}

/// Windows lets an app place its windows.
pub fn places_windows() -> bool {
    true
}
