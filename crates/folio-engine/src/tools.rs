// The Book tools: reports about an open book, worked out in Rust and shown by the tools window
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! A tool asks with a [`ToolRequest`] and is answered with a [`ToolReport`]. Both are typed, so the
//! window, an agent (`book_report` in `folio-mcp`) and the tests all read the same thing, and a new
//! tool adds a variant to each rather than a new command. Reports are pure functions of the book
//! and what is known of its pictures; changing the book is the tool's own intents, sent the way any
//! other change is, so every fix is one undo step (D-49).

use folio_layout::assets::{AssetFact, AssetFacts, ERROR_PPI, WARN_PPI};
use folio_layout::preflight::picture_ppi;
use folio_model::{Book, Fit, Frame};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// What a tool is asked for.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(tag = "tool", rename_all = "camelCase")]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub enum ToolRequest {
    /// Every picture in the book, how sharp it is where it is set, and what it says it shows.
    Pictures,
}

/// What a tool found.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "tool", rename_all = "camelCase")]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub enum ToolReport {
    Pictures { rows: Vec<PictureRow> },
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
pub fn run(book: &Book, facts: &AssetFacts, request: &ToolRequest) -> ToolReport {
    match request {
        ToolRequest::Pictures => ToolReport::Pictures {
            rows: pictures(book, facts),
        },
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
        let ToolReport::Pictures { rows } = run(&book, &facts, &ToolRequest::Pictures);
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
