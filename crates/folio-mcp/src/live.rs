// The live link: the app serves the agent's tools on a local socket, and the server relays to it
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

// The app serves the same MCP server over its own open books on a socket only the person can
// reach (a Unix socket in their runtime folder, a named pipe on Windows). `folio-mcp` finds that
// socket and relays standard input and output to it, so an agent edits the book the person has
// open, through the one writer, and the window follows as it does for any change.

use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use folio_engine::Books;
use rmcp::ServiceExt;
use tokio::io::{AsyncRead, AsyncWrite};

use crate::FolioServer;

/// Names the socket or pipe the app listens on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Endpoint(PathBuf);

/// Set to use another endpoint than the person's own, which two running copies of the app, or a
/// test, need.
pub const ENDPOINT_VARIABLE: &str = "FOLIO_AGENT_ENDPOINT";

impl Endpoint {
    /// The endpoint at `path`.
    pub fn at(path: impl Into<PathBuf>) -> Self {
        Self(path.into())
    }

    /// The person's own endpoint: `FOLIO_AGENT_ENDPOINT` if it is set, otherwise a socket in their
    /// runtime folder, or a pipe named for them on Windows.
    pub fn for_user() -> Self {
        if let Some(path) = std::env::var_os(ENDPOINT_VARIABLE) {
            return Self(PathBuf::from(path));
        }
        let user = std::env::var("USER")
            .or_else(|_| std::env::var("USERNAME"))
            .unwrap_or_else(|_| "folio".into());
        if cfg!(windows) {
            return Self(PathBuf::from(format!(r"\\.\pipe\folio-agent-{user}")));
        }
        let folder = std::env::var_os("XDG_RUNTIME_DIR")
            .map(|d| PathBuf::from(d).join("folio"))
            .unwrap_or_else(|| std::env::temp_dir().join(format!("folio-{user}")));
        Self(folder.join("agent.sock"))
    }

    pub fn path(&self) -> &Path {
        &self.0
    }
}

/// Serves Folio's tools to every agent that connects to `endpoint`, over `books`, until the
/// process ends. Fails at once if another copy of the app is already listening there.
pub async fn serve(books: Arc<Books>, endpoint: &Endpoint) -> io::Result<()> {
    imp::serve(books, endpoint).await
}

/// Connects to the app listening at `endpoint`, if there is one.
pub async fn connect(endpoint: &Endpoint) -> Option<impl AsyncRead + AsyncWrite + Unpin + Send> {
    imp::connect(endpoint).await.ok()
}

/// Passes bytes between the agent's standard input and output and the app, until either closes.
pub async fn relay(
    app: impl AsyncRead + AsyncWrite + Unpin + Send + 'static,
    input: impl AsyncRead + Unpin + Send + 'static,
    output: impl AsyncWrite + Unpin + Send + 'static,
) -> io::Result<()> {
    let (mut from_app, mut to_app) = tokio::io::split(app);
    let (mut input, mut output) = (input, output);
    let up = tokio::spawn(async move { tokio::io::copy(&mut input, &mut to_app).await });
    let down = tokio::spawn(async move { tokio::io::copy(&mut from_app, &mut output).await });
    // Either side closing ends the session.
    tokio::select! {
        r = up => r.map_err(io::Error::other)?.map(drop),
        r = down => r.map_err(io::Error::other)?.map(drop),
    }
}

/// Runs one agent's session over `stream` until it ends.
async fn session(books: Arc<Books>, stream: impl AsyncRead + AsyncWrite + Unpin + Send + 'static) {
    match FolioServer::new(books).serve(stream).await {
        Ok(running) => {
            if let Err(e) = running.waiting().await {
                log::debug!("an agent's session ended: {e}");
            }
        }
        Err(e) => log::debug!("an agent could not connect: {e}"),
    }
}

#[cfg(unix)]
mod imp {
    use std::os::unix::fs::{DirBuilderExt, PermissionsExt};

    use tokio::net::{UnixListener, UnixStream};

    use super::*;

    pub async fn connect(endpoint: &Endpoint) -> io::Result<UnixStream> {
        UnixStream::connect(endpoint.path()).await
    }

    pub async fn serve(books: Arc<Books>, endpoint: &Endpoint) -> io::Result<()> {
        let path = endpoint.path();
        if let Some(folder) = path.parent() {
            std::fs::DirBuilder::new()
                .recursive(true)
                .mode(0o700)
                .create(folder)?;
        }
        if path.exists() {
            // A socket left by a copy that died is taken over; one that answers is in use.
            if UnixStream::connect(path).await.is_ok() {
                return Err(io::Error::new(
                    io::ErrorKind::AddrInUse,
                    "another copy of Folio is already listening for agents",
                ));
            }
            std::fs::remove_file(path)?;
        }
        let listener = UnixListener::bind(path)?;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))?;
        loop {
            let (stream, _) = listener.accept().await?;
            tokio::spawn(session(books.clone(), stream));
        }
    }
}

#[cfg(windows)]
mod imp {
    use tokio::net::windows::named_pipe::{ClientOptions, ServerOptions};

    use super::*;

    pub async fn connect(endpoint: &Endpoint) -> io::Result<impl AsyncRead + AsyncWrite + Unpin> {
        ClientOptions::new().open(endpoint.path())
    }

    pub async fn serve(books: Arc<Books>, endpoint: &Endpoint) -> io::Result<()> {
        // The first instance fails if another copy of the app already made the pipe, so no
        // other program can stand in for it. A pipe is closed to other computers by default.
        let mut server = ServerOptions::new()
            .first_pipe_instance(true)
            .reject_remote_clients(true)
            .create(endpoint.path())?;
        loop {
            server.connect().await?;
            let connected = server;
            server = ServerOptions::new()
                .reject_remote_clients(true)
                .create(endpoint.path())?;
            tokio::spawn(session(books.clone(), connected));
        }
    }
}
