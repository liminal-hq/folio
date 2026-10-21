// folio-mcp: serves Folio's books to an AI agent over stdio
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::process::ExitCode;
use std::sync::Arc;

use clap::{Parser, Subcommand, ValueEnum};
use folio_engine::Books;
use folio_mcp::host::{app_data_dir, CliHost};
use folio_mcp::install::{self, Client, Places, Target};
use folio_mcp::live::{self, Endpoint};
use folio_mcp::FolioServer;
use rmcp::{transport::stdio, ServiceExt};

/// Which books the agent works on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum Mode {
    /// The books open in Folio if it is running, otherwise books of its own.
    Auto,
    /// Only the books open in Folio; fail if it is not running.
    Live,
    /// Only books of its own, never Folio's.
    Headless,
}

/// Serves Folio's books to an AI agent over the Model Context Protocol, on standard input and
/// output.
#[derive(Debug, Parser)]
#[command(version, about)]
struct Args {
    #[arg(long, value_enum, default_value_t = Mode::Auto)]
    mode: Mode,
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Adds Folio to an agent program's list of MCP servers.
    Install {
        /// The program to add Folio to.
        #[arg(value_enum)]
        client: Client,
        /// Show what would change instead of changing it.
        #[arg(long)]
        print: bool,
    },
    /// Checks that Folio can be reached: this program, the running app, and each agent program's
    /// list.
    Doctor,
}

#[tokio::main]
async fn main() -> ExitCode {
    let args = Args::parse();
    let result = match args.command {
        Some(Command::Install { client, print }) => install_into(client, print),
        Some(Command::Doctor) => doctor().await,
        None => run(args.mode).await,
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("folio-mcp: {e}");
            ExitCode::FAILURE
        }
    }
}

async fn run(mode: Mode) -> Result<(), Box<dyn std::error::Error>> {
    if mode != Mode::Headless {
        if let Some(app) = live::connect(&Endpoint::for_user()).await {
            // The agent edits the book the person has open, through the app.
            return Ok(live::relay(app, tokio::io::stdin(), tokio::io::stdout()).await?);
        }
        if mode == Mode::Live {
            return Err("Folio is not running, or is not listening for agents".into());
        }
    }
    let books = Books::new(Arc::new(CliHost));
    if let Some(dir) = app_data_dir() {
        books.spelling.keep_in(dir.join("dictionaries"));
    }
    let service = FolioServer::new(Arc::new(books)).serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}

/// Adds Folio to `client`, or says how.
fn install_into(client: Client, print: bool) -> Result<(), Box<dyn std::error::Error>> {
    let exe = std::env::current_exe()?;
    let at = Places::here().ok_or("cannot find your home folder")?;
    match install::target(client, &at, &exe) {
        Target::Command(words) => {
            println!("{}", words.join(" "));
            if print {
                return Ok(());
            }
            let status = std::process::Command::new(&words[0])
                .args(&words[1..])
                .status()
                .map_err(|e| format!("could not run `{}`: {e}", words[0]))?;
            if !status.success() {
                return Err(format!("`{}` did not succeed", words[0]).into());
            }
        }
        Target::File(path) => {
            if print {
                let existing = std::fs::read_to_string(&path).unwrap_or_default();
                let shown = path.display().to_string();
                println!(
                    "{shown}:\n{}",
                    install::with_server(client, &existing, &exe, &shown)?
                );
                return Ok(());
            }
            install::write_server(client, &path, &exe)?;
            println!("Added Folio to {}", path.display());
            println!("Restart {} to see it.", client.name());
        }
    }
    Ok(())
}

/// Says what is and is not set up.
async fn doctor() -> Result<(), Box<dyn std::error::Error>> {
    let exe = std::env::current_exe()?;
    println!(
        "folio-mcp {} at {}",
        env!("CARGO_PKG_VERSION"),
        exe.display()
    );
    let endpoint = Endpoint::for_user();
    if live::connect(&endpoint).await.is_some() {
        println!("✓ Folio is running and listening for agents");
    } else {
        println!("– Folio is not running; an agent will work on books of its own");
    }
    let Some(at) = Places::here() else {
        return Ok(());
    };
    for client in Client::ALL {
        match install::target(client, &at, &exe) {
            Target::File(path) => {
                let listed = std::fs::read_to_string(&path)
                    .is_ok_and(|text| install::lists_server(client, &text));
                let mark = if listed { "✓" } else { "–" };
                println!("{mark} {}: {}", client.name(), path.display());
            }
            Target::Command(_) => println!(
                "– {}: add it with `folio-mcp install {}`",
                client.name(),
                client.name()
            ),
        }
    }
    Ok(())
}
