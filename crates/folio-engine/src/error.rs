// The books' error: a stable kind and a message
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
    /// A file offered as a picture is not one Folio can place.
    #[error("{0}")]
    BadImage(String),
    /// A dictionary could not be downloaded, checked or kept.
    #[error("{0}")]
    Dictionary(String),
    /// A recovery copy could not be written, read or found.
    #[error("{0}")]
    Recovery(String),
    /// A book could not be exported, or its file could not be written.
    #[error("{0}")]
    Export(String),
    /// A document could not be brought in, with the reason in plain words.
    #[error("{0}")]
    Import(String),
    /// The book is already open somewhere else: in another Folio window, or held by an agent.
    #[error("`{0}` is already open in another window or program, so it cannot be opened here too")]
    Locked(String),
    /// One intent of a batch was refused, and the batch was taken back whole.
    #[error("intent {index} (counting from 0) was refused, so none of the {total} were applied: {message}")]
    Batch {
        index: usize,
        total: usize,
        message: String,
    },
    /// A window asked for a spread the book does not have.
    #[error("the book has no spread `{0}`")]
    NoSuchSpread(String),
    /// A window asked for a master page the book does not have.
    #[error("the book has no master page `{0}`")]
    NoSuchMaster(String),
}

impl Error {
    /// The stable name the frontend tells errors apart by.
    fn kind(&self) -> &'static str {
        match self {
            Error::NoSuchBook(_) => "noSuchBook",
            Error::NeedsPath => "needsPath",
            Error::Locked(_) => "locked",
            Error::Batch { .. } => "batch",
            Error::Preset(_) => "preset",
            Error::Package(PackageError::NotFolio(_)) => "notFolio",
            Error::Package(PackageError::NewerFormat { .. } | PackageError::NewerSchema { .. }) => {
                "newerVersion"
            }
            Error::Package(_) => "package",
            Error::Session(_) => "session",
            Error::Layout(_) => "layout",
            Error::NoSuchSpread(_) => "noSuchSpread",
            Error::NoSuchMaster(_) => "noSuchMaster",
            Error::BadImage(_) => "image",
            Error::Export(_) => "export",
            Error::Import(_) => "import",
            Error::Dictionary(_) => "dictionary",
            Error::Recovery(_) => "recovery",
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
