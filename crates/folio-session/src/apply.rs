// Checking and applying each kind of intent
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use folio_model::{
    Book, Frame, FrameId, InlineContent, Intent, Page, ReorderTo, Run, Spread, SpreadId, SpreadKind,
};

use crate::error::SessionError;
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

/// Works out what `intent` touches, or says why it cannot be done.
pub fn plan(book: &Book, intent: &Intent, ids: &mut IdSource) -> Result<Plan, SessionError> {
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
            find(&found.frames, frame).ok_or_else(|| SessionError::NoSuchFrame(frame.0.clone()))?;
            Ok(one(s.clone(), Some(MergeKey::Move(frame.clone()))))
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
            Ok(one(on, None))
        }
        Intent::OverrideStyle { frame, .. } => {
            check_text(book, frame).or_else(|e| match e {
                // Overrides sit on a frame whose words come from a flow too.
                SessionError::FlowText(_) => Ok(()),
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
            find(&found.frames, frame).ok_or_else(|| SessionError::NoSuchFrame(frame.0.clone()))?;
            Ok(one(s.clone(), None))
        }
        Intent::Reorder {
            spread: s, frame, ..
        } => {
            let found = spread(book, s)?;
            if !found.frames.iter().any(|f| &f.base().id == frame) {
                return Err(SessionError::NotOnSpread(frame.0.clone(), s.0.clone()));
            }
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
        Intent::SetBookGeometry { .. } => Ok(Plan {
            touched: Vec::new(),
            new_spread: None,
            merge: None,
        }),
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
        Frame::Text(_) => Ok(()),
        _ => Err(SessionError::NotText(frame.0.clone())),
    }
}
