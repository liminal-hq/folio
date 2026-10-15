// Why an intent was refused
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use thiserror::Error;

/// An intent that could not be applied. Nothing changed.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum SessionError {
    #[error("there is no spread `{0}`")]
    NoSuchSpread(String),
    #[error("there is no master page `{0}`")]
    NoSuchMaster(String),
    #[error("there is no frame `{0}`")]
    NoSuchFrame(String),
    #[error("a frame `{0}` is already in the book")]
    DuplicateFrame(String),
    #[error("frame `{0}` is not a text frame")]
    NotText(String),
    #[error("the text of frame `{0}` comes from a flow, so it is not edited in place")]
    FlowText(String),
    #[error("a book needs at least one spread")]
    LastSpread,
    #[error("the cover stays where it is")]
    CoverFixed,
    #[error("frame `{0}` is not a picture frame")]
    NotImage(String),
    #[error("{0}")]
    BadGeometry(String),
    #[error("`{0}` is not a picture in this book's assets")]
    BadAsset(String),
    #[error("the frame `{0}` belongs to a master page")]
    FromMaster(String),
    #[error("the page number `{0}` comes from the page numbers choice: change that to remove it")]
    PageNumber(String),
    #[error("the frame `{0}` is locked")]
    Locked(String),
    #[error("the frame `{0}` is not on spread `{1}`")]
    NotOnSpread(String, String),
    #[error("there is no guide {0} on `{1}`")]
    NoSuchGuide(usize, String),
    #[error("guide {0} is locked")]
    GuideLocked(usize),
    #[error("a guide in the gutter, off the pages or on the cover cannot go on every page")]
    CannotPromote,
    #[error("the master page `{0}` is not used on spread `{1}`")]
    MasterNotOnSpread(String, String),
    #[error("a guide needs a position on the page")]
    BadGuide,
}
