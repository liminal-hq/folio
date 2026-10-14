// The book itself: its kind, geometry, contributors and metadata
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::flow::Flow;
use crate::ids::{BookId, MasterId};
use crate::spread::Spread;
use crate::units::{Margins, Pt, Size};

/// The kind of book, which selects a preset bundle of trim, bleed, grid, styles and export defaults.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub enum BookKind {
    Picture,
    Novel,
    Textbook,
    Illustrated,
    Poetry,
    Memoir,
}

/// Whether a book's pages are laid out by hand (`Fixed`) or flow from a story (`Flowing`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub enum LayoutMode {
    Fixed,
    Flowing,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
#[serde(rename_all = "camelCase")]
pub struct Book {
    pub schema: u32,
    pub id: BookId,
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subtitle: Option<String>,
    pub authors: Vec<Contributor>,
    pub language: String,
    pub kind: BookKind,
    pub layout: LayoutMode,
    pub trim: Size,
    pub bleed: Pt,
    pub margins: Margins,
    #[serde(default)]
    pub adjust_gutter_for_page_count: bool,
    pub grid: Grid,
    pub spreads: Vec<Spread>,
    #[serde(default)]
    pub flows: Vec<Flow>,
    #[serde(default)]
    pub masters: Vec<MasterId>,
    pub metadata: BookMetadata,
    /// Keys at the top of `book.json` that this version does not know, from imports or a newer
    /// version of Folio, kept as they were so a save does not lose them. The UI never sees them.
    #[serde(flatten, default, skip_serializing_if = "BTreeMap::is_empty")]
    #[ts(skip)]
    pub extensions: BTreeMap<String, serde_json::Value>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub struct Contributor {
    pub name: String,
    pub role: ContributorRole,
}
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
#[serde(rename_all = "lowercase")]
pub enum ContributorRole {
    Author,
    Illustrator,
    Editor,
    Translator,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub struct Grid {
    pub columns: u8,
    pub gutter: Pt,
    pub baseline: Option<Baseline>,
}
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub struct Baseline {
    pub increment: Pt,
    pub offset: Pt,
}

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub struct BookMetadata {
    pub isbn: Isbns,
    pub description: String,
    #[serde(default)]
    pub keywords: Vec<String>,
}
#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub struct Isbns {
    pub paperback: Option<String>,
    pub hardcover: Option<String>,
    pub ebook: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kinds_use_lowercase_names() {
        assert_eq!(
            serde_json::to_string(&BookKind::Picture).unwrap(),
            "\"picture\""
        );
        assert_eq!(
            serde_json::to_string(&LayoutMode::Flowing).unwrap(),
            "\"flowing\""
        );
    }
}
