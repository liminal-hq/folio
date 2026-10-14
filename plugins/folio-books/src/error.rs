// The plugin's error: a stable kind and a message
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use folio_package::PackageError;
use folio_presets::PresetError;
use folio_session::SessionError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    /// A window asked about a book that is not open (it may have been closed from another window).
    #[error("there is no open book `{0}`")]
    NoSuchBook(String),
    /// A book that has never been saved was saved without saying where.
    #[error("this book has not been saved yet, so it needs a place to go")]
    NeedsPath,
    #[error(transparent)]
    Preset(#[from] PresetError),
    #[error(transparent)]
    Package(#[from] PackageError),
    #[error(transparent)]
    Session(#[from] SessionError),
    #[error(transparent)]
    Layout(#[from] folio_layout::LayoutError),
    /// A window asked for a spread the book does not have.
    #[error("the book has no spread `{0}`")]
    NoSuchSpread(String),
}

impl Error {
    /// The stable name the frontend tells errors apart by.
    fn kind(&self) -> &'static str {
        match self {
            Error::NoSuchBook(_) => "noSuchBook",
            Error::NeedsPath => "needsPath",
            Error::Preset(_) => "preset",
            Error::Package(PackageError::NotFolio(_)) => "notFolio",
            Error::Package(PackageError::NewerFormat { .. } | PackageError::NewerSchema { .. }) => {
                "newerVersion"
            }
            Error::Package(_) => "package",
            Error::Session(_) => "session",
            Error::Layout(_) => "layout",
            Error::NoSuchSpread(_) => "noSuchSpread",
        }
    }
}

impl serde::Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeMap;
        let mut map = serializer.serialize_map(Some(2))?;
        map.serialize_entry("kind", self.kind())?;
        map.serialize_entry("message", &self.to_string())?;
        map.end()
    }
}
