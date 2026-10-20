// Which Folio style each style of an imported document is given
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// The person's choices: a style of the document (by its name there) to a Folio style id, or to
/// the empty string for body text.
pub type StyleMap = BTreeMap<String, String>;

/// A paragraph style found in the imported document, and what became of its paragraphs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub struct SourceStyle {
    /// The style's name in the document (`Heading 1`, `Normal`).
    pub name: String,
    /// How many paragraphs use it.
    pub count: u32,
    /// The words of the first of them, shortened, so the person can tell what it is.
    pub sample: String,
    /// The Folio style they were given; none is body text.
    pub folio: Option<String>,
}

/// The Folio styles a document's style can be mapped to, as `(id, name)`.
pub const MAPPABLE: [(&str, &str); 4] = [
    ("", "Body text"),
    ("h1", "Chapter title"),
    ("h2", "Subheading"),
    ("epigraph", "Quotation"),
];

/// Counts the paragraphs of each style of the document as they are read.
#[derive(Default)]
pub struct Tally {
    seen: BTreeMap<String, SourceStyle>,
}

impl Tally {
    /// Notes a paragraph of the style `name` and says which Folio style it takes: the person's
    /// choice for that name, else `default`.
    pub fn see(
        &mut self,
        name: &str,
        default: Option<&str>,
        map: &StyleMap,
        words: &str,
    ) -> Option<String> {
        let folio = match map.get(name) {
            Some(chosen) if chosen.is_empty() => None,
            Some(chosen) => Some(chosen.clone()),
            None => default.map(str::to_owned),
        };
        let entry = self
            .seen
            .entry(name.to_owned())
            .or_insert_with(|| SourceStyle {
                name: name.to_owned(),
                count: 0,
                sample: words.chars().take(48).collect(),
                folio: folio.clone(),
            });
        entry.count += 1;
        folio
    }

    pub fn into_styles(self) -> Vec<SourceStyle> {
        self.seen.into_values().collect()
    }
}
