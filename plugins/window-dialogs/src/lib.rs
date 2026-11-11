// Registers the window-dialogs plugin: dialogs that are windows of their own, blocking their opener
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

mod commands;
mod error;
pub mod models;
pub mod registry;
mod session;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
use linux as platform;

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
use windows as platform;

#[cfg(not(any(target_os = "linux", target_os = "windows")))]
mod unsupported;
#[cfg(not(any(target_os = "linux", target_os = "windows")))]
use unsupported as platform;

use tauri::{
    plugin::{Builder, TauriPlugin},
    AppHandle, Manager, Runtime,
};

pub use error::Error;
pub use session::{ended, open, post, Dialogs, OnResized, Options};

#[cfg(test)]
mod tests;

/// Registers the plugin.
pub fn init<R: Runtime>(options: Options) -> TauriPlugin<R> {
    init_with(move |_| options)
}

/// Registers the plugin with options made once the app exists, for options that need its handle.
pub fn init_with<R: Runtime>(
    options: impl FnOnce(&AppHandle<R>) -> Options + Send + 'static,
) -> TauriPlugin<R> {
    Builder::new("window-dialogs")
        .invoke_handler(tauri::generate_handler![
            commands::open_dialog,
            commands::dialog_request,
            commands::dialog_shown,
            commands::finish_dialog,
            commands::post_to_opener,
            commands::focus_dialog,
        ])
        .setup(move |app, _api| {
            let options = options(app);
            app.manage(Dialogs::new(options));
            Ok(())
        })
        .build()
}
