// The server's tools and resources, over the same books the app opens
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::path::Path;
use std::sync::Arc;

use base64::Engine as _;
use folio_engine::{Books, ExportColour, ExportTarget};
use folio_model::Intent;
use folio_presets::NewBookSpec;
use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{
    CallToolResult, ContentBlock, Implementation, ListResourcesResult, PaginatedRequestParams,
    ReadResourceRequestParams, ReadResourceResponse, ReadResourceResult, Resource,
    ResourceContents, ServerCapabilities, ServerConfig,
};
use rmcp::schemars::{self, JsonSchema};
use rmcp::service::RequestContext;
use rmcp::{tool, tool_handler, tool_router, ErrorData, RoleServer, ServerHandler};
use serde::Deserialize;
use serde_json::{json, Value};

/// The URI of the schema of the intents `apply_intents` takes.
pub const INTENT_SCHEMA_URI: &str = "folio://schema/intent";

/// The URI of the guide to the intents used most, with examples.
pub const INTENT_GUIDE_URI: &str = "folio://guide/intents";

/// The guide to the intents used most. Its examples are checked against `Intent` by a test.
pub const INTENT_GUIDE: &str = include_str!("guide.md");

const INSTRUCTIONS: &str = "Folio lays out books. Start with `list_kinds`, make a book with `new_book` or open one with `open_book`, bring a manuscript in with `import_manuscript`, change it with `apply_intents` (read the `folio://guide/intents` resource first), look at it with `get_book` and `render_page`, check it with `preflight`, and make files with `export`. Every book tool takes the `id` that `new_book` or `open_book` returned. The person may have the same book open in Folio: their window follows your changes, and they can undo them.";

/// A Book tool's request about a book.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct BookReport {
    /// The id `new_book` or `open_book` returned.
    pub id: String,
    /// What to ask, such as `{"tool": "pictures"}`.
    pub request: folio_engine::ToolRequest,
}

/// Names a book that is open.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct BookRef {
    /// The id `new_book` or `open_book` returned.
    pub id: String,
}

/// Names a book to open from a file.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct OpenBook {
    /// The path of a `.folio` file.
    pub path: String,
}

/// Names a spread of an open book, and how its picture is drawn.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct PageRef {
    /// The id `new_book` or `open_book` returned.
    pub id: String,
    /// The spread's id, from `get_book`.
    pub spread: String,
    /// Which page of the spread, counting from 0. A spread of facing pages has two.
    #[serde(default)]
    pub page: usize,
    /// The picture's width in pixels (64 to 4000); 800 when left out.
    #[serde(default)]
    pub width: Option<u32>,
}

/// Names a spread of an open book.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct SpreadRef {
    /// The id `new_book` or `open_book` returned.
    pub id: String,
    /// The spread's id, from `get_book`.
    pub spread: String,
}

/// Changes to make to a book.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ApplyIntents {
    /// The id `new_book` or `open_book` returned.
    pub id: String,
    /// The intents, in order: JSON objects each with a `type` (see the `folio://guide/intents`
    /// and `folio://schema/intent` resources). All are applied as one undo step, or none are.
    pub intents: Vec<Value>,
}

/// A book to save, and where.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct SaveBook {
    /// The id `new_book` or `open_book` returned.
    pub id: String,
    /// Where to save it, a `.folio` path. Left out, the book goes back to where it came from; a
    /// book made with `new_book` needs one.
    #[serde(default)]
    pub path: Option<String>,
}

/// A manuscript to bring into a book.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ImportManuscript {
    /// The id `new_book` or `open_book` returned.
    pub id: String,
    /// The path of a Markdown, plain text, Word (`.docx`), OpenDocument (`.odt`) or Rich Text
    /// (`.rtf`) file.
    pub path: String,
    /// For a Word or OpenDocument file: the Folio style (`h1`, `h2`, `epigraph`, or `body`) to
    /// give each of the document's styles, by the style's name. The report that comes back lists
    /// the styles found.
    #[serde(default)]
    pub styles: Option<std::collections::BTreeMap<String, String>>,
}

/// A picture to bring into a book.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ImportPicture {
    /// The id `new_book` or `open_book` returned.
    pub id: String,
    /// The path of a PNG, JPEG, GIF, WebP or SVG file. An SVG is kept but drawn as a grey box
    /// for now, so use a PNG or JPEG for a picture that must show.
    pub path: String,
}

/// Files to make from a book.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ExportBook {
    /// The id `new_book` or `open_book` returned. The book must have been saved: the files are
    /// written to an `exports` folder beside it.
    pub id: String,
    /// What to make, in this order.
    pub targets: Vec<ExportTarget>,
    /// How print files hold colour; left out, sRGB as drawn.
    #[serde(default)]
    pub colour: ExportColour,
}

/// Folio's tools for an agent, over the books of one `Books`.
#[derive(Clone)]
pub struct FolioServer {
    books: Arc<Books>,
    tool_router: ToolRouter<Self>,
}

fn text(value: &impl serde::Serialize) -> CallToolResult {
    match serde_json::to_string_pretty(value) {
        Ok(json) => CallToolResult::success(vec![ContentBlock::text(json)]),
        Err(e) => failed(format!("could not write the answer: {e}")),
    }
}

fn failed(message: impl Into<String>) -> CallToolResult {
    CallToolResult::error(vec![ContentBlock::text(message.into())])
}

impl FolioServer {
    pub fn new(books: Arc<Books>) -> Self {
        Self {
            books,
            tool_router: Self::tool_router(),
        }
    }

    /// Runs `work` on a thread of its own: laying out and drawing a page is slow, and the server
    /// must go on answering.
    async fn blocking<T: Send + 'static>(
        &self,
        work: impl FnOnce(&Books) -> T + Send + 'static,
    ) -> Result<T, ErrorData> {
        let books = self.books.clone();
        tokio::task::spawn_blocking(move || work(&books))
            .await
            .map_err(|e| ErrorData::internal_error(format!("the work stopped: {e}"), None))
    }
}

/// A book as a list of spreads, each with the ids and kinds of its frames: enough to know what to
/// ask `get_spread` for.
fn outline(book: &Value) -> Value {
    let spreads: Vec<Value> = book["spreads"]
        .as_array()
        .map(|spreads| {
            spreads
                .iter()
                .map(|spread| {
                    let frames: Vec<Value> = spread["frames"]
                        .as_array()
                        .map(|frames| {
                            frames
                                .iter()
                                .map(|f| json!({ "id": f["id"], "type": f["type"] }))
                                .collect()
                        })
                        .unwrap_or_default();
                    json!({
                        "id": spread["id"],
                        "kind": spread["kind"],
                        "pages": spread["pages"].as_array().map_or(0, Vec::len),
                        "frames": frames,
                    })
                })
                .collect()
        })
        .unwrap_or_default();
    json!({
        "trim": book["trim"],
        "bleed": book["bleed"],
        "margins": book["margins"],
        "spreads": spreads,
        "flows": book["flows"].as_array().map_or(0, Vec::len),
    })
}

#[tool_router]
impl FolioServer {
    #[tool(
        description = "The kinds of book Folio makes (picture book, novel and so on), each with the trim sizes it comes in. Use these to fill in `new_book`.",
        annotations(read_only_hint = true)
    )]
    async fn list_kinds(&self) -> Result<CallToolResult, ErrorData> {
        Ok(text(&self.books.presets()))
    }

    #[tool(
        description = "Makes a new book in memory and opens it. Every field is optional; the answer carries the book's `id` for the other tools. Nothing is written to disk until `save_book`.",
        annotations(read_only_hint = false, destructive_hint = false)
    )]
    async fn new_book(
        &self,
        Parameters(spec): Parameters<NewBookSpec>,
    ) -> Result<CallToolResult, ErrorData> {
        let made = self.blocking(move |books| books.new_book(&spec)).await?;
        Ok(match made {
            Ok(snapshot) => text(&json!({
                "id": snapshot.summary.id,
                "title": snapshot.summary.title,
                "revision": snapshot.revision,
                "book": outline(&serde_json::to_value(&snapshot.book).unwrap_or_default()),
            })),
            Err(e) => failed(e.to_string()),
        })
    }

    #[tool(
        description = "Opens a `.folio` file. If it is already open the same id comes back.",
        annotations(read_only_hint = true)
    )]
    async fn open_book(
        &self,
        Parameters(OpenBook { path }): Parameters<OpenBook>,
    ) -> Result<CallToolResult, ErrorData> {
        let opened = self
            .blocking(move |books| books.open(Path::new(&path)))
            .await?;
        Ok(match opened {
            Ok(snapshot) => text(&json!({
                "id": snapshot.summary.id,
                "title": snapshot.summary.title,
                "revision": snapshot.revision,
                "book": outline(&serde_json::to_value(&snapshot.book).unwrap_or_default()),
            })),
            Err(e) => failed(e.to_string()),
        })
    }

    #[tool(
        description = "The books that are open: id, title, kind, file and whether there are unsaved changes.",
        annotations(read_only_hint = true)
    )]
    async fn list_books(&self) -> Result<CallToolResult, ErrorData> {
        Ok(text(&self.books.list()))
    }

    #[tool(
        description = "An open book's shape: trim, bleed, margins and its spreads with the ids and kinds of their frames.",
        annotations(read_only_hint = true)
    )]
    async fn get_book(
        &self,
        Parameters(BookRef { id }): Parameters<BookRef>,
    ) -> Result<CallToolResult, ErrorData> {
        Ok(match self.books.snapshot(&id) {
            Ok(snapshot) => text(&json!({
                "id": id,
                "title": snapshot.summary.title,
                "revision": snapshot.revision,
                "canUndo": snapshot.can_undo,
                "canRedo": snapshot.can_redo,
                "book": outline(&serde_json::to_value(&snapshot.book).unwrap_or_default()),
            })),
            Err(e) => failed(e.to_string()),
        })
    }

    #[tool(
        description = "One spread in full: its frames as the book holds them, and how the words were laid out in them (lines, and whether text overflows a frame).",
        annotations(read_only_hint = true)
    )]
    async fn get_spread(
        &self,
        Parameters(SpreadRef { id, spread }): Parameters<SpreadRef>,
    ) -> Result<CallToolResult, ErrorData> {
        let wanted = spread.clone();
        let found = self
            .blocking(move |books| {
                let snapshot = books.snapshot(&id)?;
                let laid = books.layout_spread(&id, &spread, None)?;
                Ok::<_, folio_engine::Error>((snapshot, laid))
            })
            .await?;
        Ok(match found {
            Ok((snapshot, laid)) => {
                let book = serde_json::to_value(&snapshot.book).unwrap_or_default();
                let held = book["spreads"]
                    .as_array()
                    .and_then(|all| all.iter().find(|s| s["id"] == wanted.as_str()))
                    .cloned();
                match held {
                    Some(held) => text(&json!({
                        "revision": laid.revision,
                        "spread": held,
                        "layout": laid.layout,
                    })),
                    None => failed(format!("no spread {wanted}")),
                }
            }
            Err(e) => failed(e.to_string()),
        })
    }

    #[tool(
        description = "A picture (PNG) of one page of a spread, as it is drawn, so the result of an edit can be looked at.",
        annotations(read_only_hint = true)
    )]
    async fn render_page(
        &self,
        Parameters(PageRef {
            id,
            spread,
            page,
            width,
        }): Parameters<PageRef>,
    ) -> Result<CallToolResult, ErrorData> {
        let drawn = self
            .blocking(move |books| books.render_page(&id, &spread, page, width.unwrap_or(800)))
            .await?;
        Ok(match drawn {
            Ok(png) => CallToolResult::success(vec![ContentBlock::image(
                base64::engine::general_purpose::STANDARD.encode(png),
                "image/png",
            )]),
            Err(e) => failed(e.to_string()),
        })
    }

    #[tool(
        description = "The problems a printer or store would find in the book: low-resolution pictures, text outside the margins, overflowing frames and so on, each with where it is.",
        annotations(read_only_hint = true)
    )]
    async fn preflight(
        &self,
        Parameters(BookRef { id }): Parameters<BookRef>,
    ) -> Result<CallToolResult, ErrorData> {
        let checked = self.blocking(move |books| books.preflight(&id)).await?;
        Ok(match checked {
            Ok(items) => text(&items),
            Err(e) => failed(e.to_string()),
        })
    }

    #[tool(
        name = "book_report",
        description = "What one of the app's Book tools finds in the book, as the tools window shows it. The request is `{\"tool\": \"pictures\"}` for every picture with its pixels per inch where it is set, its status for print and its description.",
        annotations(read_only_hint = true)
    )]
    async fn book_report(
        &self,
        Parameters(BookReport { id, request }): Parameters<BookReport>,
    ) -> Result<CallToolResult, ErrorData> {
        let answered = self
            .blocking(move |books| books.book_tool(&id, &request))
            .await?;
        Ok(match answered {
            Ok(report) => text(&report),
            Err(e) => failed(e.to_string()),
        })
    }

    #[tool(
        description = "Changes a book: applies a list of intents (see the `folio://guide/intents` resource) as one step the person can undo, or none of them if one is refused. Answers with the new revision. Look at the result with `render_page`.",
        annotations(read_only_hint = false, destructive_hint = false)
    )]
    async fn apply_intents(
        &self,
        Parameters(ApplyIntents { id, intents }): Parameters<ApplyIntents>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut parsed = Vec::with_capacity(intents.len());
        for (index, value) in intents.into_iter().enumerate() {
            match serde_json::from_value::<Intent>(value) {
                Ok(intent) => parsed.push(intent),
                Err(e) => {
                    return Ok(failed(format!(
                        "intent {index} (counting from 0) is not one Folio knows, so nothing was applied: {e}"
                    )))
                }
            }
        }
        let applied = self
            .blocking(move |books| books.apply_batch(&id, parsed))
            .await?;
        Ok(match applied {
            Ok(outcome) => text(&outcome),
            Err(e) => failed(e.to_string()),
        })
    }

    #[tool(
        description = "Undoes the last step, which may be the person's or an earlier batch of yours.",
        annotations(read_only_hint = false, destructive_hint = false)
    )]
    async fn undo(
        &self,
        Parameters(BookRef { id }): Parameters<BookRef>,
    ) -> Result<CallToolResult, ErrorData> {
        Ok(match self.books.undo(&id) {
            Ok(patch) => text(&json!({ "undone": patch.is_some() })),
            Err(e) => failed(e.to_string()),
        })
    }

    #[tool(
        description = "Redoes the last undone step.",
        annotations(read_only_hint = false, destructive_hint = false)
    )]
    async fn redo(
        &self,
        Parameters(BookRef { id }): Parameters<BookRef>,
    ) -> Result<CallToolResult, ErrorData> {
        Ok(match self.books.redo(&id) {
            Ok(patch) => text(&json!({ "redone": patch.is_some() })),
            Err(e) => failed(e.to_string()),
        })
    }

    #[tool(
        description = "Saves a book to a `.folio` file, which overwrites the file if there is one.",
        annotations(read_only_hint = false, destructive_hint = true)
    )]
    async fn save_book(
        &self,
        Parameters(SaveBook { id, path }): Parameters<SaveBook>,
    ) -> Result<CallToolResult, ErrorData> {
        let saved = self
            .blocking(move |books| books.save(&id, path.as_deref().map(Path::new)))
            .await?;
        Ok(match saved {
            Ok(summary) => text(&summary),
            Err(e) => failed(e.to_string()),
        })
    }

    #[tool(
        description = "Closes a book without saving it. Unsaved changes are lost.",
        annotations(read_only_hint = false, destructive_hint = true)
    )]
    async fn close_book(
        &self,
        Parameters(BookRef { id }): Parameters<BookRef>,
    ) -> Result<CallToolResult, ErrorData> {
        Ok(match self.books.close(&id) {
            Ok(()) => text(&json!({ "closed": id })),
            Err(e) => failed(e.to_string()),
        })
    }

    #[tool(
        description = "Brings a manuscript (Markdown, plain text, Word, OpenDocument or Rich Text) in as the story of the book's first text flow, as one undo step. Answers with a report of the chapters found, what was left out, and the document's styles and the Folio style each got.",
        annotations(read_only_hint = false, destructive_hint = false)
    )]
    async fn import_manuscript(
        &self,
        Parameters(ImportManuscript { id, path, styles }): Parameters<ImportManuscript>,
    ) -> Result<CallToolResult, ErrorData> {
        let imported = self
            .blocking(move |books| books.import_text_with(&id, &path, &styles.unwrap_or_default()))
            .await?;
        Ok(match imported {
            Ok(report) => text(&report),
            Err(e) => failed(e.to_string()),
        })
    }

    #[tool(
        description = "Brings a picture into the book and answers with its asset path, to give to a `SetImage` intent.",
        annotations(read_only_hint = false, destructive_hint = false)
    )]
    async fn import_picture(
        &self,
        Parameters(ImportPicture { id, path }): Parameters<ImportPicture>,
    ) -> Result<CallToolResult, ErrorData> {
        let imported = self
            .blocking(move |books| books.import_image(&id, Path::new(&path)))
            .await?;
        Ok(match imported {
            Ok(asset) => text(&json!({ "asset": asset })),
            Err(e) => failed(e.to_string()),
        })
    }

    #[tool(
        description = "Makes files from a book: a sharing PDF, a KDP interior or cover PDF, an EPUB and so on, written to an `exports` folder beside the saved book. Answers with the files made.",
        annotations(read_only_hint = false, destructive_hint = false)
    )]
    async fn export(
        &self,
        Parameters(ExportBook {
            id,
            targets,
            colour,
        }): Parameters<ExportBook>,
    ) -> Result<CallToolResult, ErrorData> {
        let made = self
            .blocking(move |books| books.export(&id, &targets, colour))
            .await?;
        Ok(match made {
            Ok(report) => text(&report),
            Err(e) => failed(e.to_string()),
        })
    }

    #[tool(
        description = "The chapters of the book's story: each heading and where it starts.",
        annotations(read_only_hint = true)
    )]
    async fn story_chapters(
        &self,
        Parameters(BookRef { id }): Parameters<BookRef>,
    ) -> Result<CallToolResult, ErrorData> {
        Ok(match self.books.story_chapters(&id) {
            Ok(chapters) => text(&chapters),
            Err(e) => failed(e.to_string()),
        })
    }
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for FolioServer {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(
            ServerCapabilities::builder()
                .enable_tools()
                .enable_resources()
                .build(),
        )
        .with_server_info(Implementation::new("folio-mcp", env!("CARGO_PKG_VERSION")))
        .with_instructions(INSTRUCTIONS)
    }

    async fn list_resources(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, ErrorData> {
        let mut intents = Resource::new(INTENT_SCHEMA_URI, "intent-schema");
        intents.title = Some("Folio intents".into());
        intents.description = Some(
            "The JSON Schema of an intent: one change to a book, as `apply_intents` takes it."
                .into(),
        );
        intents.mime_type = Some("application/schema+json".into());
        let mut guide = Resource::new(INTENT_GUIDE_URI, "intent-guide");
        guide.title = Some("Changing a book with intents".into());
        guide.description = Some(
            "The intents used most, each with an example. Read this before `apply_intents`.".into(),
        );
        guide.mime_type = Some("text/markdown".into());
        Ok(ListResourcesResult {
            resources: vec![guide, intents],
            ..Default::default()
        })
    }

    async fn read_resource(
        &self,
        request: ReadResourceRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResponse, ErrorData> {
        if request.uri == INTENT_GUIDE_URI {
            return Ok(
                ReadResourceResult::new(vec![ResourceContents::TextResourceContents {
                    uri: INTENT_GUIDE_URI.into(),
                    mime_type: Some("text/markdown".into()),
                    text: INTENT_GUIDE.into(),
                    meta: None,
                }])
                .into(),
            );
        }
        if request.uri != INTENT_SCHEMA_URI {
            return Err(ErrorData::invalid_params(
                format!("no resource {}", request.uri),
                None,
            ));
        }
        let schema = serde_json::to_string_pretty(&schemars::schema_for!(Intent))
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
        Ok(
            ReadResourceResult::new(vec![ResourceContents::TextResourceContents {
                uri: INTENT_SCHEMA_URI.into(),
                mime_type: Some("application/schema+json".into()),
                text: schema,
                meta: None,
            }])
            .into(),
        )
    }
}
