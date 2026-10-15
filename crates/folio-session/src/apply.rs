// Checking and applying each kind of intent
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use folio_model::{
    Book, Frame, FrameId, Guide, InlineContent, Intent, Page, Pt, ReorderTo, Run, Spread, SpreadId,
    SpreadKind,
};

/// How far, in points, a duplicated frame sits from the one it was copied from.
const DUPLICATE_OFFSET: f64 = 12.0;

use crate::error::SessionError;
use crate::masters::apply_page_numbers;
use crate::session::IdSource;

/// What an intent will touch, worked out before anything changes, so it can be refused whole or
/// captured for undo.
#[derive(Debug, Clone)]
pub struct Plan {
    /// The spreads the intent adds, changes or removes.
    pub touched: Vec<SpreadId>,
    /// The id the spread an `AddSpread` makes will have.
    pub new_spread: Option<SpreadId>,
    /// Intents that repeat on one frame (a drag, a nudge, typing) merge into one undo step.
    pub merge: Option<MergeKey>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MergeKey {
    Move(FrameId),
    Moves(Vec<FrameId>),
    Text(FrameId),
    Style(FrameId),
}

fn spread<'a>(book: &'a Book, id: &SpreadId) -> Result<&'a Spread, SessionError> {
    book.spreads
        .iter()
        .find(|s| &s.id == id)
        .ok_or_else(|| SessionError::NoSuchSpread(id.0.clone()))
}

fn find<'a>(frames: &'a [Frame], id: &FrameId) -> Option<&'a Frame> {
    for frame in frames {
        if &frame.base().id == id {
            return Some(frame);
        }
        if let Frame::Group { children, .. } = frame {
            if let Some(found) = find(children, id) {
                return Some(found);
            }
        }
    }
    None
}

fn find_mut<'a>(frames: &'a mut [Frame], id: &FrameId) -> Option<&'a mut Frame> {
    for frame in frames {
        if &frame.base().id == id {
            return Some(frame);
        }
        if let Frame::Group { children, .. } = frame {
            if let Some(found) = find_mut(children, id) {
                return Some(found);
            }
        }
    }
    None
}

fn remove(frames: &mut Vec<Frame>, id: &FrameId) -> bool {
    if let Some(at) = frames.iter().position(|f| &f.base().id == id) {
        frames.remove(at);
        return true;
    }
    frames.iter_mut().any(|frame| match frame {
        Frame::Group { children, .. } => remove(children, id),
        _ => false,
    })
}

/// The spread a frame is on, anywhere in the book.
fn spread_of(book: &Book, frame: &FrameId) -> Result<SpreadId, SessionError> {
    book.spreads
        .iter()
        .find(|s| find(&s.frames, frame).is_some())
        .map(|s| s.id.clone())
        .ok_or_else(|| SessionError::NoSuchFrame(frame.0.clone()))
}

fn every_id(frame: &Frame, out: &mut Vec<FrameId>) {
    out.push(frame.base().id.clone());
    if let Frame::Group { children, .. } = frame {
        children.iter().for_each(|c| every_id(c, out));
    }
}

/// The largest bleed, in points, a book may ask for (a quarter of an inch is usual).
const MAX_BLEED: f64 = 72.0;

/// Refuses page geometry that cannot be laid out: lengths that are not finite or are negative, a
/// page with no size, margins that leave no room for text, and a column count the grid cannot draw.
fn check_geometry(
    book: &Book,
    trim: Option<folio_model::Size>,
    margins: Option<folio_model::Margins>,
    bleed: Option<folio_model::Pt>,
    grid: Option<&folio_model::Grid>,
) -> Result<(), SessionError> {
    let bad = |what: &str| Err(SessionError::BadGeometry(what.to_owned()));
    let trim = trim.unwrap_or(book.trim);
    let margins = margins.unwrap_or(book.margins);
    if ![trim.w.0, trim.h.0]
        .iter()
        .all(|v| v.is_finite() && *v > 0.0)
    {
        return bad("the page needs a size above zero");
    }
    let parts = [
        margins.top.0,
        margins.bottom.0,
        margins.inside.0,
        margins.outside.0,
    ];
    if !parts.iter().all(|v| v.is_finite() && *v >= 0.0) {
        return bad("margins cannot be negative");
    }
    if margins.inside.0 + margins.outside.0 >= trim.w.0
        || margins.top.0 + margins.bottom.0 >= trim.h.0
    {
        return bad("the margins leave no room on the page");
    }
    if let Some(bleed) = bleed {
        if !bleed.0.is_finite() || !(0.0..=MAX_BLEED).contains(&bleed.0) {
            return bad("the bleed must be between 0 and 1 in");
        }
    }
    if let Some(grid) = grid {
        if !grid.gutter.0.is_finite() || grid.gutter.0 < 0.0 {
            return bad("the column gutter cannot be negative");
        }
        if !(1..=12).contains(&grid.columns) {
            return bad("a grid has between 1 and 12 columns");
        }
        if let Some(b) = &grid.baseline {
            if !(b.increment.0.is_finite() && b.increment.0 > 0.0 && b.offset.0.is_finite()) {
                return bad("the baseline grid needs an increment above zero");
            }
        }
    }
    Ok(())
}

/// How far from the page a guide may be put, in points: far enough for any bleed or pasteboard.
const MAX_GUIDE: f64 = 100_000.0;

fn check_guide_position(pos: Pt) -> Result<(), SessionError> {
    if pos.0.is_finite() && pos.0.abs() <= MAX_GUIDE {
        Ok(())
    } else {
        Err(SessionError::BadGuide)
    }
}

fn guide_on<'a>(book: &'a Book, s: &SpreadId, index: usize) -> Result<&'a Guide, SessionError> {
    spread(book, s)?
        .guides
        .get(index)
        .ok_or_else(|| SessionError::NoSuchGuide(index, s.0.clone()))
}

fn check_guide_free(book: &Book, s: &SpreadId, index: usize) -> Result<(), SessionError> {
    if guide_on(book, s, index)?.locked {
        Err(SessionError::GuideLocked(index))
    } else {
        Ok(())
    }
}

fn guide_mut<'a>(book: &'a mut Book, s: &SpreadId, index: usize) -> Option<&'a mut Guide> {
    book.spreads
        .iter_mut()
        .find(|sp| &sp.id == s)
        .and_then(|sp| sp.guides.get_mut(index))
}

/// Refuses a change to a frame that is locked or belongs to a master page.
fn ensure_free(frame: &Frame) -> Result<(), SessionError> {
    let base = frame.base();
    if base.locked {
        return Err(SessionError::Locked(base.id.0.clone()));
    }
    if base.from_master.is_some() {
        return Err(SessionError::FromMaster(base.id.0.clone()));
    }
    Ok(())
}

/// Works out what `intent` touches, or says why it cannot be done.
pub fn plan(book: &Book, intent: &Intent, ids: &mut IdSource) -> Result<Plan, SessionError> {
    let mut made = plan_one(book, intent, ids)?;
    // Page numbers follow the pages, so a change to which pages there are, and where, redoes them
    // all and every spread is part of the change.
    let restructures = matches!(
        intent,
        Intent::AddSpread { .. } | Intent::DeleteSpread { .. } | Intent::MoveSpread { .. }
    );
    if restructures && (!book.page_numbers.is_none() || !book.master_pages.is_empty()) {
        for spread in &book.spreads {
            if !made.touched.contains(&spread.id) {
                made.touched.push(spread.id.clone());
            }
        }
    }
    Ok(made)
}

fn plan_one(book: &Book, intent: &Intent, ids: &mut IdSource) -> Result<Plan, SessionError> {
    let one = |id: SpreadId, merge: Option<MergeKey>| Plan {
        touched: vec![id],
        new_spread: None,
        merge,
    };
    match intent {
        Intent::MoveFrame {
            spread: s, frame, ..
        } => {
            let found = spread(book, s)?;
            let target = find(&found.frames, frame)
                .ok_or_else(|| SessionError::NoSuchFrame(frame.0.clone()))?;
            ensure_free(target)?;
            Ok(one(s.clone(), Some(MergeKey::Move(frame.clone()))))
        }
        Intent::MoveFrames { spread: s, moves } => {
            let found = spread(book, s)?;
            let mut named = Vec::new();
            for m in moves {
                let target = find(&found.frames, &m.frame)
                    .ok_or_else(|| SessionError::NoSuchFrame(m.frame.0.clone()))?;
                ensure_free(target)?;
                named.push(m.frame.clone());
            }
            named.sort_by(|a, b| a.0.cmp(&b.0));
            Ok(one(s.clone(), Some(MergeKey::Moves(named))))
        }
        Intent::SetImage { frame, asset, .. } => {
            let found = book
                .spreads
                .iter()
                .find_map(|s| find(&s.frames, frame))
                .ok_or_else(|| SessionError::NoSuchFrame(frame.0.clone()))?;
            let Frame::Image(_) = found else {
                return Err(SessionError::NotImage(frame.0.clone()));
            };
            ensure_free(found)?;
            if let Some(asset) = asset {
                let inside = asset.starts_with("assets/images/")
                    && !asset.split('/').any(|part| part == ".." || part.is_empty());
                if !inside {
                    return Err(SessionError::BadAsset(asset.clone()));
                }
            }
            Ok(one(spread_of(book, frame)?, None))
        }
        Intent::SetPageNumbers { .. } => Ok(Plan {
            touched: book.spreads.iter().map(|s| s.id.clone()).collect(),
            new_spread: None,
            merge: None,
        }),
        Intent::HideMasterItem {
            spread: s,
            page,
            frame,
            ..
        } => {
            let found = spread(book, s)?;
            let on = found
                .pages
                .iter()
                .find(|p| &p.id == page)
                .ok_or_else(|| SessionError::NotOnSpread(page.0.clone(), s.0.clone()))?;
            let owns = on.master.as_ref().is_some_and(|m| {
                book.master_pages
                    .iter()
                    .any(|mp| &mp.id == m && mp.frames.iter().any(|f| &f.base().id == frame))
            });
            if !owns {
                return Err(SessionError::NoSuchFrame(frame.0.clone()));
            }
            Ok(one(s.clone(), None))
        }
        Intent::AddGuide { spread: s, pos, .. } => {
            spread(book, s)?;
            check_guide_position(*pos)?;
            Ok(one(s.clone(), None))
        }
        Intent::MoveGuide {
            spread: s,
            index,
            pos,
        } => {
            check_guide_position(*pos)?;
            check_guide_free(book, s, *index)?;
            Ok(one(s.clone(), None))
        }
        Intent::LockGuide {
            spread: s, index, ..
        } => {
            guide_on(book, s, *index)?;
            Ok(one(s.clone(), None))
        }
        Intent::RemoveGuide { spread: s, index } => {
            check_guide_free(book, s, *index)?;
            Ok(one(s.clone(), None))
        }
        Intent::ClearGuides { spread: named } => {
            if let Some(s) = named {
                spread(book, s)?;
            }
            Ok(Plan {
                touched: book
                    .spreads
                    .iter()
                    .filter(|sp| named.as_ref().is_none_or(|n| n == &sp.id))
                    .filter(|sp| sp.guides.iter().any(|g| !g.locked))
                    .map(|sp| sp.id.clone())
                    .collect(),
                new_spread: None,
                merge: None,
            })
        }
        Intent::SetText { frame, .. } => {
            check_text(book, frame)?;
            Ok(one(
                spread_of(book, frame)?,
                Some(MergeKey::Text(frame.clone())),
            ))
        }
        Intent::SetStyle { frame, .. } => {
            let on = spread_of(book, frame)?;
            let styled = book
                .spreads
                .iter()
                .find_map(|sp| find(&sp.frames, frame))
                .is_some_and(|f| matches!(f, Frame::Text(_) | Frame::Shape(_)));
            if !styled {
                return Err(SessionError::NotText(frame.0.clone()));
            }
            if let Some(found) = book.spreads.iter().find_map(|sp| find(&sp.frames, frame)) {
                ensure_free(found)?;
            }
            Ok(one(on, None))
        }
        Intent::OverrideStyle { frame, .. } => {
            check_text(book, frame).or_else(|e| match e {
                // Overrides sit on a frame whose words come from a flow too.
                SessionError::FlowText(_) => book
                    .spreads
                    .iter()
                    .find_map(|sp| find(&sp.frames, frame))
                    .map_or(Ok(()), ensure_free),
                other => Err(other),
            })?;
            Ok(one(
                spread_of(book, frame)?,
                Some(MergeKey::Style(frame.clone())),
            ))
        }
        Intent::AddFrame { spread: s, frame } => {
            spread(book, s)?;
            let mut new_ids = Vec::new();
            every_id(frame, &mut new_ids);
            for id in new_ids {
                if book
                    .spreads
                    .iter()
                    .any(|sp| find(&sp.frames, &id).is_some())
                {
                    return Err(SessionError::DuplicateFrame(id.0));
                }
            }
            Ok(one(s.clone(), None))
        }
        Intent::DeleteFrame { spread: s, frame } => {
            let found = spread(book, s)?;
            let target = find(&found.frames, frame)
                .ok_or_else(|| SessionError::NoSuchFrame(frame.0.clone()))?;
            ensure_free(target)?;
            Ok(one(s.clone(), None))
        }
        Intent::DuplicateFrame { spread: s, frame }
        | Intent::LockFrame {
            spread: s, frame, ..
        } => {
            let found = spread(book, s)?;
            if !found.frames.iter().any(|f| &f.base().id == frame) {
                return Err(SessionError::NotOnSpread(frame.0.clone(), s.0.clone()));
            }
            Ok(one(s.clone(), None))
        }
        Intent::Reorder {
            spread: s, frame, ..
        } => {
            let found = spread(book, s)?;
            let target = found
                .frames
                .iter()
                .find(|f| &f.base().id == frame)
                .ok_or_else(|| SessionError::NotOnSpread(frame.0.clone(), s.0.clone()))?;
            ensure_free(target)?;
            Ok(one(s.clone(), None))
        }
        Intent::AddSpread { after, .. } => {
            spread(book, after)?;
            let id = SpreadId(ids("sp"));
            Ok(Plan {
                touched: vec![id.clone()],
                new_spread: Some(id),
                merge: None,
            })
        }
        Intent::DeleteSpread { spread: s } => {
            let found = spread(book, s)?;
            if found.kind == SpreadKind::Cover {
                return Err(SessionError::CoverFixed);
            }
            if book.spreads.len() == 1 {
                return Err(SessionError::LastSpread);
            }
            Ok(one(s.clone(), None))
        }
        Intent::MoveSpread { spread: s, .. } => {
            if spread(book, s)?.kind == SpreadKind::Cover {
                return Err(SessionError::CoverFixed);
            }
            Ok(Plan {
                touched: Vec::new(),
                new_spread: None,
                merge: None,
            })
        }
        Intent::SetBookGeometry {
            trim,
            margins,
            bleed,
            grid,
        } => {
            check_geometry(book, *trim, *margins, *bleed, grid.as_ref())?;
            Ok(Plan {
                touched: Vec::new(),
                new_spread: None,
                merge: None,
            })
        }
    }
}

/// A copy of a frame with every id new, so a duplicated spread shares nothing with its source.
fn renewed(frame: &Frame, ids: &mut IdSource) -> Frame {
    let mut copy = frame.clone();
    renew(&mut copy, ids);
    copy
}

fn renew(frame: &mut Frame, ids: &mut IdSource) {
    frame.base_mut().id = FrameId(ids("fr"));
    if let Frame::Group { children, .. } = frame {
        children.iter_mut().for_each(|c| renew(c, ids));
    }
}

/// Applies an intent that `plan` accepted.
pub fn run(book: &mut Book, intent: Intent, plan: &Plan, ids: &mut IdSource) {
    let restructures = matches!(
        intent,
        Intent::AddSpread { .. } | Intent::DeleteSpread { .. } | Intent::MoveSpread { .. }
    );
    run_one(book, intent, plan, ids);
    if restructures {
        apply_page_numbers(book);
    }
}

fn run_one(book: &mut Book, intent: Intent, plan: &Plan, ids: &mut IdSource) {
    match intent {
        Intent::MoveFrame {
            spread: s,
            frame,
            rect,
        } => {
            if let Some(f) = book
                .spreads
                .iter_mut()
                .find(|sp| sp.id == s)
                .and_then(|sp| find_mut(&mut sp.frames, &frame))
            {
                f.base_mut().rect = rect;
            }
        }
        Intent::SetImage { frame, asset, fit } => {
            if let Some(Frame::Image(img)) = book
                .spreads
                .iter_mut()
                .find_map(|sp| find_mut(&mut sp.frames, &frame))
            {
                if let Some(asset) = asset {
                    img.image.asset = Some(asset);
                }
                if let Some(fit) = fit {
                    img.image.fit = fit;
                }
            }
        }
        Intent::SetPageNumbers { position } => {
            book.page_numbers = position;
            apply_page_numbers(book);
        }
        Intent::HideMasterItem {
            spread: s,
            page,
            frame,
            hide,
        } => {
            if let Some(sp) = book.spreads.iter_mut().find(|sp| sp.id == s) {
                let at = sp.master_overrides.iter().position(|o| o.page_id == page);
                match (at, hide) {
                    (Some(at), true) => {
                        let hidden = &mut sp.master_overrides[at].hide;
                        if !hidden.contains(&frame) {
                            hidden.push(frame);
                        }
                    }
                    (None, true) => sp.master_overrides.push(folio_model::MasterOverride {
                        page_id: page,
                        hide: vec![frame],
                    }),
                    (Some(at), false) => {
                        sp.master_overrides[at].hide.retain(|f| f != &frame);
                        if sp.master_overrides[at].hide.is_empty() {
                            sp.master_overrides.remove(at);
                        }
                    }
                    (None, false) => {}
                }
            }
        }
        Intent::AddGuide {
            spread: s,
            axis,
            pos,
        } => {
            if let Some(sp) = book.spreads.iter_mut().find(|sp| sp.id == s) {
                sp.guides.push(Guide {
                    axis,
                    pos,
                    locked: false,
                });
            }
        }
        Intent::MoveGuide {
            spread: s,
            index,
            pos,
        } => {
            if let Some(g) = guide_mut(book, &s, index) {
                g.pos = pos;
            }
        }
        Intent::LockGuide {
            spread: s,
            index,
            locked,
        } => {
            if let Some(g) = guide_mut(book, &s, index) {
                g.locked = locked;
            }
        }
        Intent::RemoveGuide { spread: s, index } => {
            if let Some(sp) = book.spreads.iter_mut().find(|sp| sp.id == s) {
                if index < sp.guides.len() {
                    sp.guides.remove(index);
                }
            }
        }
        Intent::ClearGuides { spread: named } => {
            for sp in book
                .spreads
                .iter_mut()
                .filter(|sp| named.as_ref().is_none_or(|n| n == &sp.id))
            {
                sp.guides.retain(|g| g.locked);
            }
        }
        Intent::SetText { frame, text } => {
            if let Some(Frame::Text(t)) = book
                .spreads
                .iter_mut()
                .find_map(|sp| find_mut(&mut sp.frames, &frame))
            {
                t.content = Some(InlineContent {
                    inline: vec![Run {
                        text,
                        style: None,
                        bold: false,
                        italic: false,
                    }],
                });
            }
        }
        Intent::SetStyle { frame, style } => {
            if let Some(f) = book
                .spreads
                .iter_mut()
                .find_map(|sp| find_mut(&mut sp.frames, &frame))
            {
                match f {
                    Frame::Text(t) => t.style = style,
                    Frame::Shape(sh) => sh.style = Some(style),
                    _ => {}
                }
            }
        }
        Intent::OverrideStyle { frame, overrides } => {
            if let Some(Frame::Text(t)) = book
                .spreads
                .iter_mut()
                .find_map(|sp| find_mut(&mut sp.frames, &frame))
            {
                t.overrides = Some(overrides);
            }
        }
        Intent::AddFrame { spread: s, frame } => {
            if let Some(sp) = book.spreads.iter_mut().find(|sp| sp.id == s) {
                sp.frames.push(frame);
            }
        }
        Intent::DeleteFrame { spread: s, frame } => {
            if let Some(sp) = book.spreads.iter_mut().find(|sp| sp.id == s) {
                remove(&mut sp.frames, &frame);
            }
        }
        Intent::MoveFrames { spread: s, moves } => {
            if let Some(sp) = book.spreads.iter_mut().find(|sp| sp.id == s) {
                for m in moves {
                    if let Some(f) = find_mut(&mut sp.frames, &m.frame) {
                        f.base_mut().rect = m.rect;
                    }
                }
            }
        }
        Intent::DuplicateFrame { spread: s, frame } => {
            if let Some(sp) = book.spreads.iter_mut().find(|sp| sp.id == s) {
                if let Some(source) = sp.frames.iter().find(|f| f.base().id == frame) {
                    let mut copy = renewed(source, ids);
                    let base = copy.base_mut();
                    base.rect.x = Pt(base.rect.x.0 + DUPLICATE_OFFSET);
                    base.rect.y = Pt(base.rect.y.0 + DUPLICATE_OFFSET);
                    // A copy is the user's own: it is neither locked nor tied to a master.
                    base.locked = false;
                    base.from_master = None;
                    // A copy is not another slot in the source's flow.
                    if let Frame::Text(t) = &mut copy {
                        t.flow = None;
                    }
                    sp.frames.push(copy);
                }
            }
        }
        Intent::LockFrame {
            spread: s,
            frame,
            locked,
        } => {
            if let Some(f) = book
                .spreads
                .iter_mut()
                .find(|sp| sp.id == s)
                .and_then(|sp| find_mut(&mut sp.frames, &frame))
            {
                f.base_mut().locked = locked;
            }
        }
        Intent::Reorder {
            spread: s,
            frame,
            to,
        } => {
            if let Some(sp) = book.spreads.iter_mut().find(|sp| sp.id == s) {
                if let Some(at) = sp.frames.iter().position(|f| f.base().id == frame) {
                    let moved = sp.frames.remove(at);
                    let last = sp.frames.len();
                    let target = match to {
                        ReorderTo::Front => last,
                        ReorderTo::Back => 0,
                        ReorderTo::Index(i) => (i as usize).min(last),
                    };
                    sp.frames.insert(target, moved);
                }
            }
        }
        Intent::AddSpread { after, duplicate } => {
            let Some(at) = book.spreads.iter().position(|sp| sp.id == after) else {
                return;
            };
            let id = plan
                .new_spread
                .clone()
                .unwrap_or_else(|| SpreadId(ids("sp")));
            let source = &book.spreads[at];
            let made = if duplicate && source.kind != SpreadKind::Cover {
                Spread {
                    id,
                    kind: source.kind,
                    pages: source
                        .pages
                        .iter()
                        .map(|p| Page {
                            id: folio_model::PageId(ids("pg")),
                            master: p.master.clone(),
                        })
                        .collect(),
                    frames: source.frames.iter().map(|f| renewed(f, ids)).collect(),
                    guides: source.guides.clone(),
                    master_overrides: Vec::new(),
                }
            } else {
                // The page after the cover opens on the right; every other new spread is a pair.
                let (kind, pages) = if source.kind == SpreadKind::Cover {
                    (SpreadKind::Opener, 1)
                } else {
                    (SpreadKind::Spread, 2)
                };
                Spread {
                    id,
                    kind,
                    pages: (0..pages)
                        .map(|_| Page {
                            id: folio_model::PageId(ids("pg")),
                            master: None,
                        })
                        .collect(),
                    frames: Vec::new(),
                    guides: Vec::new(),
                    master_overrides: Vec::new(),
                }
            };
            book.spreads.insert(at + 1, made);
        }
        Intent::DeleteSpread { spread: s } => book.spreads.retain(|sp| sp.id != s),
        Intent::MoveSpread { spread: s, to } => {
            if let Some(at) = book.spreads.iter().position(|sp| sp.id == s) {
                let moved = book.spreads.remove(at);
                // Nothing goes in front of the cover.
                let first = usize::from(
                    book.spreads
                        .first()
                        .is_some_and(|f| f.kind == SpreadKind::Cover),
                );
                let target = (to as usize).clamp(first, book.spreads.len());
                book.spreads.insert(target, moved);
            }
        }
        Intent::SetBookGeometry {
            trim,
            margins,
            bleed,
            grid,
        } => {
            if let Some(trim) = trim {
                book.trim = trim;
            }
            if let Some(margins) = margins {
                book.margins = margins;
            }
            if let Some(bleed) = bleed {
                book.bleed = bleed;
            }
            if let Some(grid) = grid {
                book.grid = grid;
            }
            // The page numbers sit by the margins, so a new page or margin moves them.
            if !book.page_numbers.is_none() {
                apply_page_numbers(book);
            }
        }
    }
}

/// Checks a text intent against the frame it names: it must be a text frame whose words are its own.
pub fn check_text(book: &Book, frame: &FrameId) -> Result<(), SessionError> {
    let found = book
        .spreads
        .iter()
        .find_map(|s| find(&s.frames, frame))
        .ok_or_else(|| SessionError::NoSuchFrame(frame.0.clone()))?;
    match found {
        Frame::Text(t) if t.flow.is_some() => Err(SessionError::FlowText(frame.0.clone())),
        Frame::Text(_) => ensure_free(found),
        _ => Err(SessionError::NotText(frame.0.clone())),
    }
}
