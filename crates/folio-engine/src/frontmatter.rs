// The front matter the Book tools make: the copyright page and the contents
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Both are ordinary frames on an ordinary spread, so they are edited like any other and the tools
//! only refresh them. A frame the tool made has a fixed id (`copyright`, `toc_heading`,
//! `toc_titles`, `toc_pages`), which is how the next run finds it: the words in it are replaced and
//! nothing else is touched, so a frame the person has moved stays where they put it.

use folio_model::{
    Align, Book, Frame, FrameBase, FrameId, InlineContent, Intent, Pt, Rect, Run, SpreadId,
    SpreadKind, StyleId, StyleOverrides, TextFrame,
};
use folio_presets::templates::Metrics;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// What goes on a copyright page, besides what the book already says about itself.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub struct CopyrightFields {
    pub year: u32,
    /// Who holds the copyright.
    pub holder: String,
    /// The rights line, such as `All rights reserved.`
    pub rights: String,
    /// Which printing this is, such as `First edition`.
    pub printing: String,
}

/// One line of the contents: a chapter's title and the page it begins on.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub struct ContentsEntry {
    pub title: String,
    pub page: String,
}

const COPYRIGHT: &str = "copyright";
const HEADING: &str = "toc_heading";
const TITLES: &str = "toc_titles";
const PAGES: &str = "toc_pages";

/// The words of the copyright page, a line for each thing that is known.
pub fn copyright_text(book: &Book, fields: &CopyrightFields) -> String {
    let mut lines = Vec::new();
    let holder = fields.holder.trim();
    if !holder.is_empty() {
        lines.push(format!("Copyright © {} {holder}", fields.year));
    }
    if !fields.rights.trim().is_empty() {
        lines.push(fields.rights.trim().to_owned());
    }
    if !lines.is_empty() {
        lines.push(String::new());
    }
    if !book.title.trim().is_empty() {
        lines.push(book.title.trim().to_owned());
    }
    if let Some(isbn) = book
        .metadata
        .isbn
        .paperback
        .as_deref()
        .filter(|i| !i.trim().is_empty())
    {
        lines.push(format!("ISBN {}", isbn.trim()));
    }
    if !fields.printing.trim().is_empty() {
        lines.push(fields.printing.trim().to_owned());
    }
    lines.join("\n")
}

fn find<'a>(book: &'a Book, id: &str) -> Option<(&'a SpreadId, &'a Frame)> {
    book.spreads.iter().find_map(|spread| {
        spread
            .frames
            .iter()
            .find(|f| f.base().id.0 == id)
            .map(|f| (&spread.id, f))
    })
}

fn pt(inches: f64) -> Pt {
    Pt::from_in(inches)
}

fn text_frame(
    id: &str,
    rect: (f64, f64, f64, f64),
    style: &str,
    text: &str,
    overrides: Option<StyleOverrides>,
) -> Frame {
    Frame::Text(TextFrame {
        base: FrameBase {
            id: FrameId(id.to_owned()),
            rect: Rect {
                x: pt(rect.0),
                y: pt(rect.1),
                w: pt(rect.2),
                h: pt(rect.3),
            },
            rotation: 0.0,
            locked: false,
            from_master: None,
            description: None,
        },
        style: StyleId(style.to_owned()),
        content: Some(InlineContent {
            inline: vec![Run {
                text: text.to_owned(),
                style: None,
                bold: false,
                italic: false,
                font: None,
                colour: None,
            }],
        }),
        flow: None,
        columns: 1,
        baseline_lock: false,
        overrides,
    })
}

/// The spread a new front-matter page goes after: the opener, which is the title page, or the page
/// the person put the copyright on. `None` when the book is only a cover.
fn after_front(book: &Book, copyright_on: Option<&SpreadId>) -> Option<SpreadId> {
    if let Some(on) = copyright_on {
        return Some(on.clone());
    }
    book.spreads
        .iter()
        .find(|s| s.kind != SpreadKind::Cover)
        .map(|s| s.id.clone())
}

/// What makes the copyright page: its words put in the frame the tool made before, or a new spread
/// after the title page with the frame on its left page, which is the back of the title page.
pub fn copyright_intents(book: &Book, fields: &CopyrightFields, k: &Metrics) -> Vec<Intent> {
    let text = copyright_text(book, fields);
    if let Some((_, frame)) = find(book, COPYRIGHT) {
        return vec![Intent::SetText {
            frame: frame.base().id.clone(),
            text,
        }];
    }
    let Some(after) = after_front(book, None) else {
        return Vec::new();
    };
    let small = StyleOverrides {
        size: Some(Pt(8.5)),
        align: Some(Align::Left),
        ..StyleOverrides::default()
    };
    vec![Intent::AddSpreadWith {
        after,
        frames: vec![text_frame(
            COPYRIGHT,
            (k.m, k.h * 0.5, k.w - 2.0 * k.m, k.h * 0.5 - k.m),
            "bodyL",
            &text,
            Some(small),
        )],
    }]
}

/// What makes the contents: the words put in the three frames the tool made before, or a new spread
/// after the copyright page (or the title page) with the heading, titles and page numbers on its
/// right page.
pub fn contents_intents(book: &Book, entries: &[ContentsEntry], k: &Metrics) -> Vec<Intent> {
    let titles = entries
        .iter()
        .map(|e| e.title.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    let pages = entries
        .iter()
        .map(|e| e.page.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    let heading = find(book, HEADING);
    let on_titles = find(book, TITLES);
    let on_pages = find(book, PAGES);
    if let (Some((_, h)), Some((_, t)), Some((_, p))) = (heading, on_titles, on_pages) {
        return vec![
            Intent::SetText {
                frame: h.base().id.clone(),
                text: "Contents".into(),
            },
            Intent::SetText {
                frame: t.base().id.clone(),
                text: titles,
            },
            Intent::SetText {
                frame: p.base().id.clone(),
                text: pages,
            },
        ];
    }
    let copyright_on = find(book, COPYRIGHT).map(|(spread, _)| spread);
    let Some(after) = after_front(book, copyright_on) else {
        return Vec::new();
    };
    let width = k.w - 2.0 * k.m;
    let numbers = width * 0.18;
    let top = k.m + 1.0;
    let right = StyleOverrides {
        align: Some(Align::Right),
        ..StyleOverrides::default()
    };
    vec![Intent::AddSpreadWith {
        after,
        frames: vec![
            text_frame(
                HEADING,
                (k.w + k.m, k.m, width, 0.8),
                "h1",
                "Contents",
                None,
            ),
            text_frame(
                TITLES,
                (k.w + k.m, top, width - numbers - 0.2, k.h - top - k.m),
                "body",
                &titles,
                Some(StyleOverrides {
                    align: Some(Align::Left),
                    ..StyleOverrides::default()
                }),
            ),
            text_frame(
                PAGES,
                (k.w + k.m + width - numbers, top, numbers, k.h - top - k.m),
                "body",
                &pages,
                Some(right),
            ),
        ],
    }]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn book() -> Book {
        folio_presets::new_book(&folio_presets::NewBookSpec::default())
            .unwrap()
            .book
    }

    fn fields() -> CopyrightFields {
        CopyrightFields {
            year: 2026,
            holder: "Jane L. Morris".into(),
            rights: "All rights reserved.".into(),
            printing: "First edition".into(),
        }
    }

    const K: Metrics = Metrics {
        w: 8.5,
        h: 8.5,
        m: 0.5,
        b: 0.125,
    };

    #[test]
    fn the_copyright_text_has_a_line_for_each_thing_that_is_known() {
        let mut book = book();
        book.title = "A Hallowe'en Tail".into();
        book.metadata.isbn.paperback = Some("978-0-306-40615-7".into());
        assert_eq!(
            copyright_text(&book, &fields()),
            "Copyright © 2026 Jane L. Morris\nAll rights reserved.\n\nA Hallowe'en Tail\nISBN 978-0-306-40615-7\nFirst edition"
        );
        // No holder, no rights and no ISBN leave only the title and the printing.
        book.metadata.isbn.paperback = None;
        let bare = CopyrightFields {
            holder: String::new(),
            rights: String::new(),
            ..fields()
        };
        assert_eq!(
            copyright_text(&book, &bare),
            "A Hallowe'en Tail\nFirst edition"
        );
    }

    #[test]
    fn the_copyright_page_is_added_after_the_title_page_and_refreshed_in_place_after_that() {
        let book = book();
        let title_page = book.spreads[1].id.clone();
        let first = copyright_intents(&book, &fields(), &K);
        let [Intent::AddSpreadWith { after, frames }] = first.as_slice() else {
            panic!("one new spread")
        };
        assert_eq!(*after, title_page);
        assert_eq!(frames[0].base().id.0, "copyright");
        // Once the page is there, the same words go in the same frame.
        let mut with = book.clone();
        with.spreads.insert(
            2,
            folio_model::Spread {
                id: SpreadId("sp_c".into()),
                kind: SpreadKind::Spread,
                pages: Vec::new(),
                frames: frames.clone(),
                guides: Vec::new(),
                master_overrides: Vec::new(),
                bg_span: None,
                section: None,
            },
        );
        let again = copyright_intents(&with, &fields(), &K);
        assert!(matches!(again.as_slice(), [Intent::SetText { .. }]));
    }

    #[test]
    fn the_contents_has_a_heading_and_the_titles_beside_their_pages() {
        let book = book();
        let entries = vec![
            ContentsEntry {
                title: "The pumpkin".into(),
                page: "3".into(),
            },
            ContentsEntry {
                title: "The cat".into(),
                page: "9".into(),
            },
        ];
        let made = contents_intents(&book, &entries, &K);
        let [Intent::AddSpreadWith { frames, .. }] = made.as_slice() else {
            panic!("one new spread")
        };
        let ids: Vec<&str> = frames.iter().map(|f| f.base().id.0.as_str()).collect();
        assert_eq!(ids, vec!["toc_heading", "toc_titles", "toc_pages"]);
    }
}
