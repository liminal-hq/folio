// Master pages made from the page numbers choice
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use folio_model::{
    Align, Book, Frame, Master, MasterId, PageNumbers, Pt, Spread, SpreadKind, StyleOverrides,
};
use serde_json::json;

/// The master a page that opens on the right uses, and the one a page on the left uses when the
/// numbers are on the outside.
pub const RECTO: &str = "mst_a";
pub const VERSO: &str = "mst_b";

/// The ids of the two frames the page numbers manage.
const NUMBER_A: &str = "fr_pn_a";
const NUMBER_B: &str = "fr_pn_b";
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

fn number_frame(id: &str, x: f64, y: f64, align: Align) -> Frame {
    let align = match align {
        Align::Left => "left",
        Align::Center => "center",
        Align::Right => "right",
        Align::Justify => "justify",
    };
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

fn ours(id: &MasterId) -> bool {
    id.0 == RECTO || id.0 == VERSO
}

pub fn is_number(frame: &Frame) -> bool {
    matches!(frame.base().id.0.as_str(), NUMBER_A | NUMBER_B)
}

/// Puts the number frame `id` on `master` at the given box, or moves the one already there. Only its
/// box and alignment are ours: the person may have changed the words or the style.
fn place_number(master: &mut Master, id: &str, x: f64, y: f64, align: Align) {
    if let Some(Frame::Text(text)) = master.frames.iter_mut().find(|f| f.base().id.0 == id) {
        text.base.rect.x = Pt(x);
        text.base.rect.y = Pt(y);
        let overrides = text.overrides.get_or_insert(StyleOverrides {
            size: None,
            align: None,
            bold: None,
            italic: None,
        });
        overrides.align = Some(align);
    } else {
        master.frames.push(number_frame(id, x, y, align));
    }
}

fn master_mut<'a>(book: &'a mut Book, id: &str, name: &str) -> &'a mut Master {
    let at = book.master_pages.iter().position(|m| m.id.0 == id);
    let at = at.unwrap_or_else(|| {
        book.master_pages.push(Master {
            id: MasterId(id.into()),
            name: name.into(),
            frames: Vec::new(),
            guides: Vec::new(),
        });
        book.master_pages.len() - 1
    });
    &mut book.master_pages[at]
}

/// Whether a master holds anything but the number frames, so it is not ours to remove.
fn holds_more(master: &Master) -> bool {
    master.frames.iter().any(|f| !is_number(f)) || !master.guides.is_empty()
}

/// Gives the pages that have no master, or one of ours, the master their side uses: B on the left
/// when there is one, A otherwise, and none for a cover or when neither is there.
fn assign(book: &mut Book) {
    let has = |id: &str| book.master_pages.iter().any(|m| m.id.0 == id);
    let (has_a, has_b) = (has(RECTO), has(VERSO));
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
            } else if !recto && has_b {
                Some(MasterId(VERSO.into()))
            } else if has_a {
                Some(MasterId(RECTO.into()))
            } else {
                None
            };
        }
    }
}

/// Puts the page numbers `book.page_numbers` asks for on masters A and B, and takes them off when
/// there are none. Only the number frames are managed: anything else on a master stays, a master
/// that holds something is never removed, and the pages keep using it. A kept master B gets a
/// number of its own when the numbers are centred, so the left-hand pages are not left without one.
pub fn apply_page_numbers(book: &mut Book) {
    let mode = book.page_numbers;
    if mode == PageNumbers::None {
        for master in book.master_pages.iter_mut().filter(|m| ours(&m.id)) {
            master.frames.retain(|f| !is_number(f));
        }
        // A master of ours with nothing left on it was only there for the numbers.
        book.master_pages.retain(|m| !ours(&m.id) || holds_more(m));
        assign(book);
        prune_overrides(book);
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
    match mode {
        PageNumbers::Centre => {
            let centred = (w - NUMBER_WIDTH) / 2.0;
            place_number(
                master_mut(book, RECTO, "A"),
                NUMBER_A,
                centred,
                y,
                Align::Center,
            );
            // Master B only held the left-hand number, unless the person put something on it.
            book.master_pages
                .retain(|m| m.id.0 != VERSO || holds_more(m));
            if let Some(b) = book.master_pages.iter_mut().find(|m| m.id.0 == VERSO) {
                place_number(b, NUMBER_B, centred, y, Align::Center);
            }
        }
        _ => {
            place_number(
                master_mut(book, RECTO, "A"),
                NUMBER_A,
                w - outside - NUMBER_WIDTH,
                y,
                Align::Right,
            );
            place_number(
                master_mut(book, VERSO, "B"),
                NUMBER_B,
                outside,
                y,
                Align::Left,
            );
        }
    }
    assign(book);
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
