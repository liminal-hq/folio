// Composition root: registers the Tauri plugins and starts the app
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

mod agent;
mod app_info;
mod com;
mod recents;
mod settings;
mod storage;
mod updates;
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
    com::init_main_thread();

    #[allow(unused_mut)]
    let mut builder = tauri::Builder::default()
        // `tauri-plugin-log`'s own `plugin:log|log` command is what `src/services/logger.ts`
        // forwards the webview's `console.*` calls into, so forwarded messages land in the same
        // stdout stream and log file as native `log::info!()` calls.
        .plugin(
            tauri_plugin_log::Builder::default()
                .level(log_level())
                // The debug-only MCP bridge's websocket internals are extremely chatty at `Trace`
                // (every handshake read and write poll) and drown out everything else.
                .level_for("tungstenite", log::LevelFilter::Warn)
                .level_for("tokio_tungstenite", log::LevelFilter::Warn)
                // The async runtime polls constantly; at `Trace` it buries everything else.
                .level_for("mio", log::LevelFilter::Warn)
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
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_system_appearance::init())
        .plugin(tauri_plugin_window_manager::init())
        // After the store plugin it saves through.
        .plugin(tauri_plugin_folio_settings::init_with(settings::storage))
        .plugin(tauri_plugin_folio_books::init())
        .invoke_handler(tauri::generate_handler![
            agent::agent_install,
            agent::agent_status,
            app_info::app_info,
            updates::update_state,
            updates::update_check,
            updates::update_download,
            updates::update_install,
            windows::open_window,
            windows::open_book_tools,
            windows::book_tools_area
        ])
        .setup(|app| {
            tauri::Manager::manage(app, windows::ToolsArea::default());
            recents::wire(app.handle());
            agent::serve(app.handle());
            updates::watch(app.handle());
            windows::open_or_focus(app.handle(), WindowKind::Main)?;
            Ok(())
        });

    // Lets an agent drive and screenshot the running app during development. Not on Windows, where
    // the bridge does not yet compile against Tauri's `windows` crate (see `Cargo.toml`), and
    // never in a release build.
    #[cfg(all(debug_assertions, not(windows)))]
    {
        builder = builder.plugin(tauri_plugin_mcp_bridge::init());
    }

    builder
        .build(tauri::generate_context!())
        .expect("error while building the Folio application")
        .run(|_app, event| {
            if matches!(event, tauri::RunEvent::Exit) {
                agent::stop();
            }
        });
}
