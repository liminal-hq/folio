// The open books: one session and package each, one writer, events to whoever is listening
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

// Nothing here knows about a window or a toolkit. The app that hosts the books says how events
// reach the people looking at them and where the person's folders are, through `Host`.

mod error;
mod host;
mod punctuation;
mod recovery;
mod spelling;
mod state;
mod tools;
mod wire;

pub use error::Error;
pub use host::Host;
pub use punctuation::PunctuationRule;
pub use recovery::Recovery;
pub use spelling::Spelling;
pub use state::{BookEvent, BookHook, Books};
pub use tools::{
    ChecklistId, ChecklistItem, ChecklistStatus, Findings, Market, PictureRow, PictureStatus,
    PrintCosts, PunctuationExample, PunctuationFound, SaleRoyalty, StyleStray, ToolFix, ToolFixed,
    ToolReport, ToolRequest,
};
pub use wire::*;

/// Sent to every window after a change to an open book, with a `PatchEvent`.
pub const PATCH_EVENT: &str = "folio-books://patch";

/// Sent to every window when books open, close, save or become changed or clean, with a
/// `BooksChanged`.
pub const CHANGED_EVENT: &str = "folio-books://changed";

/// Sent to every window while a dictionary downloads: how much has arrived.
pub const DICTIONARY_PROGRESS_EVENT: &str = "folio-books://dictionary-progress";

/// Sent to every window while an export runs: which file it has reached.
pub const EXPORT_PROGRESS_EVENT: &str = "folio-books://export-progress";
