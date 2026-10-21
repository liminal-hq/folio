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
use std::path::PathBuf;
use std::sync::Arc;

use folio_mcp::install::{self, Client, Places, Target};
use folio_mcp::live::{self, Endpoint};
use folio_protocol::{AgentProgram, AgentStatus};
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

/// The `folio-mcp` program that goes with the app: beside it, as the bundle puts it.
fn companion() -> Option<PathBuf> {
    let name = if cfg!(windows) {
        "folio-mcp.exe"
    } else {
        "folio-mcp"
    };
    let beside = std::env::current_exe().ok()?.parent()?.join(name);
    beside.is_file().then_some(beside)
}

/// What the Connect an agent window shows: whether agents can reach the books, and the programs
/// Folio can be added to.
#[tauri::command]
pub async fn agent_status() -> AgentStatus {
    let endpoint = Endpoint::for_user();
    AgentStatus {
        listening: !disabled() && live::connect(&endpoint).await.is_some(),
        endpoint: endpoint.path().display().to_string(),
        binary: companion().map(|p| p.display().to_string()),
        programs: Client::ALL
            .iter()
            .map(|c| AgentProgram {
                id: c.name().to_owned(),
                label: c.label().to_owned(),
            })
            .collect(),
    }
}

/// Adds Folio to an agent program's list of MCP servers, and says what was done.
#[tauri::command]
pub async fn agent_install(program: String) -> Result<String, String> {
    let client = Client::ALL
        .into_iter()
        .find(|c| c.name() == program)
        .ok_or_else(|| format!("Folio does not know how to add itself to {program}."))?;
    let exe = companion().ok_or("The folio-mcp program is not beside Folio.")?;
    let at = Places::here().ok_or("Folio cannot find your home folder.")?;
    match install::target(client, &at, &exe) {
        Target::File(path) => {
            install::write_server(client, &path, &exe).map_err(|e| e.to_string())?;
            Ok(format!(
                "Added Folio to {}. Restart {} to see it.",
                path.display(),
                client.label()
            ))
        }
        Target::Command(words) => {
            let output = std::process::Command::new(&words[0])
                .args(&words[1..])
                .output()
                .map_err(|e| format!("Could not run `{}`: {e}", words[0]))?;
            if output.status.success() {
                Ok(format!("Added Folio to {}.", client.label()))
            } else {
                Err(String::from_utf8_lossy(&output.stderr).trim().to_owned())
            }
        }
    }
}
