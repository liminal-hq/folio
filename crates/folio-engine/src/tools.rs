// The Book tools: reports about an open book, worked out in Rust and shown by the tools window
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! A tool asks with a [`ToolRequest`] and is answered with a [`ToolReport`]. Both are typed, so the
//! window, an agent (`book_report` in `folio-mcp`) and the tests all read the same thing, and a new
//! tool adds a variant to each rather than a new command. Reports are pure functions of the book
//! and what is known of its pictures; changing the book is the tool's own intents, sent the way any
//! other change is, so every fix is one undo step (D-49).

use crate::punctuation::{self, PunctuationRule};
use folio_layout::assets::{AssetFact, AssetFacts, ERROR_PPI, WARN_PPI};
use folio_layout::preflight::picture_ppi;
use folio_model::{Book, Fit, Frame};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// What a tool is asked for.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(
    tag = "tool",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub enum ToolRequest {
    /// Every picture in the book, how sharp it is where it is set, and what it says it shows.
    Pictures,
    /// What the book still lacks before it can be sent to a store.
    Checklist,
    /// Where straight quotes, double hyphens, three dots and double spaces are in the words.
    Punctuation,
    /// Text boxes whose size, alignment, weight or typeface differ from their style's.
    Styles,
    /// What a copy costs to print and what is earned on a sale, at the prices given. A cost left out
    /// is the dated estimate (`PrintCosts::estimate`).
    Royalty {
        hardcover: bool,
        colour: bool,
        /// The list price of the printed book, and of the ebook, in dollars.
        list_price: f64,
        ebook_price: f64,
        fixed_cost: Option<f64>,
        per_page_cost: Option<f64>,
    },
}

/// What a tool found.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(
    tag = "tool",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub enum ToolReport {
    Pictures {
        rows: Vec<PictureRow>,
    },
    Checklist {
        items: Vec<ChecklistItem>,
    },
    Punctuation {
        found: Vec<PunctuationFound>,
    },
    Styles {
        frames: Vec<StyleStray>,
    },
    Royalty {
        pages: u32,
        /// What one printed copy costs, and the figures that made it.
        print_cost: f64,
        costs: PrintCosts,
        /// What is earned on a printed copy sold through each way of selling it.
        sales: Vec<SaleRoyalty>,
        /// The least list price that does not lose money on a copy sold on Amazon.com.
        minimum_price: f64,
        /// What is earned on an ebook at the price given.
        ebook_royalty: f64,
    },
}

/// A thing the punctuation tool would tidy, how many times, and a few examples of it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub struct PunctuationFound {
    pub rule: PunctuationRule,
    pub count: u32,
    pub examples: Vec<PunctuationExample>,
}

/// Some words as they are, and as they would be.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub struct PunctuationExample {
    pub before: String,
    pub after: String,
}

/// A text box that sets its words differently from the style it names.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub struct StyleStray {
    pub spread: String,
    pub frame: String,
    pub style: String,
    /// What it changes, such as `size`, `align`, `bold`, `italic` or `font`.
    pub changes: Vec<String>,
}

/// A change a tool makes to the book, all of it one undo step.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(
    tag = "tool",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub enum ToolFix {
    /// Tidies the punctuation in every story and text box, by the rules given.
    Punctuation { rules: Vec<PunctuationRule> },
    /// Puts text boxes back to their style's size, alignment, weight and typeface.
    ResetStyles { frames: Vec<String> },
}

/// What a fix did.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub struct ToolFixed {
    /// How many paragraphs or boxes were changed; none when there was nothing to do.
    pub changed: u32,
}

/// What a store form asks for, and whether the book has it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub enum ChecklistId {
    Title,
    Author,
    Description,
    Language,
    Keywords,
    Isbn,
    PageCount,
    Cover,
    Problems,
    Pictures,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub enum ChecklistStatus {
    Done,
    /// Not there yet, and a store will ask for it.
    Todo,
    /// There, but a store may not like it.
    Warning,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub struct ChecklistItem {
    pub id: ChecklistId,
    pub status: ChecklistStatus,
    /// What the book has, to show and to copy into the form; a count for the ones that count.
    pub value: Option<String>,
}

/// What printing a copy costs. KDP's own pages disagree with each other and change, so these are an
/// estimate with the date it was made, which the person can replace with the figures KDP's calculator
/// gives them.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub struct PrintCosts {
    /// A cost on every copy, and one for every page, in dollars.
    pub fixed: f64,
    pub per_page: f64,
    /// When the estimate was made. ⚠ Check KDP's calculator before relying on it.
    pub as_of: String,
}

impl PrintCosts {
    /// The estimate Folio starts from. ⚠ Unverified against KDP's current pages (CLAUDE.md): they
    /// disagree about the fixed cost for black ink, so the person is asked to confirm in KDP's
    /// calculator, and can enter its figures in place of these.
    pub fn estimate(hardcover: bool, colour: bool) -> Self {
        let per_page = if colour { 0.07 } else { 0.012 };
        Self {
            fixed: if hardcover { 6.80 } else { 1.00 },
            per_page,
            as_of: "2026-10-21".to_owned(),
        }
    }
}

/// What is earned on a printed copy sold one way.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub struct SaleRoyalty {
    pub market: Market,
    pub royalty: f64,
}

/// The ways of selling a printed copy the calculator shows. Royalty rates ⚠ are KDP's 60% on Amazon
/// and 40% on expanded distribution, from KDP's help pages; check them before a release.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub enum Market {
    AmazonCom,
    ExpandedDistribution,
}

/// What the checklist needs that only the app can know: how many problems layout found.
#[derive(Debug, Clone, Copy, Default)]
pub struct Findings {
    /// Problems that stop a print (preflight errors that are not left alone).
    pub errors: usize,
}

/// How a picture stands for print.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub enum PictureStatus {
    /// Sharp enough where it is set.
    Good,
    /// Under 300 pixels per inch, which a printer may show.
    Low,
    /// Under 150 pixels per inch, which will look soft.
    VeryLow,
    /// A drawing, as sharp at any size.
    Vector,
    /// The frame names a picture that is not in the book.
    Missing,
    /// The picture is there, but its size could not be read.
    Unreadable,
    /// A picture frame with no picture put in it yet.
    Empty,
}

/// One picture frame of the book.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub struct PictureRow {
    pub spread: String,
    pub frame: String,
    /// The picture's place in the package, when one is in the frame.
    pub asset: Option<String>,
    /// Its size in pixels, when it is a picture that has one.
    pub width: Option<u32>,
    pub height: Option<u32>,
    /// Pixels per inch where it is set, when it has pixels.
    pub ppi: Option<f64>,
    pub fit: Fit,
    /// What the picture says it shows, for a reader who cannot see it.
    pub description: Option<String>,
    pub status: PictureStatus,
}

/// Answers `request` for `book`, given what is known of its pictures.
pub fn run(
    book: &Book,
    facts: &AssetFacts,
    findings: Findings,
    request: &ToolRequest,
) -> ToolReport {
    match request {
        ToolRequest::Pictures => ToolReport::Pictures {
            rows: pictures(book, facts),
        },
        ToolRequest::Checklist => ToolReport::Checklist {
            items: checklist(book, facts, findings),
        },
        ToolRequest::Punctuation => ToolReport::Punctuation {
            found: punctuation_found(book),
        },
        ToolRequest::Styles => ToolReport::Styles {
            frames: style_strays(book),
        },
        ToolRequest::Royalty {
            hardcover,
            colour,
            list_price,
            ebook_price,
            fixed_cost,
            per_page_cost,
        } => royalty(
            book,
            *hardcover,
            *colour,
            *list_price,
            *ebook_price,
            *fixed_cost,
            *per_page_cost,
        ),
    }
}

/// The words a tool may change: each paragraph of a story, and each text box or shape that holds
/// its own words.
enum Source {
    Paragraph {
        flow: folio_model::FlowId,
        para: usize,
        text: String,
    },
    Frame {
        frame: folio_model::FrameId,
        text: String,
    },
}

fn sources(book: &Book) -> Vec<Source> {
    let mut out = Vec::new();
    for flow in &book.flows {
        for (para, p) in flow.paragraphs.iter().enumerate() {
            out.push(Source::Paragraph {
                flow: flow.id.clone(),
                para,
                text: folio_model::text_of(&p.inline),
            });
        }
    }
    for spread in &book.spreads {
        for frame in &spread.frames {
            match frame {
                Frame::Text(t) if t.flow.is_none() => {
                    if let Some(content) = &t.content {
                        out.push(Source::Frame {
                            frame: t.base.id.clone(),
                            text: folio_model::text_of(&content.inline),
                        });
                    }
                }
                Frame::Shape(s) => {
                    if let Some(content) = &s.content {
                        out.push(Source::Frame {
                            frame: s.base.id.clone(),
                            text: folio_model::text_of(&content.inline),
                        });
                    }
                }
                _ => {}
            }
        }
    }
    out
}

fn punctuation_found(book: &Book) -> Vec<PunctuationFound> {
    let all = sources(book);
    punctuation::RULES
        .iter()
        .filter_map(|&rule| {
            let mut count = 0;
            let mut examples = Vec::new();
            for source in &all {
                let text = match source {
                    Source::Paragraph { text, .. } | Source::Frame { text, .. } => text,
                };
                let (_, n) = punctuation::apply(text, rule);
                count += n as u32;
                if n > 0 && examples.len() < 3 {
                    if let Some((before, after)) = punctuation::example(text, rule) {
                        examples.push(PunctuationExample { before, after });
                    }
                }
            }
            (count > 0).then_some(PunctuationFound {
                rule,
                count,
                examples,
            })
        })
        .collect()
}

fn style_strays(book: &Book) -> Vec<StyleStray> {
    let mut out = Vec::new();
    for spread in &book.spreads {
        for frame in &spread.frames {
            let (base, style, overrides) = match frame {
                Frame::Text(t) => (&t.base, t.style.0.as_str(), t.overrides.as_ref()),
                Frame::Shape(s) => (
                    &s.base,
                    s.style.as_ref().map_or("", |s| s.0.as_str()),
                    s.overrides.as_ref(),
                ),
                _ => continue,
            };
            let Some(o) = overrides else { continue };
            let mut changes = Vec::new();
            if o.size.is_some() {
                changes.push("size".to_owned());
            }
            if o.align.is_some() {
                changes.push("align".to_owned());
            }
            if o.bold.is_some() {
                changes.push("bold".to_owned());
            }
            if o.italic.is_some() {
                changes.push("italic".to_owned());
            }
            if o.font.is_some() {
                changes.push("font".to_owned());
            }
            if !changes.is_empty() {
                out.push(StyleStray {
                    spread: spread.id.0.clone(),
                    frame: base.id.0.clone(),
                    style: style.to_owned(),
                    changes,
                });
            }
        }
    }
    out
}

/// The intents that make `fix` happen, in order; none when there is nothing to change.
pub fn fix_intents(book: &Book, fix: &ToolFix) -> Vec<folio_model::Intent> {
    use folio_model::Intent;
    match fix {
        ToolFix::Punctuation { rules } => sources(book)
            .into_iter()
            .filter_map(|source| match source {
                Source::Paragraph { flow, para, text } => {
                    let new = punctuation::smarten(&text, rules);
                    (new != text).then_some(Intent::SetParagraphText {
                        flow,
                        para,
                        text: new,
                    })
                }
                Source::Frame { frame, text } => {
                    let new = punctuation::smarten(&text, rules);
                    (new != text).then_some(Intent::SetText { frame, text: new })
                }
            })
            .collect(),
        ToolFix::ResetStyles { frames } => style_strays(book)
            .into_iter()
            .filter(|stray| frames.contains(&stray.frame))
            .map(|stray| Intent::OverrideStyle {
                frame: folio_model::FrameId(stray.frame),
                overrides: folio_model::StyleOverrides::default(),
            })
            .collect(),
    }
}

/// KDP's smallest and largest printed book, in pages. ⚠ Check against KDP's pages before a release.
const KDP_MIN_PAGES: u32 = 24;
const KDP_MAX_PAGES: u32 = 828;

fn checklist(book: &Book, facts: &AssetFacts, findings: Findings) -> Vec<ChecklistItem> {
    use ChecklistId::*;
    use ChecklistStatus::*;
    let item = |id, ok: bool, value: Option<String>| ChecklistItem {
        id,
        status: if ok { Done } else { Todo },
        value,
    };
    let text = |s: &str| (!s.trim().is_empty()).then(|| s.trim().to_owned());
    let authors = book
        .authors
        .iter()
        .map(|a| a.name.trim())
        .filter(|n| !n.is_empty())
        .collect::<Vec<_>>()
        .join(", ");
    let keywords = book.metadata.keywords.len();
    let pages = folio_model::spine_pages(book);
    let isbn = book
        .metadata
        .isbn
        .paperback
        .clone()
        .filter(|i| !i.trim().is_empty());
    let isbn_ok = isbn.as_deref().is_some_and(folio_model::is_valid_isbn13);
    let missing = pictures(book, facts)
        .iter()
        .filter(|row| {
            matches!(
                row.status,
                PictureStatus::Missing | PictureStatus::Unreadable
            )
        })
        .count();
    let mut items = vec![
        item(Title, text(&book.title).is_some(), text(&book.title)),
        item(Author, !authors.is_empty(), text(&authors)),
        item(
            Description,
            text(&book.metadata.description).is_some(),
            text(&book.metadata.description),
        ),
        item(
            Language,
            text(&book.language).is_some(),
            text(&book.language),
        ),
        ChecklistItem {
            id: Keywords,
            status: match keywords {
                0 => Todo,
                1..=6 => Warning,
                _ => Done,
            },
            value: Some(book.metadata.keywords.join(", ")).filter(|k| !k.is_empty()),
        },
        ChecklistItem {
            id: Isbn,
            status: match (&isbn, isbn_ok) {
                (None, _) => Warning,
                (Some(_), true) => Done,
                (Some(_), false) => Todo,
            },
            value: isbn,
        },
    ];
    items.push(ChecklistItem {
        id: PageCount,
        status: if (KDP_MIN_PAGES..=KDP_MAX_PAGES).contains(&pages) {
            Done
        } else {
            Todo
        },
        value: Some(pages.to_string()),
    });
    items.push(item(
        Cover,
        book.spreads
            .first()
            .is_some_and(|s| s.kind == folio_model::SpreadKind::Cover),
        None,
    ));
    items.push(ChecklistItem {
        id: Problems,
        status: if findings.errors == 0 { Done } else { Todo },
        value: Some(findings.errors.to_string()),
    });
    items.push(ChecklistItem {
        id: Pictures,
        status: if missing == 0 { Done } else { Todo },
        value: Some(missing.to_string()),
    });
    items
}

fn royalty(
    book: &Book,
    hardcover: bool,
    colour: bool,
    list_price: f64,
    ebook_price: f64,
    fixed_cost: Option<f64>,
    per_page_cost: Option<f64>,
) -> ToolReport {
    let pages = folio_model::spine_pages(book);
    let mut costs = PrintCosts::estimate(hardcover, colour);
    if let Some(fixed) = fixed_cost.filter(|c| c.is_finite() && *c >= 0.0) {
        costs.fixed = fixed;
    }
    if let Some(per_page) = per_page_cost.filter(|c| c.is_finite() && *c >= 0.0) {
        costs.per_page = per_page;
    }
    let print_cost = costs.fixed + f64::from(pages) * costs.per_page;
    let list = if list_price.is_finite() {
        list_price.max(0.0)
    } else {
        0.0
    };
    let earned = |rate: f64| (list * rate - print_cost).max(0.0);
    let ebook = if ebook_price.is_finite() {
        ebook_price.max(0.0)
    } else {
        0.0
    };
    // ⚠ KDP's ebook bands: 70% from $2.99 to $9.99 less a delivery fee, otherwise 35%.
    let ebook_royalty = if (2.99..=9.99).contains(&ebook) {
        (ebook * 0.70 - 0.15).max(0.0)
    } else {
        ebook * 0.35
    };
    ToolReport::Royalty {
        pages,
        print_cost,
        costs,
        sales: vec![
            SaleRoyalty {
                market: Market::AmazonCom,
                royalty: earned(0.60),
            },
            SaleRoyalty {
                market: Market::ExpandedDistribution,
                royalty: earned(0.40),
            },
        ],
        minimum_price: print_cost / 0.60,
        ebook_royalty,
    }
}

fn pictures(book: &Book, facts: &AssetFacts) -> Vec<PictureRow> {
    let mut rows = Vec::new();
    for spread in &book.spreads {
        for frame in &spread.frames {
            let Frame::Image(image) = frame else {
                continue;
            };
            let r = &image.base.rect;
            let mut row = PictureRow {
                spread: spread.id.0.clone(),
                frame: image.base.id.0.clone(),
                asset: image.image.asset.clone(),
                width: None,
                height: None,
                ppi: None,
                fit: image.image.fit,
                description: image.base.description.clone(),
                status: PictureStatus::Empty,
            };
            if let Some(asset) = &image.image.asset {
                row.status = match facts.0.get(asset) {
                    None => PictureStatus::Missing,
                    Some(AssetFact::Vector) => PictureStatus::Vector,
                    Some(AssetFact::Unreadable) => PictureStatus::Unreadable,
                    Some(AssetFact::Raster { w, h }) => {
                        let ppi = picture_ppi((*w, *h), r.w.0, r.h.0, image.image.fit);
                        row.width = Some(*w);
                        row.height = Some(*h);
                        row.ppi = Some(ppi);
                        if ppi < ERROR_PPI {
                            PictureStatus::VeryLow
                        } else if ppi < WARN_PPI {
                            PictureStatus::Low
                        } else {
                            PictureStatus::Good
                        }
                    }
                };
            }
            rows.push(row);
        }
    }
    rows
}

#[cfg(test)]
mod tests {
    use super::*;
    use folio_model::{
        Book, FrameBase, FrameId, ImageFrame, ImageRef, Page, PageId, Pt, Rect, Spread, SpreadId,
        SpreadKind,
    };
    use std::collections::HashMap;

    fn book_with(frames: Vec<Frame>) -> Book {
        let made = folio_presets::new_book(&folio_presets::NewBookSpec::default()).unwrap();
        let mut book = made.book;
        book.spreads = vec![Spread {
            id: SpreadId("sp".into()),
            kind: SpreadKind::Spread,
            pages: vec![Page {
                id: PageId("pg".into()),
                master: None,
                bg: None,
            }],
            frames,
            guides: Vec::new(),
            master_overrides: Vec::new(),
            bg_span: None,
            section: None,
        }];
        book
    }

    fn picture(id: &str, asset: Option<&str>, w: f64, h: f64, alt: Option<&str>) -> Frame {
        Frame::Image(ImageFrame {
            base: FrameBase {
                id: FrameId(id.into()),
                rect: Rect {
                    x: Pt(0.0),
                    y: Pt(0.0),
                    w: Pt(w),
                    h: Pt(h),
                },
                rotation: 0.0,
                locked: false,
                from_master: None,
                description: alt.map(str::to_owned),
            },
            image: ImageRef {
                asset: asset.map(str::to_owned),
                fit: Fit::Fill,
                focus: (0.5, 0.5),
                crop: None,
            },
            placeholder_label: None,
        })
    }

    #[test]
    fn every_picture_frame_is_a_row_with_how_sharp_it_is_where_it_is_set() {
        let book = book_with(vec![
            // 3000 px across 360 pt (5 in) is 600 pixels per inch.
            picture("a", Some("assets/a.png"), 360.0, 360.0, Some("An otter")),
            // 900 px across 360 pt is 180: low.
            picture("b", Some("assets/b.png"), 360.0, 360.0, None),
            // 300 px across 360 pt is 60: very low.
            picture("c", Some("assets/c.png"), 360.0, 360.0, None),
            picture("d", Some("assets/gone.png"), 100.0, 100.0, None),
            picture("e", Some("assets/e.svg"), 100.0, 100.0, None),
            picture("f", Some("assets/f.bin"), 100.0, 100.0, None),
            picture("g", None, 100.0, 100.0, None),
        ]);
        let facts = AssetFacts(HashMap::from([
            (
                "assets/a.png".into(),
                AssetFact::Raster { w: 3000, h: 3000 },
            ),
            ("assets/b.png".into(), AssetFact::Raster { w: 900, h: 900 }),
            ("assets/c.png".into(), AssetFact::Raster { w: 300, h: 300 }),
            ("assets/e.svg".into(), AssetFact::Vector),
            ("assets/f.bin".into(), AssetFact::Unreadable),
        ]));
        let ToolReport::Pictures { rows } =
            run(&book, &facts, Findings::default(), &ToolRequest::Pictures)
        else {
            panic!("a pictures report")
        };
        let status: Vec<(&str, PictureStatus)> =
            rows.iter().map(|r| (r.frame.as_str(), r.status)).collect();
        assert_eq!(
            status,
            vec![
                ("a", PictureStatus::Good),
                ("b", PictureStatus::Low),
                ("c", PictureStatus::VeryLow),
                ("d", PictureStatus::Missing),
                ("e", PictureStatus::Vector),
                ("f", PictureStatus::Unreadable),
                ("g", PictureStatus::Empty),
            ]
        );
        assert!((rows[0].ppi.unwrap() - 600.0).abs() < 1e-9);
        assert_eq!(rows[0].description.as_deref(), Some("An otter"));
        assert_eq!(rows[3].ppi, None);
    }
}

#[cfg(test)]
mod publishing_tests {
    use super::*;
    use folio_model::Contributor;

    fn book() -> Book {
        folio_presets::new_book(&folio_presets::NewBookSpec::default())
            .unwrap()
            .book
    }

    fn status(items: &[ChecklistItem], id: ChecklistId) -> ChecklistStatus {
        items.iter().find(|i| i.id == id).unwrap().status
    }

    #[test]
    fn the_checklist_says_what_a_store_will_still_ask_for() {
        let mut book = book();
        book.title = "Pip".into();
        book.authors = vec![Contributor {
            name: "Mom".into(),
            role: folio_model::ContributorRole::Author,
        }];
        book.metadata.description = String::new();
        book.metadata.keywords = vec!["otter".into(), "bedtime".into()];
        book.metadata.isbn.paperback = Some("978-0-306-40615-8".into());
        let facts = AssetFacts::default();
        let ToolReport::Checklist { items } = run(
            &book,
            &facts,
            Findings { errors: 2 },
            &ToolRequest::Checklist,
        ) else {
            panic!("a checklist")
        };
        use ChecklistId::*;
        use ChecklistStatus::*;
        assert_eq!(status(&items, Title), Done);
        assert_eq!(status(&items, Author), Done);
        assert_eq!(status(&items, Description), Todo);
        assert_eq!(status(&items, Keywords), Warning, "fewer than seven");
        assert_eq!(status(&items, Isbn), Todo, "a bad check digit");
        assert_eq!(status(&items, Problems), Todo);
        assert_eq!(status(&items, Cover), Done);
        // A good ISBN, seven keywords and no problems are done.
        book.metadata.isbn.paperback = Some("978-0-306-40615-7".into());
        book.metadata.keywords = (1..=7).map(|n| format!("keyword {n}")).collect();
        let ToolReport::Checklist { items } =
            run(&book, &facts, Findings::default(), &ToolRequest::Checklist)
        else {
            panic!("a checklist")
        };
        assert_eq!(status(&items, Isbn), Done);
        assert_eq!(status(&items, Keywords), Done);
        assert_eq!(status(&items, Problems), Done);
    }

    #[test]
    fn a_royalty_is_the_share_of_the_price_less_the_printing_cost_and_never_below_nothing() {
        let book = book();
        let ask = |list: f64, ebook: f64, fixed: Option<f64>| {
            run(
                &book,
                &AssetFacts::default(),
                Findings::default(),
                &ToolRequest::Royalty {
                    hardcover: false,
                    colour: false,
                    list_price: list,
                    ebook_price: ebook,
                    fixed_cost: fixed,
                    per_page_cost: None,
                },
            )
        };
        let ToolReport::Royalty {
            pages,
            print_cost,
            sales,
            minimum_price,
            ebook_royalty,
            ..
        } = ask(10.0, 4.99, None)
        else {
            panic!("a royalty")
        };
        // The estimate: 1.00 and 0.012 a page, and a book is at least 24 pages.
        assert_eq!(pages, 24);
        assert!((print_cost - (1.0 + 24.0 * 0.012)).abs() < 1e-9);
        assert!((sales[0].royalty - (6.0 - print_cost)).abs() < 1e-9);
        assert!((sales[1].royalty - (4.0 - print_cost)).abs() < 1e-9);
        assert!((minimum_price - print_cost / 0.6).abs() < 1e-9);
        // The 70% band, less delivery, and the 35% one outside it.
        assert!((ebook_royalty - (4.99 * 0.7 - 0.15)).abs() < 1e-9);
        let ToolReport::Royalty { ebook_royalty, .. } = ask(10.0, 0.99, None) else {
            panic!("a royalty")
        };
        assert!((ebook_royalty - 0.99 * 0.35).abs() < 1e-9);
        // A price below the cost earns nothing, and the person's own cost replaces the estimate.
        let ToolReport::Royalty { sales, costs, .. } = ask(1.0, 0.0, Some(2.30)) else {
            panic!("a royalty")
        };
        assert_eq!(sales[0].royalty, 0.0);
        assert_eq!(costs.fixed, 2.30);
    }
}
