// The book itself: its kind, geometry, contributors and metadata
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::flow::Flow;
use crate::frame::Frame;
use crate::ids::{BookId, MasterId};
use crate::spread::Spread;
use crate::units::{Margins, Pt, Size};

/// The `book.json` schema version this build writes (and the newest it reads).
pub const SCHEMA_VERSION: u32 = 1;

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
    /// The master pages: frames that repeat on every page that uses them.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub master_pages: Vec<Master>,
    /// Where the page numbers go, kept so the masters can be made again when pages come and go.
    #[serde(default, skip_serializing_if = "PageNumbers::is_none")]
    pub page_numbers: PageNumbers,
    pub metadata: BookMetadata,
    /// Keys at the top of `book.json` that this version does not know, from imports or a newer
    /// version of Folio, kept as they were so a save does not lose them. The UI never sees them.
    #[serde(flatten, default, skip_serializing_if = "BTreeMap::is_empty")]
    #[ts(skip)]
    pub extensions: BTreeMap<String, serde_json::Value>,
}

/// A master page: frames drawn on every page that uses it, in the page's own coordinates (the
/// origin is the top-left of its trim). Text in them may hold `{page}` and `{book.title}`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub struct Master {
    pub id: MasterId,
    pub name: String,
    pub frames: Vec<Frame>,
}

/// Where the page numbers go.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
#[serde(rename_all = "lowercase")]
pub enum PageNumbers {
    /// There are none.
    #[default]
    None,
    /// In the middle of the bottom margin.
    Centre,
    /// In the bottom margin, at the outer edge of each page.
    Outside,
}

impl PageNumbers {
    pub fn is_none(&self) -> bool {
        *self == PageNumbers::None
    }
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

impl Book {
    /// The number of pages in the book, not counting the cover, which a printer counts apart.
    pub fn page_count(&self) -> usize {
        self.spreads
            .iter()
            .filter(|s| s.kind != crate::spread::SpreadKind::Cover)
            .map(|s| s.pages.len())
            .sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_page_count_leaves_out_the_cover() {
        let book: Book = serde_json::from_value(serde_json::json!({
            "schema": 1, "id": "b", "title": "T", "authors": [], "language": "en", "kind": "picture",
            "layout": "fixed", "trim": { "w": 1, "h": 1 }, "bleed": 0,
            "margins": { "top": 0, "bottom": 0, "inside": 0, "outside": 0 },
            "grid": { "columns": 1, "gutter": 0, "baseline": null },
            "spreads": [
                { "id": "c", "kind": "cover", "pages": [{ "id": "p0", "master": null }], "frames": [] },
                { "id": "a", "kind": "opener", "pages": [{ "id": "p1", "master": null }], "frames": [] },
                { "id": "b", "kind": "spread", "pages": [{ "id": "p2", "master": null }, { "id": "p3", "master": null }], "frames": [] }
            ],
            "metadata": { "isbn": { "paperback": null, "hardcover": null, "ebook": null }, "description": "" }
        }))
        .unwrap();
        assert_eq!(book.page_count(), 3);
    }

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
