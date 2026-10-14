// Composition root: registers the Tauri plugins and starts the app
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

mod app_info;
mod windows;

use folio_protocol::WindowKind;

/// The level floor applied to every log line — native Rust and forwarded webview `console.*`
/// calls alike. Verbose in a debug build, `Info` and up in a release one.
fn log_level() -> log::LevelFilter {
    if cfg!(debug_assertions) {
        log::LevelFilter::Trace
    } else {
        log::LevelFilter::Info
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // `tauri-plugin-log`'s own `plugin:log|log` command is what `src/services/logger.ts`
        // forwards the webview's `console.*` calls into, so forwarded messages land in the same
        // stdout stream and log file as native `log::info!()` calls.
        .plugin(
            tauri_plugin_log::Builder::default()
                .level(log_level())
                .format(|out, message, record| {
                    out.finish(format_args!(
                        "[{}][{}][{}] {}",
                        chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f %:z"),
                        record.level(),
                        record.target(),
                        message
                    ))
                })
                .build(),
        )
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_system_appearance::init())
        .plugin(tauri_plugin_window_manager::init())
        .invoke_handler(tauri::generate_handler![app_info::app_info])
        .setup(|app| {
            windows::open_or_focus(app.handle(), WindowKind::Main)?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running the Folio application");
}
