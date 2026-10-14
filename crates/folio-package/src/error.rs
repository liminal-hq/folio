// What can go wrong reading or writing a `.folio` file
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum PackageError {
    #[error("could not read or write {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("the file is not a readable zip: {0}")]
    Zip(String),
    #[error("this is not a Folio book: {0}")]
    NotFolio(String),
    #[error(
        "the book was saved by a newer Folio (format {found}); this one reads format {supported}"
    )]
    NewerFormat { found: String, supported: String },
    #[error(
        "the book was saved by a newer Folio (schema {found}); this one reads schema {supported}"
    )]
    NewerSchema { found: u32, supported: u32 },
    #[error("{file} could not be read: {message}")]
    Json { file: String, message: String },
    #[error("the book has no `{0}`")]
    Missing(String),
    #[error("`{0}` is not a path inside a book")]
    BadPath(String),
}

impl PackageError {
    pub(crate) fn io(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        Self::Io {
            path: path.into(),
            source,
        }
    }
}

impl From<zip::result::ZipError> for PackageError {
    fn from(error: zip::result::ZipError) -> Self {
        Self::Zip(error.to_string())
    }
}
