// What a person chooses when starting a book, and why a choice can be refused
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use folio_model::BookKind;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use ts_rs::TS;

/// The paper a printed book is on, which decides how thick its spine is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub enum Paper {
    #[default]
    White,
    Cream,
    Colour,
}

/// Where the book's words come from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub enum Words {
    /// The kind's sample pages, to change as you go.
    #[default]
    Samples,
    /// A cover and one empty spread.
    Empty,
}

/// The typeface family the book's styles start in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub enum Look {
    /// Atkinson Hyperlegible: clear, warm, easy to read.
    #[default]
    Friendly,
    /// Inter: clean and sans serif.
    Modern,
    /// Literata: a classic serif body.
    Classic,
}

/// Everything the New book wizard asks, and the quick start fills in with defaults.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", default)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub struct NewBookSpec {
    pub kind: BookKind,
    /// Which of the kind's trim sizes, counting from 0.
    pub trim_index: u32,
    pub paper: Paper,
    /// Left empty, the book keeps the kind's sample title (or "Untitled" when it starts empty).
    pub title: String,
    pub subtitle: String,
    pub author: String,
    /// An ISBN-13 for the paperback, if the author has one. Checked when given.
    pub isbn: String,
    pub words: Words,
    /// The cover's colour, `#rrggbb`.
    pub cover_colour: String,
    pub look: Look,
}

impl Default for NewBookSpec {
    fn default() -> Self {
        Self {
            kind: BookKind::Picture,
            trim_index: 0,
            paper: Paper::default(),
            title: String::new(),
            subtitle: String::new(),
            author: String::new(),
            isbn: String::new(),
            words: Words::default(),
            cover_colour: "#5b2a4a".into(),
            look: Look::default(),
        }
    }
}

/// A choice that cannot make a book.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum PresetError {
    #[error("this kind of book has no size number {0}")]
    NoSuchTrim(u32),
    #[error("`{0}` is not a valid ISBN-13")]
    BadIsbn(String),
    #[error("`{0}` is not a colour written as #rrggbb")]
    BadColour(String),
}

impl Serialize for PresetError {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeMap;
        let kind = match self {
            PresetError::NoSuchTrim(_) => "noSuchTrim",
            PresetError::BadIsbn(_) => "badIsbn",
            PresetError::BadColour(_) => "badColour",
        };
        let mut map = serializer.serialize_map(Some(2))?;
        map.serialize_entry("kind", kind)?;
        map.serialize_entry("message", &self.to_string())?;
        map.end()
    }
}
