// Sections: where a book's page numbering changes style or starts again
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::spread::{Spread, SpreadKind};

/// How the pages of a section are numbered.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
#[serde(rename_all = "lowercase")]
pub enum Numbering {
    /// 1, 2, 3.
    #[default]
    Arabic,
    /// i, ii, iii: the front matter.
    Roman,
    /// No number is shown, though the pages are still counted.
    None,
}

/// A section starts at the spread that carries it and runs to the next spread that carries one.
/// Pages before the first are in the book's own implied section: counted from 1 in Arabic.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
#[serde(rename_all = "camelCase")]
pub struct Section {
    /// What the section is called: "Contents", "Part one". The `{section}` token fills it in.
    pub name: String,
    #[serde(default)]
    pub numbering: Numbering,
    /// The number of the section's first page. Without it, counting carries on from the page before.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start: Option<u32>,
}

/// The longest a section's name can be, in characters, and the highest number it can start at.
pub const MAX_SECTION_NAME: usize = 80;
pub const MAX_SECTION_START: u32 = 99_999;

impl Section {
    /// The section as it is kept, or why it cannot be: a trimmed name of a sensible length and a
    /// start within range.
    pub fn checked(&self) -> Result<Section, String> {
        let name = self.name.trim().to_owned();
        if name.chars().count() > MAX_SECTION_NAME {
            return Err(format!(
                "a section's name is at most {MAX_SECTION_NAME} characters"
            ));
        }
        if self.start.is_some_and(|s| s == 0 || s > MAX_SECTION_START) {
            return Err(format!(
                "a section starts at a page from 1 to {MAX_SECTION_START}"
            ));
        }
        Ok(Section {
            name,
            numbering: self.numbering,
            start: self.start,
        })
    }
}

/// What one page is called, in the numbering of its section.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PageLabel {
    /// How the page is written: "7", "vii", or nothing when its section shows no numbers.
    pub label: String,
    /// The name of the section the page is in; empty before the first section.
    pub section: String,
}

/// `n` in lower-case Roman numerals, for 1 to 3999; larger numbers are left in Arabic.
pub fn roman(n: u32) -> String {
    if n == 0 || n > 3999 {
        return n.to_string();
    }
    const STEPS: [(u32, &str); 13] = [
        (1000, "m"),
        (900, "cm"),
        (500, "d"),
        (400, "cd"),
        (100, "c"),
        (90, "xc"),
        (50, "l"),
        (40, "xl"),
        (10, "x"),
        (9, "ix"),
        (5, "v"),
        (4, "iv"),
        (1, "i"),
    ];
    let mut rest = n;
    let mut out = String::new();
    for (value, symbol) in STEPS {
        while rest >= value {
            out.push_str(symbol);
            rest -= value;
        }
    }
    out
}

/// The label of every page of every spread, in order. The cover is not counted and has none.
/// The TypeScript `pageLabels` is its twin, and both are tested against `labelCases.json`.
pub fn page_labels(spreads: &[Spread]) -> Vec<Vec<PageLabel>> {
    let mut count: u32 = 0;
    let mut current: Option<&Section> = None;
    spreads
        .iter()
        .map(|spread| {
            if spread.kind == SpreadKind::Cover {
                return spread
                    .pages
                    .iter()
                    .map(|_| PageLabel {
                        label: String::new(),
                        section: String::new(),
                    })
                    .collect();
            }
            if let Some(section) = &spread.section {
                current = Some(section);
                if let Some(start) = section.start {
                    count = start - 1;
                }
            }
            spread
                .pages
                .iter()
                .map(|_| {
                    count += 1;
                    let numbering = current.map_or(Numbering::Arabic, |s| s.numbering);
                    PageLabel {
                        label: match numbering {
                            Numbering::Arabic => count.to_string(),
                            Numbering::Roman => roman(count),
                            Numbering::None => String::new(),
                        },
                        section: current.map(|s| s.name.clone()).unwrap_or_default(),
                    }
                })
                .collect()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roman_numerals_are_lower_case_and_stop_at_3999() {
        for (n, want) in [
            (1, "i"),
            (4, "iv"),
            (9, "ix"),
            (14, "xiv"),
            (1994, "mcmxciv"),
            (3999, "mmmcmxcix"),
        ] {
            assert_eq!(roman(n), want);
        }
        assert_eq!(roman(4000), "4000");
        assert_eq!(roman(0), "0");
    }

    #[test]
    fn a_section_is_trimmed_and_kept_within_range() {
        let s = |name: &str, start| Section {
            name: name.into(),
            numbering: Numbering::Roman,
            start,
        };
        assert_eq!(
            s("  Contents ", Some(1)).checked().unwrap().name,
            "Contents"
        );
        assert!(s(&"x".repeat(81), None).checked().is_err());
        assert!(s("a", Some(0)).checked().is_err());
        assert!(s("a", Some(100_000)).checked().is_err());
    }
}
