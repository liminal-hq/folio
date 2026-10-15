// Master pages made from the page numbers choice
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use folio_model::{
    Align, Axis, Book, Frame, Guide, Master, MasterId, PageNumbers, Pt, Spread, SpreadId,
    SpreadKind, StyleOverrides,
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

/// Makes master A, and master B when the numbers are outside, if they are missing, and gives every
/// page after the cover that has no master the one its side uses. Run when something first needs a
/// master: editing one, or a guide on every page.
pub fn ensure_masters(book: &mut Book) {
    master_mut(book, RECTO, "A");
    if book.page_numbers == PageNumbers::Outside {
        master_mut(book, VERSO, "B");
    }
    assign(book);
}

/// The master a guide is on the facing side of, when the book has both A and B: a position `local`
/// on one is `trim width − local` on the other.
fn facing(book: &Book, master: &MasterId) -> Option<MasterId> {
    let has = |id: &str| book.master_pages.iter().any(|m| m.id.0 == id);
    if !(has(RECTO) && has(VERSO)) {
        return None;
    }
    match master.0.as_str() {
        RECTO => Some(MasterId(VERSO.into())),
        VERSO => Some(MasterId(RECTO.into())),
        _ => None,
    }
}

/// Whether `guide` on `spread` can go on every page: it is not on the cover, and a vertical one is over
/// a page and not in the gutter or off the spread.
pub fn can_promote(book: &Book, spread: &Spread, guide: &Guide) -> bool {
    if spread.kind == SpreadKind::Cover {
        return false;
    }
    match guide.axis {
        Axis::Y => true,
        Axis::X => over_page(book, spread, guide.pos.0).is_some(),
    }
}

/// The page of `spread` that the position `x` (from the first page's left edge) is over, and its
/// position on that page. The gutter, and anything off the spread, is over no page.
fn over_page(book: &Book, spread: &Spread, x: f64) -> Option<(usize, f64)> {
    let w = book.trim.w.0;
    let pages = spread.pages.len();
    if !x.is_finite() || x < 0.0 || x > w * pages as f64 {
        return None;
    }
    let index = (x / w).floor() as usize;
    if index >= pages {
        return Some((pages - 1, w));
    }
    if index > 0 && (x - index as f64 * w).abs() < f64::EPSILON {
        return None;
    }
    Some((index, x - index as f64 * w))
}

/// Moves the guide `index` of `spread` onto the masters its pages use (making them first when the
/// book has none), where it shows on every page that uses them. Returns whether it could.
pub fn promote_guide(book: &mut Book, spread: &SpreadId, index: usize) -> bool {
    let Some(at) = book.spreads.iter().position(|s| &s.id == spread) else {
        return false;
    };
    let Some(&guide) = book.spreads[at].guides.get(index) else {
        return false;
    };
    if !can_promote(book, &book.spreads[at], &guide) {
        return false;
    }
    ensure_masters(book);
    let w = book.trim.w.0;
    let pages = book.spreads[at].pages.clone();
    let mut landings: Vec<(MasterId, f64)> = Vec::new();
    match guide.axis {
        Axis::Y => {
            for page in &pages {
                if let Some(m) = &page.master {
                    landings.push((m.clone(), guide.pos.0));
                }
            }
        }
        Axis::X => {
            let Some((i, local)) = over_page(book, &book.spreads[at], guide.pos.0) else {
                return false;
            };
            if let Some(m) = &pages[i].master {
                landings.push((m.clone(), local));
                if let Some(other) = facing(book, m) {
                    landings.push((other, w - local));
                }
            }
        }
    }
    for (id, pos) in landings {
        if let Some(master) = book.master_pages.iter_mut().find(|m| m.id == id) {
            let same = |g: &Guide| g.axis == guide.axis && (g.pos.0 - pos).abs() < 1e-6;
            if !master.guides.iter().any(same) {
                master.guides.push(Guide {
                    axis: guide.axis,
                    pos: folio_model::Pt(pos),
                    locked: false,
                });
            }
        }
    }
    book.spreads[at].guides.remove(index);
    true
}

/// The page of `spread` that uses `master`, first of them, if any.
pub fn page_using(spread: &Spread, master: &MasterId) -> Option<usize> {
    spread
        .pages
        .iter()
        .position(|p| p.master.as_ref() == Some(master))
}

/// Moves the guide `index` of `master` back onto `spread`, where it shows on its own, and takes the
/// mirrored copy off the facing master. Returns whether it could.
pub fn demote_guide(book: &mut Book, master: &MasterId, index: usize, spread: &SpreadId) -> bool {
    let Some(at) = book.spreads.iter().position(|s| &s.id == spread) else {
        return false;
    };
    let Some(page) = page_using(&book.spreads[at], master) else {
        return false;
    };
    let Some(m) = book.master_pages.iter_mut().find(|m| &m.id == master) else {
        return false;
    };
    if index >= m.guides.len() {
        return false;
    }
    let guide = m.guides.remove(index);
    let w = book.trim.w.0;
    let on_spread = match guide.axis {
        Axis::X => guide.pos.0 + page as f64 * w,
        Axis::Y => guide.pos.0,
    };
    // The copies promoting made: a horizontal guide is on every master the spread uses, and a
    // vertical one is mirrored onto the facing master.
    let copies: Vec<(MasterId, f64)> = match guide.axis {
        Axis::X => facing(book, master)
            .map(|other| (other, w - guide.pos.0))
            .into_iter()
            .collect(),
        Axis::Y => {
            let mut used: Vec<MasterId> = Vec::new();
            for p in &book.spreads[at].pages {
                if let Some(m) = &p.master {
                    if m != master && !used.contains(m) {
                        used.push(m.clone());
                    }
                }
            }
            used.into_iter().map(|m| (m, guide.pos.0)).collect()
        }
    };
    for (other, pos) in copies {
        if let Some(o) = book.master_pages.iter_mut().find(|m| m.id == other) {
            if let Some(i) = o
                .guides
                .iter()
                .position(|g| g.axis == guide.axis && !g.locked && (g.pos.0 - pos).abs() < 1e-6)
            {
                o.guides.remove(i);
            }
        }
    }
    book.spreads[at].guides.push(Guide {
        axis: guide.axis,
        pos: folio_model::Pt(on_spread),
        locked: false,
    });
    true
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
