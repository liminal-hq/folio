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
