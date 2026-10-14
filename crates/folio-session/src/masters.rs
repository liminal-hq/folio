// Master pages made from the page numbers choice
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use folio_model::{Book, Frame, Master, MasterId, PageNumbers, Spread, SpreadKind};
use serde_json::json;

/// The master a page that opens on the right uses, and the one a page on the left uses when the
/// numbers are on the outside.
pub const RECTO: &str = "mst_a";
pub const VERSO: &str = "mst_b";

const NUMBER_WIDTH: f64 = 60.0;
const NUMBER_HEIGHT: f64 = 18.0;

/// Whether the page at `index` of `spread` is on the right, so its binding edge is on the left.
pub fn is_recto(spread: &Spread, index: usize) -> bool {
    if spread.pages.len() == 2 {
        index == 1
    } else {
        spread.kind != SpreadKind::Closer
    }
}

fn number_frame(id: &str, x: f64, y: f64, align: &str) -> Frame {
    serde_json::from_value(json!({
        "type": "text",
        "id": id,
        "box": { "x": x, "y": y, "w": NUMBER_WIDTH, "h": NUMBER_HEIGHT },
        "style": "folio",
        "content": { "inline": [{ "text": "{page}" }] },
        "columns": 1,
        "baselineLock": false,
        "overrides": { "size": null, "align": align, "bold": null, "italic": null },
    }))
    .expect("the page number frame is well formed")
}

/// Makes the master pages `book.page_numbers` asks for and gives every page after the cover its
/// master. With no numbers there are no masters, and every page is without one.
pub fn apply_page_numbers(book: &mut Book) {
    let mode = book.page_numbers;
    let ours = |id: &MasterId| id.0 == RECTO || id.0 == VERSO;
    // Masters that came from elsewhere (an import, a newer Folio) are not ours to remove.
    book.master_pages.retain(|m| !ours(&m.id));
    if mode == PageNumbers::None {
        for page in book.spreads.iter_mut().flat_map(|s| s.pages.iter_mut()) {
            if page.master.as_ref().is_some_and(ours) {
                page.master = None;
            }
        }
        prune_overrides(book);
        book.masters = book.master_pages.iter().map(|m| m.id.clone()).collect();
        return;
    }
    let (w, h, bottom, outside) = (
        book.trim.w.0,
        book.trim.h.0,
        book.margins.bottom.0,
        book.margins.outside.0,
    );
    // Centred in the bottom margin, and never lower than the edge of the page.
    let y = (h - bottom + (bottom - NUMBER_HEIGHT) / 2.0).min(h - NUMBER_HEIGHT - 4.0);
    let recto = match mode {
        PageNumbers::Centre => number_frame("fr_pn_a", (w - NUMBER_WIDTH) / 2.0, y, "center"),
        _ => number_frame("fr_pn_a", w - outside - NUMBER_WIDTH, y, "right"),
    };
    book.master_pages.push(Master {
        id: MasterId(RECTO.into()),
        name: "A".into(),
        frames: vec![recto],
        guides: Vec::new(),
    });
    if mode == PageNumbers::Outside {
        book.master_pages.push(Master {
            id: MasterId(VERSO.into()),
            name: "B".into(),
            frames: vec![number_frame("fr_pn_b", outside, y, "left")],
            guides: Vec::new(),
        });
    }
    book.masters = book.master_pages.iter().map(|m| m.id.clone()).collect();
    for spread in &mut book.spreads {
        let cover = spread.kind == SpreadKind::Cover;
        let rectos: Vec<bool> = (0..spread.pages.len())
            .map(|i| is_recto(spread, i))
            .collect();
        for (page, recto) in spread.pages.iter_mut().zip(rectos) {
            // A page the person gave another master keeps it.
            if page.master.as_ref().is_some_and(|m| !ours(m)) {
                continue;
            }
            page.master = if cover {
                None
            } else if mode == PageNumbers::Outside && !recto {
                Some(MasterId(VERSO.into()))
            } else {
                Some(MasterId(RECTO.into()))
            };
        }
    }
    prune_overrides(book);
}

/// Forgets hidden master items that are no longer on the master their page uses, so a number hidden
/// on a page that has since changed sides, or been through None and back, does not stay hidden.
fn prune_overrides(book: &mut Book) {
    let masters = &book.master_pages;
    for spread in &mut book.spreads {
        let pages = spread.pages.clone();
        spread.master_overrides.retain_mut(|o| {
            let on = pages
                .iter()
                .find(|p| p.id == o.page_id)
                .and_then(|p| p.master.as_ref())
                .and_then(|id| masters.iter().find(|m| &m.id == id));
            o.hide
                .retain(|f| on.is_some_and(|m| m.frames.iter().any(|mf| &mf.base().id == f)));
            !o.hide.is_empty()
        });
    }
}
