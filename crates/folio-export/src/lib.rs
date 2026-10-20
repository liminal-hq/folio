// Exporters: books out of Folio as PDF and EPUB
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Everything here draws from the page scene (`folio_layout::scene`), never from the canvas, so
//! a file holds the very glyphs layout chose.

pub mod cover;
pub mod epub;
pub mod error;
pub mod fixed;
pub mod job;
pub mod pdf;
pub mod pdfx;
pub mod raster;
mod texture;
mod time;

pub use error::ExportError;
pub use job::{export_target, slug, Exported, Target};
