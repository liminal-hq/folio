// Platforms where the opener cannot be blocked: the page's own blocker is the fallback
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use tauri::{Runtime, WebviewWindow};

pub fn block<R: Runtime>(_dialog: &WebviewWindow<R>, _opener: &WebviewWindow<R>) -> bool {
    false
}

pub fn unblock<R: Runtime>(_opener: &WebviewWindow<R>) {}

pub fn places_windows() -> bool {
    true
}
