// Why an export did not happen
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ExportError {
    #[error("layout failed: {0}")]
    Layout(#[from] folio_layout::error::LayoutError),
    #[error("the PDF could not be written: {0}")]
    Pdf(String),
    #[error("the EPUB could not be written: {0}")]
    Epub(String),
    #[error("the book has no pages to export")]
    NoPages,
}
