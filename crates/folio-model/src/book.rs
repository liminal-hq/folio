// The book itself: its kind, geometry, contributors and metadata
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::background::Background;
use crate::cover::{BarcodeChoice, Paper};
use crate::flow::Flow;
use crate::frame::Frame;
use crate::ids::{BookId, MasterId};
use crate::spread::{Guide, Spread};
use crate::styles::Styles;
use crate::units::{Margins, Pt, Size};

/// The `book.json` schema version this build writes (and the newest it reads).
pub const SCHEMA_VERSION: u32 = 2;

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
    /// The paper a printed copy is on, which decides how thick the spine is.
    #[serde(default, skip_serializing_if = "is_default_paper")]
    pub paper: Paper,
    /// Who makes the barcode on the paperback cover.
    #[serde(default, skip_serializing_if = "is_default_barcode")]
    pub barcode: BarcodeChoice,
    /// Words the person has put on the back cover, over the blurb.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub back_notes: Vec<BackNote>,
    /// Preflight issues the person has said to leave alone, each by its key (D-31).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub preflight_ignore: Vec<String>,
    /// Font files kept inside the package, so the book is laid out the same on a machine without
    /// them (D-30). Each names a file under `assets/fonts/`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fonts: Vec<BookFont>,
    /// What the person has told the proofing tools to leave alone, kept with the book.
    #[serde(default, skip_serializing_if = "Proofing::is_empty")]
    pub proofing: Proofing,
    /// The named styles and the typeface body text starts in; `styles.json` in the package.
    #[serde(default, skip_serializing_if = "Styles::is_empty")]
    pub styles: Styles,
    pub metadata: BookMetadata,
    /// Keys at the top of `book.json` that this version does not know, from imports or a newer
    /// version of Folio, kept as they were so a save does not lose them. The UI never sees them.
    #[serde(flatten, default, skip_serializing_if = "BTreeMap::is_empty")]
    #[ts(skip)]
    pub extensions: BTreeMap<String, serde_json::Value>,
}

/// The longest a word on the ignore list can be, in characters, and the most words the list holds.
pub const MAX_IGNORED_WORD: usize = 64;
pub const MAX_IGNORED_WORDS: usize = 5000;

/// The words spelling leaves alone in this book: names, made-up words, terms of the trade. It
/// travels with the file and is undone with it (D-25).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub struct Proofing {
    #[serde(default)]
    pub ignore: Vec<String>,
}

impl Proofing {
    pub fn is_empty(&self) -> bool {
        self.ignore.is_empty()
    }

    /// `word` as it would be kept: trimmed, one word with no spaces, not empty and not too long.
    pub fn clean(word: &str) -> Result<String, String> {
        let word = word.trim();
        if word.is_empty() || word.chars().any(char::is_whitespace) {
            return Err("an ignored word is a single word".into());
        }
        if word.chars().count() > MAX_IGNORED_WORD {
            return Err(format!(
                "an ignored word is at most {MAX_IGNORED_WORD} characters"
            ));
        }
        Ok(word.to_owned())
    }
}

/// A font file the book keeps with it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub struct BookFont {
    /// The family name styles and runs ask for.
    pub family: String,
    /// CSS weight: 400 regular, 700 bold.
    pub weight: u16,
    pub italic: bool,
    /// The file in the package, such as `assets/fonts/9f2c…c1.ttf`.
    pub path: String,
    /// Which face of a collection file; 0 for an ordinary font.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub index: u32,
}

fn is_zero(value: &u32) -> bool {
    *value == 0
}

/// A language tag as it would be kept (a BCP 47 code such as `en-CA` or `fr`): trimmed, a two or
/// three letter language and up to three more parts of letters or digits.
pub fn clean_language(tag: &str) -> Result<String, String> {
    let tag = tag.trim();
    let mut parts = tag.split('-');
    let language_ok = parts
        .next()
        .is_some_and(|l| (2..=3).contains(&l.len()) && l.chars().all(|c| c.is_ascii_alphabetic()));
    let rest: Vec<&str> = parts.collect();
    let rest_ok = rest.len() <= 3
        && rest
            .iter()
            .all(|p| (1..=8).contains(&p.len()) && p.chars().all(|c| c.is_ascii_alphanumeric()));
    if !language_ok || !rest_ok {
        return Err("a language is a code such as en-CA or fr".into());
    }
    Ok(tag.to_owned())
}

/// A master page: frames drawn on every page that uses it, in the page's own coordinates (the
/// origin is the top-left of its trim). Text in them may hold `{page}` and `{book.title}`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub struct Master {
    pub id: MasterId,
    pub name: String,
    pub frames: Vec<Frame>,
    /// Ruler guides that show on every page using this master, in the page's own points (the
    /// origin is the top-left of its trim). Named by their place in the list, as on a spread.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub guides: Vec<Guide>,
    /// The background of every page using this master that has none of its own.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bg: Option<Background>,
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
/// Some words the person has put on the back cover: where they sit, from the top-left of the back
/// cover's trim, and what they say.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub struct BackNote {
    pub id: String,
    pub text: String,
    pub rect: crate::Rect,
    /// The size of the words, in points.
    pub size: f64,
}

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub struct Isbns {
    pub paperback: Option<String>,
    pub hardcover: Option<String>,
    pub ebook: Option<String>,
}

/// What a book says about itself on its cover and in a store: its title, who made it, what it is
/// about and its ISBNs. They are set together, as one undo step.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
#[serde(rename_all = "camelCase")]
pub struct Details {
    pub title: String,
    pub subtitle: Option<String>,
    pub authors: Vec<Contributor>,
    pub description: String,
    pub isbn: Isbns,
}

impl Details {
    /// The details a book has now.
    pub fn of(book: &Book) -> Self {
        Self {
            title: book.title.clone(),
            subtitle: book.subtitle.clone(),
            authors: book.authors.clone(),
            description: book.metadata.description.clone(),
            isbn: book.metadata.isbn.clone(),
        }
    }

    /// Puts these details in `book`, leaving its keywords alone.
    pub fn apply(&self, book: &mut Book) {
        book.title = self.title.clone();
        book.subtitle = self.subtitle.clone();
        book.authors = self.authors.clone();
        book.metadata.description = self.description.clone();
        book.metadata.isbn = self.isbn.clone();
    }
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

fn is_default_barcode(choice: &BarcodeChoice) -> bool {
    *choice == BarcodeChoice::default()
}

fn is_default_paper(paper: &Paper) -> bool {
    *paper == Paper::default()
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
