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
use std::path::{Path, PathBuf};
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

/// The `folio-mcp` program that goes with the app: beside it, as the bundle puts it. An AppImage
/// runs from a mount that is gone once Folio quits, so an agent program is given a copy kept in the
/// user's data folder instead.
fn companion() -> Option<PathBuf> {
    let name = if cfg!(windows) {
        "folio-mcp.exe"
    } else {
        "folio-mcp"
    };
    let beside = std::env::current_exe().ok()?.parent()?.join(name);
    if !beside.is_file() {
        return None;
    }
    if std::env::var_os("APPIMAGE").is_none() {
        return Some(beside);
    }
    let kept = Places::here()
        .map(|at| {
            kept_dir(
                &at.home,
                std::env::var_os("XDG_DATA_HOME").map(PathBuf::from),
            )
        })
        .and_then(|dir| keep_copy(&beside, &dir).ok());
    Some(kept.unwrap_or_else(|| {
        log::warn!("could not keep a copy of folio-mcp outside the AppImage");
        beside
    }))
}

/// Where an AppImage keeps its copy of `folio-mcp`: the data folder, under the app's identifier.
fn kept_dir(home: &Path, xdg_data: Option<PathBuf>) -> PathBuf {
    xdg_data
        .filter(|d| d.is_absolute())
        .unwrap_or_else(|| home.join(".local/share"))
        .join("ca.liminalhq.folio/bin")
}

/// Copies `source` into `dir` as `folio-mcp`, unless the copy there is already the same, and says
/// where it is. The copy is written beside its place and renamed over it, so an agent program that
/// is running it never sees half a file.
fn keep_copy(source: &Path, dir: &Path) -> std::io::Result<PathBuf> {
    let target = dir.join("folio-mcp");
    let bytes = std::fs::read(source)?;
    if std::fs::read(&target).is_ok_and(|kept| kept == bytes) {
        return Ok(target);
    }
    std::fs::create_dir_all(dir)?;
    let partial = dir.join(".folio-mcp.partial");
    std::fs::write(&partial, &bytes)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&partial, std::fs::Permissions::from_mode(0o755))?;
    }
    std::fs::rename(&partial, &target)?;
    Ok(target)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_copy_goes_in_the_data_folder_under_the_app_identifier() {
        let home = Path::new("/home/mom");
        assert_eq!(
            kept_dir(home, None),
            Path::new("/home/mom/.local/share/ca.liminalhq.folio/bin")
        );
        assert_eq!(
            kept_dir(home, Some("/data".into())),
            Path::new("/data/ca.liminalhq.folio/bin")
        );
        // A relative XDG_DATA_HOME is ignored, as the specification says.
        assert_eq!(
            kept_dir(home, Some("data".into())),
            Path::new("/home/mom/.local/share/ca.liminalhq.folio/bin")
        );
    }

    #[test]
    fn a_copy_is_kept_once_and_replaced_when_the_program_changes() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("mounted-folio-mcp");
        let into = dir.path().join("data/bin");
        std::fs::write(&source, b"one").unwrap();
        let kept = keep_copy(&source, &into).unwrap();
        assert_eq!(std::fs::read(&kept).unwrap(), b"one");
        assert_eq!(keep_copy(&source, &into).unwrap(), kept);

        std::fs::write(&source, b"two").unwrap();
        keep_copy(&source, &into).unwrap();
        assert_eq!(std::fs::read(&kept).unwrap(), b"two");
        // Nothing is left behind but the program.
        assert_eq!(std::fs::read_dir(&into).unwrap().count(), 1);
    }

    #[cfg(unix)]
    #[test]
    fn the_kept_copy_can_be_run() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("folio-mcp");
        std::fs::write(&source, b"program").unwrap();
        let kept = keep_copy(&source, &dir.path().join("bin")).unwrap();
        assert_eq!(
            std::fs::metadata(kept).unwrap().permissions().mode() & 0o777,
            0o755
        );
    }
}
