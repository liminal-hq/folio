// Exporters: books out of Folio as PDF and EPUB
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Everything here draws from the page scene (`folio_layout::scene`), never from the canvas, so
//! a file holds the very glyphs layout chose.

pub mod error;
pub mod job;
pub mod pdf;

pub use error::ExportError;
pub use job::{export_target, slug, Exported, Target};
