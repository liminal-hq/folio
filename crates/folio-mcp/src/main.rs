// folio-mcp: serves Folio's books to an AI agent over stdio
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::process::ExitCode;
use std::sync::Arc;

use clap::{Parser, ValueEnum};
use folio_engine::Books;
use folio_mcp::host::{app_data_dir, CliHost};
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
}

#[tokio::main]
async fn main() -> ExitCode {
    let args = Args::parse();
    match run(args.mode).await {
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
