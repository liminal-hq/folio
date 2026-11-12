// Windows: the dialog is owned by its opener, which stays enabled
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT
//
// `parent` makes the opener the dialog's owner window, so the dialog stays above it, is minimised
// and closed with it, and is not in the taskbar.

/// Windows lets an app place its windows.
pub fn places_windows() -> bool {
    true
}
