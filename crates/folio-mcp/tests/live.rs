// The live link: an agent reaches the books the app holds, through the socket and through the relay
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

#![cfg(unix)]

use std::sync::{Arc, Mutex};
use std::time::Duration;

use folio_engine::{Books, BooksChanged, ExportProgress, Host, PatchEvent};
use folio_mcp::live::{self, Endpoint};
use rmcp::model::{CallToolRequestParams, ContentBlock};
use rmcp::ServiceExt;
use serde_json::Value;

/// A host that remembers the changes announced to it, as the app's windows hear them.
#[derive(Default)]
struct Windows {
    changes: Mutex<usize>,
}

impl Host for Windows {
    fn books_changed(&self, _: &BooksChanged) {
        *self.changes.lock().unwrap() += 1;
    }
    fn patched(&self, _: &PatchEvent) {}
    fn export_progress(&self, _: &ExportProgress) {}
    fn home_dir(&self) -> Option<std::path::PathBuf> {
        None
    }
    fn document_dir(&self) -> Option<std::path::PathBuf> {
        None
    }
    fn version(&self) -> String {
        "test".into()
    }
}

/// Starts the app's side: books and a socket in a folder of their own.
async fn app() -> (Arc<Books>, Arc<Windows>, Endpoint, tempfile::TempDir) {
    let dir = tempfile::tempdir().unwrap();
    let endpoint = Endpoint::at(dir.path().join("agent.sock"));
    let windows = Arc::new(Windows::default());
    let books = Arc::new(Books::new(windows.clone()));
    tokio::spawn({
        let (books, endpoint) = (books.clone(), endpoint.clone());
        async move { live::serve(books, &endpoint).await }
    });
    for _ in 0..100 {
        if live::connect(&endpoint).await.is_some() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    (books, windows, endpoint, dir)
}

fn text(blocks: &[ContentBlock]) -> Value {
    let text = blocks
        .iter()
        .find_map(|b| match b {
            ContentBlock::Text(t) => Some(t.text.clone()),
            _ => None,
        })
        .expect("a text answer");
    serde_json::from_str(&text).unwrap()
}

#[tokio::test]
async fn an_agent_on_the_socket_works_on_the_books_the_app_holds() {
    let (books, windows, endpoint, _dir) = app().await;
    let stream = live::connect(&endpoint)
        .await
        .expect("the app is listening");
    let client = ().serve(stream).await.unwrap();

    let before = *windows.changes.lock().unwrap();
    let made = client
        .call_tool(CallToolRequestParams::new("new_book"))
        .await
        .unwrap();
    let id = text(&made.content)["id"].as_str().unwrap().to_owned();

    // The book is the app's: it is in its list, and its windows were told.
    assert_eq!(books.list().len(), 1);
    assert_eq!(books.list()[0].id, id);
    assert!(*windows.changes.lock().unwrap() > before);
    client.cancel().await.unwrap();
}

#[tokio::test]
async fn the_relay_carries_an_agent_to_the_app_and_back() {
    let (books, _windows, endpoint, _dir) = app().await;
    let app_side = live::connect(&endpoint).await.unwrap();
    // Standard input and output of the server, as a pipe each way to a client.
    let (agent, harness) = tokio::io::duplex(1 << 20);
    let (harness_read, harness_write) = tokio::io::split(harness);
    tokio::spawn(live::relay(app_side, harness_read, harness_write));
    let client = ().serve(agent).await.unwrap();

    let made = client
        .call_tool(CallToolRequestParams::new("new_book"))
        .await
        .unwrap();
    assert_ne!(made.is_error, Some(true));
    assert_eq!(books.list().len(), 1);
    client.cancel().await.unwrap();
}

#[tokio::test]
async fn a_second_copy_cannot_take_the_socket_a_running_app_holds() {
    let (_books, _windows, endpoint, _dir) = app().await;
    let other = Arc::new(Books::new(Arc::new(Windows::default())));
    let taken = live::serve(other, &endpoint).await;
    assert_eq!(taken.unwrap_err().kind(), std::io::ErrorKind::AddrInUse);
}

#[tokio::test]
async fn a_socket_left_by_a_dead_app_is_taken_over() {
    let dir = tempfile::tempdir().unwrap();
    let endpoint = Endpoint::at(dir.path().join("agent.sock"));
    // A socket file nothing listens on, as a crash leaves behind.
    drop(std::os::unix::net::UnixListener::bind(endpoint.path()).unwrap());
    assert!(endpoint.path().exists());
    let books = Arc::new(Books::new(Arc::new(Windows::default())));
    tokio::spawn({
        let endpoint = endpoint.clone();
        async move { live::serve(books, &endpoint).await }
    });
    let mut connected = false;
    for _ in 0..100 {
        if live::connect(&endpoint).await.is_some() {
            connected = true;
            break;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    assert!(connected);
}
