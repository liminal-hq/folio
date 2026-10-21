// folio-mcp: serves Folio's books to an AI agent over stdio
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::sync::Arc;

use clap::Parser;
use folio_engine::Books;
use folio_mcp::host::{app_data_dir, CliHost};
use folio_mcp::FolioServer;
use rmcp::{transport::stdio, ServiceExt};

/// Serves Folio's books to an AI agent over the Model Context Protocol.
#[derive(Debug, Parser)]
#[command(version, about)]
struct Args {}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    Args::parse();
    let books = Books::new(Arc::new(CliHost));
    if let Some(dir) = app_data_dir() {
        books.spelling.keep_in(dir.join("dictionaries"));
    }
    let service = FolioServer::new(Arc::new(books)).serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}
