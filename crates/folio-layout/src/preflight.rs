// Preflight geometry: checks on where frames sit, from the model alone
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use folio_model::{Book, Frame, Rect, Spread, SpreadKind};

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

/// Where the pages of `spread` sit. An opener is the right-hand page alone, so it starts at the
/// gutter; a closer, a single page and a cover start at the origin.
pub fn page_rects(book: &Book, spread: &Spread) -> Vec<PageRect> {
    let w = book.trim.w.0;
    let h = book.trim.h.0;
    let count = spread.pages.len();
    (0..count)
        .map(|i| {
            let recto = match spread.kind {
                SpreadKind::Opener | SpreadKind::Single | SpreadKind::Cover => true,
                SpreadKind::Closer => false,
                SpreadKind::Spread => i == 1 || count == 1,
            };
            let slot = if spread.kind == SpreadKind::Opener {
                1
            } else {
                i
            };
            PageRect {
                rect: Rect {
                    x: folio_model::Pt(slot as f64 * w),
                    y: folio_model::Pt(0.0),
                    w: folio_model::Pt(w),
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
            Frame::Image(_) => {
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
    PreflightItem {
        kind,
        spread: spread.id.clone(),
        frame: frame.base().id.clone(),
    }
}
