// Preflight geometry: checks on where frames sit, from the model alone
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use folio_model::{Background, Book, Fit, Frame, Rect, Spread, SpreadKind};

use crate::assets::{effective_ppi, AssetFact, AssetFacts, ERROR_PPI, WARN_PPI};

use crate::wire::{PreflightItem, PreflightKind};

/// How close to the trim edge a picture's edge counts as touching it, in points (0.02 in).
const TOUCH_PT: f64 = 0.02 * 72.0;
const EPS: f64 = 1e-6;

/// A page's place on its spread, from the spread's origin (the left-most page's top-left trim).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PageRect {
    pub rect: Rect,
    /// Facing the gutter on the right, so the inside margin is on its left.
    pub recto: bool,
}

/// Where the pages of `spread` sit, in the spread's own space: the first page's top-left trim is
/// the origin, and the next page starts one trim width on. A page alone (an opener, a closer, a
/// single page or the cover, which is the wide printed sheet) starts at the origin whichever side of its spread it is on, so the
/// frames on it are placed from the page's own left edge and agree with the canvas, the masters
/// and the guides, which all count from there. An opener is a right-hand page and a closer a
/// left-hand one; that decides its margins and bleed, not where it is.
pub fn page_rects(book: &Book, spread: &Spread) -> Vec<PageRect> {
    let w = book.trim.w.0;
    let h = book.trim.h.0;
    let count = spread.pages.len();
    // The cover is the printed sheet: back, spine and front as one page.
    let cover = spread.kind == SpreadKind::Cover;
    let wide = if cover {
        folio_model::cover_width(book).0
    } else {
        w
    };
    (0..count)
        .map(|i| {
            let recto = match spread.kind {
                SpreadKind::Opener | SpreadKind::Single | SpreadKind::Cover => true,
                SpreadKind::Closer => false,
                SpreadKind::Spread => i == 1 || count == 1,
            };
            PageRect {
                rect: Rect {
                    x: folio_model::Pt(i as f64 * w),
                    y: folio_model::Pt(0.0),
                    w: folio_model::Pt(wide),
                    h: folio_model::Pt(h),
                },
                recto,
            }
        })
        .collect()
}

/// The area inside the margins of `page`.
fn safe_area(book: &Book, page: &PageRect) -> (f64, f64, f64, f64) {
    let m = &book.margins;
    let (left, right) = if page.recto {
        (m.inside.0, m.outside.0)
    } else {
        (m.outside.0, m.inside.0)
    };
    let r = &page.rect;
    (
        r.x.0 + left,
        r.y.0 + m.top.0,
        r.x.0 + r.w.0 - right,
        r.y.0 + r.h.0 - m.bottom.0,
    )
}

/// Frames whose rectangles break a print rule: text outside the safe margin and pictures that
/// touch the trim but stop short of the bleed. Covers are left alone (their layout is the wrap's).
pub fn geometry_checks(book: &Book, spread: &Spread) -> Vec<PreflightItem> {
    let mut out = Vec::new();
    if spread.kind == SpreadKind::Cover {
        return out;
    }
    let pages = page_rects(book, spread);
    let Some(first) = pages.first() else {
        return out;
    };
    let (x0, y0) = (first.rect.x.0, first.rect.y.0);
    let x1 = pages
        .iter()
        .map(|p| p.rect.x.0 + p.rect.w.0)
        .fold(x0, f64::max);
    let y1 = first.rect.y.0 + first.rect.h.0;
    let bleed = book.bleed.0;

    for frame in &spread.frames {
        let base = frame.base();
        if base.from_master.is_some() {
            continue;
        }
        // A turned frame is held to the box that holds its turned corners.
        let r = &base.rect.turned_bounds(base.rotation);
        let (fx0, fy0, fx1, fy1) = (r.x.0, r.y.0, r.x.0 + r.w.0, r.y.0 + r.h.0);
        match frame {
            Frame::Text(_) | Frame::Table(_) => {
                let inside = pages.iter().any(|page| {
                    let (sx0, sy0, sx1, sy1) = safe_area(book, page);
                    fx0 >= sx0 - EPS && fy0 >= sy0 - EPS && fx1 <= sx1 + EPS && fy1 <= sy1 + EPS
                });
                if !inside {
                    out.push(item(PreflightKind::TextOutsideMargin, spread, frame));
                }
            }
            Frame::Image(image) => {
                if image.image.asset.is_none() {
                    out.push(item(PreflightKind::PicturePlaceholder, spread, frame));
                } else if base
                    .description
                    .as_deref()
                    .is_none_or(|d| d.trim().is_empty())
                {
                    out.push(item(PreflightKind::AltMissing, spread, frame));
                }
                // Each outer edge that touches the trim must run on to the bleed line.
                // The gutter side of an opener or a closer is the fold, not a trim edge.
                let (left_is_trim, right_is_trim) = match spread.kind {
                    SpreadKind::Opener => (false, true),
                    SpreadKind::Closer => (true, false),
                    _ => (true, true),
                };
                let edges = [
                    (fx0, x0, !left_is_trim || fx0 <= x0 - bleed + EPS),
                    (fx1, x1, !right_is_trim || fx1 >= x1 + bleed - EPS),
                    (fy0, y0, fy0 <= y0 - bleed + EPS),
                    (fy1, y1, fy1 >= y1 + bleed - EPS),
                ];
                if bleed > 0.0
                    && edges
                        .iter()
                        .any(|&(edge, trim, reaches)| (edge - trim).abs() < TOUCH_PT && !reaches)
                {
                    out.push(item(PreflightKind::PictureShortOfBleed, spread, frame));
                }
            }
            _ => {}
        }
    }
    out
}

fn item(kind: PreflightKind, spread: &Spread, frame: &Frame) -> PreflightItem {
    PreflightItem::of_frame(kind, &spread.id, &frame.base().id)
}

/// How a picture of `natural` pixels set in a `w` × `h` point box with `fit` is sharp enough:
/// the problem with it, if it has one.
fn sharpness(natural: (u32, u32), w: f64, h: f64, fit: Fit) -> Option<PreflightKind> {
    let ppi = picture_ppi(natural, w, h, fit);
    if ppi < ERROR_PPI {
        Some(PreflightKind::VeryLowResolution)
    } else if ppi < WARN_PPI {
        Some(PreflightKind::LowResolution)
    } else {
        None
    }
}

/// How sharp a picture of `natural` pixels is, in pixels per inch, when it is set in a `w` × `h`
/// point box with `fit`.
pub fn picture_ppi(natural: (u32, u32), w: f64, h: f64, fit: Fit) -> f64 {
    let (nw, nh) = (f64::from(natural.0), f64::from(natural.1));
    // The points one pixel covers: a fill covers the box with the picture, a fit keeps it all inside,
    // and a stretch pulls each side to the box's.
    let drawn = match fit {
        Fit::Fit => {
            let scale = (w / nw).min(h / nh);
            (nw * scale, nh * scale)
        }
        Fit::Stretch => (w, h),
        Fit::Fill | Fit::Manual => {
            let scale = (w / nw).max(h / nh);
            (nw * scale, nh * scale)
        }
    };
    effective_ppi(natural, drawn.0, drawn.1)
}

/// The problems with the pictures of `spread`, in its picture frames and in its backgrounds: a
/// picture that is not in the book, one drawn too coarsely for print, and a background in a book
/// with no bleed. Covers are checked too, since a cover's picture is its background.
pub fn asset_checks(book: &Book, spread: &Spread, facts: &AssetFacts) -> Vec<PreflightItem> {
    let mut out = Vec::new();
    for frame in &spread.frames {
        let Frame::Image(image) = frame else {
            continue;
        };
        let Some(asset) = &image.image.asset else {
            continue;
        };
        let r = &image.base.rect;
        match facts.0.get(asset) {
            None => out.push(item(PreflightKind::PictureMissing, spread, frame)),
            Some(AssetFact::Raster { w, h }) => {
                if let Some(kind) = sharpness((*w, *h), r.w.0, r.h.0, image.image.fit) {
                    out.push(item(kind, spread, frame));
                }
            }
            Some(_) => {}
        }
    }
    let trim = (book.trim.w.0, book.trim.h.0);
    let bleed = book.bleed.0;
    let resolved = folio_model::resolve(&book.master_pages, spread);
    for (page, shown) in spread.pages.iter().zip(&resolved) {
        let Some(shown) = shown else {
            continue;
        };
        if bleed <= 0.0 {
            out.push(PreflightItem::of_page(
                PreflightKind::BleedNeeded,
                &spread.id,
                &page.id,
            ));
        }
        let Background::Picture { asset, fit, .. } = &shown.bg else {
            continue;
        };
        match facts.0.get(asset) {
            None => out.push(PreflightItem::of_page(
                PreflightKind::PictureMissing,
                &spread.id,
                &page.id,
            )),
            Some(AssetFact::Raster { w, h }) => {
                // A background reaches the bleed, and one across both pages covers both.
                let pages = if shown.source == folio_model::BgSource::Span {
                    spread.pages.len() as f64
                } else {
                    1.0
                };
                let kind = sharpness(
                    (*w, *h),
                    trim.0 * pages + 2.0 * bleed,
                    trim.1 + 2.0 * bleed,
                    if *fit == folio_model::BgFit::Fit {
                        Fit::Fit
                    } else {
                        Fit::Fill
                    },
                );
                if let Some(kind) = kind {
                    out.push(PreflightItem::of_page(kind, &spread.id, &page.id));
                }
            }
            Some(_) => {}
        }
    }
    out
}

/// The paths of the pictures `spread` uses: those in its picture frames and in the backgrounds its
/// pages show.
pub fn spread_assets(book: &Book, spread: &Spread) -> Vec<String> {
    let mut out: Vec<String> = spread
        .frames
        .iter()
        .filter_map(|f| match f {
            Frame::Image(i) => i.image.asset.clone(),
            _ => None,
        })
        .collect();
    for shown in folio_model::resolve(&book.master_pages, spread)
        .into_iter()
        .flatten()
    {
        if let Background::Picture { asset, .. } = shown.bg {
            out.push(asset);
        }
    }
    out.sort();
    out.dedup();
    out
}

/// How much ink, in per cent of all four plates, a press holds before it dries badly. ⚠ Verify
/// against KDP's help pages: they give 240% for some papers and 300% for others.
pub const INK_LIMIT_PERCENT: u32 = 300;

/// The checks on the swatches a spread's colours refer to: one that is not in the book, and one that
/// lays down too much ink. Each swatch is said once for the spread.
pub fn swatch_checks(book: &Book, spread: &Spread) -> Vec<PreflightItem> {
    let mut used = std::collections::BTreeSet::new();
    let mut copy = spread.clone();
    folio_model::visit_spread(&mut copy, &mut |colour| {
        if let Some(id) = colour.swatch_id() {
            used.insert(id.to_owned());
        }
    });
    used.into_iter()
        .filter_map(
            |id| match book.styles.swatches.iter().find(|s| s.id.0 == id) {
                None => Some(PreflightItem::of_swatch(
                    PreflightKind::SwatchMissing,
                    &spread.id,
                    &id,
                )),
                Some(swatch)
                    if swatch
                        .ink
                        .is_some_and(|ink| ink.total() > INK_LIMIT_PERCENT) =>
                {
                    Some(PreflightItem::of_swatch(
                        PreflightKind::InkOverLimit,
                        &spread.id,
                        &swatch.name,
                    ))
                }
                Some(_) => None,
            },
        )
        .collect()
}

/// Text under this size, in points, is too fine for a black made of several inks.
const SMALL_TEXT_PT: f64 = 12.0;

/// A black this heavy on the black plate is meant as black; with colour in it as well, it is "rich".
const RICH_BLACK_FROM: u8 = 80;
const RICH_BLACK_COLOUR: u32 = 20;

/// Small text set in a rich black, said once for each swatch on a frame.
pub fn rich_black_checks(book: &Book, spread: &Spread) -> Vec<PreflightItem> {
    let mut items = Vec::new();
    for frame in &spread.frames {
        let folio_model::Frame::Text(text) = frame else {
            continue;
        };
        let Some(content) = &text.content else {
            continue;
        };
        let mut said = std::collections::BTreeSet::new();
        for run in &content.inline {
            let def = |id: &str| book.styles.styles.get(id);
            let style = run
                .style
                .as_ref()
                .and_then(|s| def(&s.0))
                .or_else(|| def(&text.style.0));
            let size = style.and_then(|d| d.size_pt).unwrap_or(12.0);
            let colour = run
                .colour
                .as_ref()
                .or_else(|| style.and_then(|d| d.colour.as_ref()));
            let Some(id) = colour.and_then(|c| c.swatch_id()) else {
                continue;
            };
            let Some(swatch) = book.styles.swatches.iter().find(|s| s.id.0 == id) else {
                continue;
            };
            let rich = swatch.ink.is_some_and(|ink| {
                ink.k >= RICH_BLACK_FROM
                    && u32::from(ink.c) + u32::from(ink.m) + u32::from(ink.y) >= RICH_BLACK_COLOUR
            });
            if size < SMALL_TEXT_PT && rich && said.insert(id.to_owned()) {
                items.push(PreflightItem {
                    family: Some(swatch.name.clone()),
                    ..PreflightItem::of_frame(
                        PreflightKind::RichBlackSmallText,
                        &spread.id,
                        &text.base.id,
                    )
                });
            }
        }
    }
    items
}
