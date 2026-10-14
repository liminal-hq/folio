// An open book in memory: the one place intents are validated and applied, with undo and redo
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT
//
// The model is the source of truth and the UI holds a mirrored copy (D-02). The UI sends intents;
// a `Session` checks and applies each one, keeps what is needed to undo it, and answers with a
// `Patch` the UI applies to its copy. The revision grows by one on every change, undo and redo, so
// a window can tell a newer state from an older one that arrives late.
//
// This crate has no `tauri` dependency: the plugin that exposes it to the webview is a thin
// adapter, and the session is tested headlessly.

mod apply;
mod delta;
mod error;
mod masters;
mod patch;
mod session;

pub use error::SessionError;
pub use patch::{Patch, PatchOp};
pub use session::{IdSource, Session, COALESCE_WINDOW, UNDO_DEPTH};
