// Lends the open books to AI agents: serves Folio's MCP tools on a local socket
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT
//
// An agent connects through `folio-mcp`, which relays to this socket, and works on the books the
// person has open: the same sessions, the same undo history and the same events, so the window
// follows the agent's changes as it follows its own. The socket is reachable only by the person
// (D-45). Setting `FOLIO_AGENTS=off` leaves it closed.

use std::io::ErrorKind;
use std::sync::Arc;

use folio_mcp::live::{self, Endpoint};
use tauri::{AppHandle, Manager, Runtime};
use tauri_plugin_folio_books::Books;

/// Whether the person has asked for the socket to stay closed.
fn disabled() -> bool {
    std::env::var("FOLIO_AGENTS")
        .is_ok_and(|v| matches!(v.to_lowercase().as_str(), "off" | "0" | "false" | "no"))
}

/// Starts serving the tools. Call once the books plugin is set up.
pub fn serve<R: Runtime>(app: &AppHandle<R>) {
    if disabled() {
        log::info!("agents are switched off, so the agent socket stays closed");
        return;
    }
    let Some(books) = app.try_state::<Arc<Books>>() else {
        return;
    };
    let books = Arc::clone(&books);
    tauri::async_runtime::spawn(async move {
        let endpoint = Endpoint::for_user();
        log::info!("agents can connect at {}", endpoint.path().display());
        match live::serve(books, &endpoint).await {
            Err(e) if e.kind() == ErrorKind::AddrInUse => {
                log::info!("another copy of Folio is serving agents: {e}");
            }
            Err(e) => log::warn!("could not listen for agents: {e}"),
            Ok(()) => {}
        }
    });
}

/// Takes the socket away when the app quits, so the next agent starts its own books rather than
/// finding no one at home.
pub fn stop() {
    if cfg!(unix) && !disabled() {
        let _ = std::fs::remove_file(Endpoint::for_user().path());
    }
}
