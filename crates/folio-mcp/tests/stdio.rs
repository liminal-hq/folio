// The server driven by a client over an in-memory pipe, as an agent harness would drive it
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::sync::Arc;

use base64::Engine as _;
use folio_engine::Books;
use folio_mcp::host::CliHost;
use folio_mcp::server::INTENT_SCHEMA_URI;
use folio_mcp::FolioServer;
use rmcp::model::{CallToolRequestParams, ContentBlock, ReadResourceRequestParams};
use rmcp::ServiceExt;
use serde_json::{json, Value};

type Client = rmcp::service::RunningService<rmcp::RoleClient, ()>;

async fn connect() -> Client {
    let (server_side, client_side) = tokio::io::duplex(1 << 20);
    let server = FolioServer::new(Arc::new(Books::new(Arc::new(CliHost))));
    tokio::spawn(async move {
        let running = server.serve(server_side).await.expect("the server starts");
        running.waiting().await.expect("the server runs");
    });
    ().serve(client_side).await.expect("the client connects")
}

fn call(name: &'static str, arguments: Value) -> CallToolRequestParams {
    CallToolRequestParams::new(name)
        .with_arguments(arguments.as_object().cloned().unwrap_or_default())
}

fn first_text(blocks: &[ContentBlock]) -> String {
    blocks
        .iter()
        .find_map(|b| match b {
            ContentBlock::Text(t) => Some(t.text.clone()),
            _ => None,
        })
        .expect("a text answer")
}

#[tokio::test]
async fn an_agent_makes_a_book_and_looks_at_its_cover() {
    let client = connect().await;

    let tools = client.list_all_tools().await.unwrap();
    let names: Vec<&str> = tools.iter().map(|t| &*t.name).collect();
    for wanted in [
        "list_kinds",
        "new_book",
        "get_book",
        "render_page",
        "preflight",
    ] {
        assert!(names.contains(&wanted), "{wanted} is offered: {names:?}");
    }

    let made = client.call_tool(call("new_book", json!({}))).await.unwrap();
    assert_ne!(made.is_error, Some(true));
    let made: Value = serde_json::from_str(&first_text(&made.content)).unwrap();
    let id = made["id"].as_str().unwrap().to_owned();
    let cover = made["book"]["spreads"][0]["id"]
        .as_str()
        .unwrap()
        .to_owned();

    let drawn = client
        .call_tool(call(
            "render_page",
            json!({ "id": id, "spread": cover, "width": 300 }),
        ))
        .await
        .unwrap();
    let ContentBlock::Image(image) = &drawn.content[0] else {
        panic!("a picture comes back: {:?}", drawn.content);
    };
    assert_eq!(image.mime_type, "image/png");
    let png = base64::engine::general_purpose::STANDARD
        .decode(&image.data)
        .unwrap();
    assert!(png.starts_with(b"\x89PNG"));

    let missing = client
        .call_tool(call("get_book", json!({ "id": "bs_999" })))
        .await
        .unwrap();
    assert_eq!(missing.is_error, Some(true));

    client.cancel().await.unwrap();
}

#[tokio::test]
async fn the_intent_schema_is_a_resource() {
    let client = connect().await;
    let listed = client.list_all_resources().await.unwrap();
    assert!(listed.iter().any(|r| r.uri == INTENT_SCHEMA_URI));
    let read = client
        .read_resource(ReadResourceRequestParams::new(INTENT_SCHEMA_URI))
        .await
        .unwrap();
    let json = serde_json::to_string(&read).unwrap();
    assert!(json.contains("MoveFrame"), "the schema names the intents");
    client.cancel().await.unwrap();
}

#[test]
fn every_example_in_the_intent_guide_is_an_intent() {
    let mut seen = 0;
    let mut rest = folio_mcp::server::INTENT_GUIDE;
    while let Some(start) = rest.find("```json\n") {
        let body = &rest[start + 8..];
        let end = body.find("```").expect("a closed block");
        let example = &body[..end];
        if let Err(e) = serde_json::from_str::<folio_model::Intent>(example) {
            panic!("this example is not an intent: {e}\n{example}");
        }
        seen += 1;
        rest = &body[end..];
    }
    assert!(seen >= 8, "the guide has its examples: {seen}");
}

#[tokio::test]
async fn an_agent_changes_a_book_in_one_undoable_step_and_exports_it() {
    let client = connect().await;
    let made = client.call_tool(call("new_book", json!({}))).await.unwrap();
    let made: Value = serde_json::from_str(&first_text(&made.content)).unwrap();
    let id = made["id"].as_str().unwrap().to_owned();
    let cover = made["book"]["spreads"][0]["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let frame = made["book"]["spreads"][0]["frames"][0]["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let moved = |x: f64| {
        json!({"type": "MoveFrame", "on": {"spread": cover}, "frame": frame,
               "box": {"x": x, "y": 0, "w": 100, "h": 100}})
    };

    // Two intents, one step.
    let applied = client
        .call_tool(call(
            "apply_intents",
            json!({"id": id, "intents": [moved(10.0), moved(20.0)]}),
        ))
        .await
        .unwrap();
    assert_ne!(applied.is_error, Some(true), "{:?}", applied.content);
    let applied: Value = serde_json::from_str(&first_text(&applied.content)).unwrap();
    assert_eq!(applied["applied"], 2);

    // An intent Folio does not know is refused with the reason, and nothing is applied.
    let unknown = client
        .call_tool(call(
            "apply_intents",
            json!({"id": id, "intents": [moved(30.0), {"type": "Teleport"}]}),
        ))
        .await
        .unwrap();
    assert_eq!(unknown.is_error, Some(true));
    assert!(first_text(&unknown.content).contains("intent 1"));

    // One undo takes the whole batch back.
    client
        .call_tool(call("undo", json!({"id": id})))
        .await
        .unwrap();
    let book = client
        .call_tool(call("get_book", json!({"id": id})))
        .await
        .unwrap();
    let book: Value = serde_json::from_str(&first_text(&book.content)).unwrap();
    assert_eq!(book["canUndo"], false);

    // Saved, it exports a PDF.
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("Pip").join("Pip.folio");
    client
        .call_tool(call(
            "save_book",
            json!({"id": id, "path": file.to_string_lossy()}),
        ))
        .await
        .unwrap();
    let exported = client
        .call_tool(call("export", json!({"id": id, "targets": ["sharingPdf"]})))
        .await
        .unwrap();
    assert_ne!(exported.is_error, Some(true), "{:?}", exported.content);
    let report: Value = serde_json::from_str(&first_text(&exported.content)).unwrap();
    let pdf = std::fs::read(report["files"][0]["path"].as_str().unwrap()).unwrap();
    assert!(pdf.starts_with(b"%PDF-"));
    client.cancel().await.unwrap();
}
