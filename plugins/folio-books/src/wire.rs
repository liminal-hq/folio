// The shapes the plugin sends to the webview, generated as TypeScript
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use folio_layout::wire::SpreadLayout;
use folio_model::{Book, BookKind};
use folio_session::Patch;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use ts_rs::TS;

/// One open book, in a line: enough for a tab, a title bar and the recents.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub struct BookSummary {
    /// The session's id, which every other command takes. Not the book's own id: the same book
    /// opened later is a new session.
    pub id: String,
    pub title: String,
    pub kind: BookKind,
    /// Where the book was opened from or last saved to; `None` for one that has never been saved.
    pub path: Option<String>,
    /// There is something to save: a new book that was never saved, or changes since the last save.
    pub dirty: bool,
}

/// A spread laid out, and the revision of the book it was laid out from. A window draws it only
/// while that is the revision it holds.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub struct SpreadLayoutResult {
    #[ts(type = "number")]
    pub revision: u64,
    pub layout: SpreadLayout,
}

/// An open book in full, for a window that is showing it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub struct BookSnapshot {
    pub summary: BookSummary,
    /// The revision the book is at. Events with a higher revision apply on top of this.
    #[ts(type = "number")]
    pub revision: u64,
    pub book: Book,
    /// `styles.json`, the named styles and the look.
    #[ts(type = "unknown")]
    pub styles: Value,
    /// `settings.json`, the book's own choices such as the paper.
    #[ts(type = "unknown")]
    pub settings: Value,
    pub can_undo: bool,
    pub can_redo: bool,
}

/// Sent to every window after a change to an open book.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub struct PatchEvent {
    pub id: String,
    pub patch: Patch,
    pub can_undo: bool,
    pub can_redo: bool,
}

/// Sent to every window when books open, close, are saved, or become changed or clean.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub struct BooksChanged {
    pub books: Vec<BookSummary>,
}

/// A word the speller does not know, by where it is in the text asked about, in UTF-16 code units.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub struct Misspelling {
    pub start: u32,
    pub end: u32,
    pub word: String,
}

/// What checking a text found.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub struct SpellCheck {
    /// Whether there is a dictionary for the book's language. Without one nothing is flagged, and
    /// the UI can say so instead of showing a clean page.
    pub supported: bool,
    pub misspellings: Vec<Misspelling>,
}
