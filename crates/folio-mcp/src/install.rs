// Adds `folio-mcp` to the configuration of the agent programs that read an MCP server list
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

// Each program keeps its list in its own file, in one of three shapes: a JSON object with
// `mcpServers` (most of them), a JSON object with `servers` (VS Code), or TOML (Codex). Claude
// Code keeps its own and is told through its command line. Installing reads the file as it is,
// changes only the `folio` entry and keeps a copy of what was there. The paths are those each
// program documents ⚠ and are worth checking against its current documentation before a release.

use std::path::{Path, PathBuf};

use serde_json::{json, Map, Value};

/// The name the server is listed under.
pub const NAME: &str = "folio";

/// An agent program Folio can be added to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "cli", derive(clap::ValueEnum))]
pub enum Client {
    ClaudeCode,
    ClaudeDesktop,
    Cursor,
    Vscode,
    Windsurf,
    Codex,
    Gemini,
}

impl Client {
    pub const ALL: [Client; 7] = [
        Client::ClaudeCode,
        Client::ClaudeDesktop,
        Client::Cursor,
        Client::Vscode,
        Client::Windsurf,
        Client::Codex,
        Client::Gemini,
    ];

    /// The name a person types.
    pub fn name(self) -> &'static str {
        match self {
            Client::ClaudeCode => "claude-code",
            Client::ClaudeDesktop => "claude-desktop",
            Client::Cursor => "cursor",
            Client::Vscode => "vscode",
            Client::Windsurf => "windsurf",
            Client::Codex => "codex",
            Client::Gemini => "gemini",
        }
    }
}

impl Client {
    /// The name the program goes by.
    pub fn label(self) -> &'static str {
        match self {
            Client::ClaudeCode => "Claude Code",
            Client::ClaudeDesktop => "Claude Desktop",
            Client::Cursor => "Cursor",
            Client::Vscode => "VS Code",
            Client::Windsurf => "Windsurf",
            Client::Codex => "Codex",
            Client::Gemini => "Gemini CLI",
        }
    }
}

/// Where the person's folders are, so the paths can be worked out (and tested) without the
/// environment.
#[derive(Debug, Clone)]
pub struct Places {
    pub home: PathBuf,
    /// `%APPDATA%` on Windows.
    pub app_data: Option<PathBuf>,
    pub os: Os,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Os {
    Linux,
    Mac,
    Windows,
}

impl Places {
    /// The places on this computer.
    pub fn here() -> Option<Self> {
        let home = std::env::var_os("HOME")
            .or_else(|| std::env::var_os("USERPROFILE"))
            .map(PathBuf::from)?;
        Some(Self {
            home,
            app_data: std::env::var_os("APPDATA").map(PathBuf::from),
            os: if cfg!(windows) {
                Os::Windows
            } else if cfg!(target_os = "macos") {
                Os::Mac
            } else {
                Os::Linux
            },
        })
    }

    /// The folder programs keep their settings in: `~/.config`, `~/Library/Application Support`
    /// or `%APPDATA%`.
    fn config(&self) -> PathBuf {
        match self.os {
            Os::Linux => self.home.join(".config"),
            Os::Mac => self.home.join("Library/Application Support"),
            Os::Windows => self
                .app_data
                .clone()
                .unwrap_or_else(|| self.home.join("AppData/Roaming")),
        }
    }
}

/// How a program is told about the server.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Target {
    /// By running a command.
    Command(Vec<String>),
    /// By a file that holds the list.
    File(PathBuf),
}

/// Where `client` keeps its list.
pub fn target(client: Client, at: &Places, exe: &Path) -> Target {
    let file = |path: PathBuf| Target::File(path);
    match client {
        Client::ClaudeCode => Target::Command(
            ["claude", "mcp", "add", "--scope", "user", NAME, "--"]
                .into_iter()
                .map(str::to_owned)
                .chain([exe.to_string_lossy().into_owned()])
                .collect(),
        ),
        Client::ClaudeDesktop => file(at.config().join("Claude/claude_desktop_config.json")),
        Client::Cursor => file(at.home.join(".cursor/mcp.json")),
        Client::Vscode => file(at.config().join("Code/User/mcp.json")),
        Client::Windsurf => file(at.home.join(".codeium/windsurf/mcp_config.json")),
        Client::Codex => file(at.home.join(".codex/config.toml")),
        Client::Gemini => file(at.home.join(".gemini/settings.json")),
    }
}

/// What can go wrong adding the server.
#[derive(Debug, thiserror::Error)]
pub enum InstallError {
    #[error("{0} is not valid, so it was left alone: {1}")]
    Invalid(String, String),
    #[error("{0}: {1}")]
    Io(String, std::io::Error),
}

/// The text of `existing` (empty if there is no file) with the server added under `client`'s key,
/// everything else as it was.
pub fn with_server(
    client: Client,
    existing: &str,
    exe: &Path,
    shown_as: &str,
) -> Result<String, InstallError> {
    let invalid = |e: String| InstallError::Invalid(shown_as.to_owned(), e);
    let command = exe.to_string_lossy().into_owned();
    if client == Client::Codex {
        let mut doc: toml_edit::DocumentMut = existing
            .parse()
            .map_err(|e: toml_edit::TomlError| invalid(e.to_string()))?;
        let servers = doc
            .entry("mcp_servers")
            .or_insert(toml_edit::table())
            .as_table_mut()
            .ok_or_else(|| invalid("`mcp_servers` is not a table".into()))?;
        let mut entry = toml_edit::Table::new();
        entry["command"] = toml_edit::value(command);
        entry["args"] = toml_edit::value(toml_edit::Array::new());
        servers.insert(NAME, toml_edit::Item::Table(entry));
        return Ok(doc.to_string());
    }
    let mut root: Value = if existing.trim().is_empty() {
        Value::Object(Map::new())
    } else {
        serde_json::from_str(existing).map_err(|e| invalid(e.to_string()))?
    };
    let (key, entry) = if client == Client::Vscode {
        (
            "servers",
            json!({ "type": "stdio", "command": command, "args": [] }),
        )
    } else {
        ("mcpServers", json!({ "command": command, "args": [] }))
    };
    let object = root
        .as_object_mut()
        .ok_or_else(|| invalid("it is not a JSON object".into()))?;
    let servers = object
        .entry(key)
        .or_insert_with(|| Value::Object(Map::new()))
        .as_object_mut()
        .ok_or_else(|| invalid(format!("`{key}` is not an object")))?;
    servers.insert(NAME.to_owned(), entry);
    let mut text = serde_json::to_string_pretty(&root).map_err(|e| invalid(e.to_string()))?;
    text.push('\n');
    Ok(text)
}

/// Whether `existing` lists the server.
pub fn lists_server(client: Client, existing: &str) -> bool {
    if client == Client::Codex {
        return existing
            .parse::<toml_edit::DocumentMut>()
            .ok()
            .and_then(|d| d.get("mcp_servers")?.get(NAME).map(|_| ()))
            .is_some();
    }
    let key = if client == Client::Vscode {
        "servers"
    } else {
        "mcpServers"
    };
    serde_json::from_str::<Value>(existing)
        .ok()
        .is_some_and(|v| v.get(key).and_then(|s| s.get(NAME)).is_some())
}

/// Adds the server to the file at `path`, keeping a copy of the file as it was beside it.
pub fn write_server(client: Client, path: &Path, exe: &Path) -> Result<(), InstallError> {
    let shown = path.display().to_string();
    let io = |e| InstallError::Io(shown.clone(), e);
    let existing = match std::fs::read_to_string(path) {
        Ok(text) => Some(text),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
        Err(e) => return Err(io(e)),
    };
    let text = with_server(client, existing.as_deref().unwrap_or(""), exe, &shown)?;
    if let Some(before) = &existing {
        let mut backup = path.as_os_str().to_owned();
        backup.push(".folio-backup");
        std::fs::write(backup, before).map_err(io)?;
    }
    if let Some(folder) = path.parent() {
        std::fs::create_dir_all(folder).map_err(io)?;
    }
    std::fs::write(path, text).map_err(io)
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXE: &str = "/opt/folio/folio-mcp";

    fn exe() -> &'static Path {
        Path::new(EXE)
    }

    #[test]
    fn a_json_list_keeps_everything_but_folio() {
        let before = r#"{ "theme": "dark", "mcpServers": { "other": { "command": "x" } } }"#;
        let after = with_server(Client::Cursor, before, exe(), "mcp.json").unwrap();
        let v: Value = serde_json::from_str(&after).unwrap();
        assert_eq!(v["theme"], "dark");
        assert_eq!(v["mcpServers"]["other"]["command"], "x");
        assert_eq!(v["mcpServers"]["folio"]["command"], EXE);
        assert!(lists_server(Client::Cursor, &after));
        assert!(!lists_server(Client::Cursor, before));
    }

    #[test]
    fn a_new_file_gets_the_list() {
        let after = with_server(Client::ClaudeDesktop, "", exe(), "c.json").unwrap();
        let v: Value = serde_json::from_str(&after).unwrap();
        assert_eq!(v["mcpServers"]["folio"]["args"], json!([]));
    }

    #[test]
    fn vscode_lists_servers_with_a_type() {
        let after = with_server(Client::Vscode, "{}", exe(), "mcp.json").unwrap();
        let v: Value = serde_json::from_str(&after).unwrap();
        assert_eq!(v["servers"]["folio"]["type"], "stdio");
        assert!(lists_server(Client::Vscode, &after));
    }

    #[test]
    fn a_file_that_is_not_json_is_left_alone() {
        let refused = with_server(Client::Cursor, "{ nope", exe(), "mcp.json");
        assert!(matches!(refused, Err(InstallError::Invalid(..))));
        let wrong = with_server(Client::Cursor, "[1]", exe(), "mcp.json");
        assert!(matches!(wrong, Err(InstallError::Invalid(..))));
    }

    #[test]
    fn codex_keeps_its_other_settings_and_comments() {
        let before = "# my settings\nmodel = \"x\"\n\n[mcp_servers.other]\ncommand = \"y\"\n";
        let after = with_server(Client::Codex, before, exe(), "config.toml").unwrap();
        assert!(after.contains("# my settings"));
        assert!(after.contains("[mcp_servers.other]"));
        assert!(after.contains("[mcp_servers.folio]"));
        assert!(after.contains(EXE));
        assert!(lists_server(Client::Codex, &after));
    }

    #[test]
    fn each_program_has_its_own_place() {
        let at = Places {
            home: PathBuf::from("/home/p"),
            app_data: None,
            os: Os::Linux,
        };
        let path = |c| match target(c, &at, exe()) {
            Target::File(p) => p,
            Target::Command(_) => PathBuf::new(),
        };
        assert_eq!(
            path(Client::ClaudeDesktop),
            Path::new("/home/p/.config/Claude/claude_desktop_config.json")
        );
        assert_eq!(path(Client::Cursor), Path::new("/home/p/.cursor/mcp.json"));
        assert_eq!(path(Client::Codex), Path::new("/home/p/.codex/config.toml"));
        let Target::Command(words) = target(Client::ClaudeCode, &at, exe()) else {
            panic!("Claude Code is told by command");
        };
        assert_eq!(words[..3], ["claude", "mcp", "add"]);
        assert_eq!(words.last().unwrap(), EXE);

        let windows = Places {
            home: PathBuf::from("C:/Users/p"),
            app_data: Some(PathBuf::from("C:/Users/p/AppData/Roaming")),
            os: Os::Windows,
        };
        let Target::File(desktop) = target(Client::ClaudeDesktop, &windows, exe()) else {
            panic!("a file");
        };
        assert!(desktop.starts_with("C:/Users/p/AppData/Roaming"));
    }

    #[test]
    fn writing_keeps_a_copy_of_what_was_there() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("mcp.json");
        std::fs::write(&file, r#"{"mcpServers":{}}"#).unwrap();
        write_server(Client::Cursor, &file, exe()).unwrap();
        assert!(lists_server(
            Client::Cursor,
            &std::fs::read_to_string(&file).unwrap()
        ));
        let backup = dir.path().join("mcp.json.folio-backup");
        assert_eq!(
            std::fs::read_to_string(backup).unwrap(),
            r#"{"mcpServers":{}}"#
        );

        // A folder that is not there yet is made.
        let nested = dir.path().join("a/b/mcp.json");
        write_server(Client::Cursor, &nested, exe()).unwrap();
        assert!(nested.exists());
    }
}
