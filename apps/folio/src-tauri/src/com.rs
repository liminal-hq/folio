// Puts the main thread in a single-threaded COM apartment before any plugin can choose another
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT
//
// The window library turns on native drag and drop with `OleInitialize`, which needs a
// single-threaded apartment. The first code to touch COM on a thread decides its apartment, and the
// system appearance plugin reads WinRT settings on the main thread in a multi-threaded one. If it
// goes first, window creation panics with `RPC_E_CHANGED_MODE`. Joining the single-threaded
// apartment here, before the plugins are set up, makes the order not matter: the plugin's own
// request then fails harmlessly and leaves this apartment in place.

/// Initialises OLE (and so COM) for the calling thread. Call once, first thing on the main thread.
#[cfg(windows)]
pub fn init_main_thread() {
    // SAFETY: initialises COM for the calling thread only; it is called once, before any other
    // COM user, and the thread keeps the apartment for the life of the process.
    if let Err(error) = unsafe { windows::Win32::System::Ole::OleInitialize(None) } {
        log::warn!("could not set up the main thread's COM apartment: {error}");
    }
}

/// No COM on this platform.
#[cfg(not(windows))]
pub fn init_main_thread() {}
