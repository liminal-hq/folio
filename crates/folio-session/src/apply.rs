// Checking and applying each kind of intent
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use folio_model::{
    clean_language, BackgroundTarget, BarcodeChoice, Book, BookFont, Colour, FlowId, Frame,
    FrameId, Guide, InlineContent, Intent, Master, MasterId, Page, Paint, Proofing, Pt, Rect,
    ReorderTo, Run, Spread, SpreadId, SpreadKind, StoryPara, Surface, TableCell, TableEdit,
    TableFrame, MAX_IGNORED_WORDS,
};

/// The most preflight issues a book can leave alone.
pub const MAX_IGNORED_ISSUES: usize = 500;

/// How long the details of a book may be.
const MAX_TITLE: usize = 300;
const MAX_DESCRIPTION: usize = 5000;
const MAX_AUTHORS: usize = 20;

/// The most fonts a book can keep, and a sanity limit on the weights a font can name.
pub const MAX_BOOK_FONTS: usize = 128;

/// Whether `fonts` could be kept: each names a family and a file under `assets/fonts/`, and none
/// is listed twice.
fn check_fonts(fonts: &[BookFont]) -> Result<(), SessionError> {
    if fonts.len() > MAX_BOOK_FONTS {
        return Err(SessionError::BadFont(format!(
            "a book keeps at most {MAX_BOOK_FONTS} fonts"
        )));
    }
    for (at, font) in fonts.iter().enumerate() {
        if font.family.trim().is_empty() {
            return Err(SessionError::BadFont("a font needs a family name".into()));
        }
        if !font.path.starts_with("assets/fonts/") || font.path.contains("..") {
            return Err(SessionError::BadFont(format!(
                "`{}` is not a file in the book's fonts",
                font.path
            )));
        }
        if !(1..=1000).contains(&font.weight) {
            return Err(SessionError::BadFont(format!(
                "{} is not a font weight",
                font.weight
            )));
        }
        if fonts[..at].contains(font) {
            return Err(SessionError::BadFont(format!(
                "{} is listed twice",
                font.family
            )));
        }
    }
    Ok(())
}

/// The widest an outline can be, and the roundest a corner, in points.
const MAX_STROKE: f64 = 100.0;
const MAX_RADIUS: f64 = 1000.0;

/// How far, in points, a duplicated frame sits from the one it was copied from.
const DUPLICATE_OFFSET: f64 = 12.0;

use crate::error::SessionError;
use crate::masters::{
    apply_page_numbers, can_promote, demote_guide, ensure_masters, page_using, promote_guide,
};
use crate::session::IdSource;

/// What an intent will touch, worked out before anything changes, so it can be refused whole or
/// captured for undo.
#[derive(Debug, Clone)]
pub struct Plan {
    /// The spreads the intent adds, changes or removes.
    pub touched: Vec<SpreadId>,
    /// The id the spread an `AddSpread` makes will have.
    pub new_spread: Option<SpreadId>,
    /// The flows whose story the intent changes.
    pub flows: Vec<FlowId>,
    /// The id the flow a `LinkFrame` starts will have.
    pub new_flow: Option<FlowId>,
    /// Ids the intent will use for what it makes, in the order it uses them.
    pub fresh: Vec<String>,
    /// Intents that repeat on one frame (a drag, a nudge, typing) merge into one undo step.
    pub merge: Option<MergeKey>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MergeKey {
    Move(FrameId),
    Moves(Vec<FrameId>),
    Text(FrameId),
    Paragraph(FlowId, usize),
    Style(FrameId),
    Shape(FrameId),
    Focus(FrameId),
    Section(SpreadId),
    Cell(FrameId, usize, usize),
    Background(BackgroundTarget),
}

/// Whether `asset` is a path inside the book's own pictures.
fn is_asset_path(asset: &str) -> bool {
    asset.starts_with("assets/images/")
        && !asset.split('/').any(|part| part == ".." || part.is_empty())
}

fn spread<'a>(book: &'a Book, id: &SpreadId) -> Result<&'a Spread, SessionError> {
    book.spreads
        .iter()
        .find(|s| &s.id == id)
        .ok_or_else(|| SessionError::NoSuchSpread(id.0.clone()))
}

fn master<'a>(book: &'a Book, id: &MasterId) -> Result<&'a Master, SessionError> {
    book.master_pages
        .iter()
        .find(|m| &m.id == id)
        .ok_or_else(|| SessionError::NoSuchMaster(id.0.clone()))
}

/// The id of a surface, for saying which one an error is about.
fn named(on: &Surface) -> String {
    match on {
        Surface::Spread(s) => s.0.clone(),
        Surface::Master(m) => m.0.clone(),
    }
}

fn frames_on<'a>(book: &'a Book, on: &Surface) -> Result<&'a [Frame], SessionError> {
    match on {
        Surface::Spread(s) => Ok(&spread(book, s)?.frames),
        Surface::Master(m) => Ok(&master(book, m)?.frames),
    }
}

fn frames_on_mut<'a>(book: &'a mut Book, on: &Surface) -> Option<&'a mut Vec<Frame>> {
    match on {
        Surface::Spread(s) => book
            .spreads
            .iter_mut()
            .find(|sp| &sp.id == s)
            .map(|sp| &mut sp.frames),
        Surface::Master(m) => book
            .master_pages
            .iter_mut()
            .find(|mp| &mp.id == m)
            .map(|mp| &mut mp.frames),
    }
}

fn guides_on<'a>(book: &'a Book, on: &Surface) -> Result<&'a [Guide], SessionError> {
    match on {
        Surface::Spread(s) => Ok(&spread(book, s)?.guides),
        Surface::Master(m) => Ok(&master(book, m)?.guides),
    }
}

fn guides_on_mut<'a>(book: &'a mut Book, on: &Surface) -> Option<&'a mut Vec<Guide>> {
    match on {
        Surface::Spread(s) => book
            .spreads
            .iter_mut()
            .find(|sp| &sp.id == s)
            .map(|sp| &mut sp.guides),
        Surface::Master(m) => book
            .master_pages
            .iter_mut()
            .find(|mp| &mp.id == m)
            .map(|mp| &mut mp.guides),
    }
}

/// A frame anywhere in the book: on a spread, or on a master page.
fn locate<'a>(book: &'a Book, id: &FrameId) -> Option<&'a Frame> {
    book.spreads
        .iter()
        .find_map(|s| find(&s.frames, id))
        .or_else(|| book.master_pages.iter().find_map(|m| find(&m.frames, id)))
}

fn locate_mut<'a>(book: &'a mut Book, id: &FrameId) -> Option<&'a mut Frame> {
    let on_spread = book.spreads.iter().any(|s| find(&s.frames, id).is_some());
    if on_spread {
        book.spreads
            .iter_mut()
            .find_map(|s| find_mut(&mut s.frames, id))
    } else {
        book.master_pages
            .iter_mut()
            .find_map(|m| find_mut(&mut m.frames, id))
    }
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

/// The spreads a change to a frame touches: the one it is on, or none when it is on a master page,
/// which the session captures for every change.
fn touched_by(book: &Book, frame: &FrameId) -> Result<Vec<SpreadId>, SessionError> {
    if let Some(s) = book
        .spreads
        .iter()
        .find(|s| find(&s.frames, frame).is_some())
    {
        return Ok(vec![s.id.clone()]);
    }
    if book
        .master_pages
        .iter()
        .any(|m| find(&m.frames, frame).is_some())
    {
        return Ok(Vec::new());
    }
    Err(SessionError::NoSuchFrame(frame.0.clone()))
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

fn guide_at<'a>(book: &'a Book, on: &Surface, index: usize) -> Result<&'a Guide, SessionError> {
    guides_on(book, on)?
        .get(index)
        .ok_or_else(|| SessionError::NoSuchGuide(index, named(on)))
}

fn check_guide_free(book: &Book, on: &Surface, index: usize) -> Result<(), SessionError> {
    if guide_at(book, on, index)?.locked {
        Err(SessionError::GuideLocked(index))
    } else {
        Ok(())
    }
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
        Intent::AddSpread { .. }
            | Intent::DeleteSpread { .. }
            | Intent::MoveSpread { .. }
            | Intent::AddFlowPages { .. }
            | Intent::TrimFlowPages { .. }
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
        flows: Vec::new(),
        new_flow: None,
        fresh: Vec::new(),
        merge,
    };
    // The spreads a change on `on` touches: a master page is captured for every change.
    let by = |on: &Surface, merge: Option<MergeKey>| Plan {
        touched: match on {
            Surface::Spread(s) => vec![s.clone()],
            Surface::Master(_) => Vec::new(),
        },
        new_spread: None,
        flows: Vec::new(),
        new_flow: None,
        fresh: Vec::new(),
        merge,
    };
    match intent {
        Intent::MoveFrame { on, frame, .. } => {
            let target = find(frames_on(book, on)?, frame)
                .ok_or_else(|| SessionError::NoSuchFrame(frame.0.clone()))?;
            ensure_free(target)?;
            Ok(by(on, Some(MergeKey::Move(frame.clone()))))
        }
        Intent::MoveFrames { on, moves } => {
            let frames = frames_on(book, on)?;
            let mut ids_moved = Vec::new();
            for m in moves {
                let target = find(frames, &m.frame)
                    .ok_or_else(|| SessionError::NoSuchFrame(m.frame.0.clone()))?;
                ensure_free(target)?;
                ids_moved.push(m.frame.clone());
            }
            ids_moved.sort_by(|a, b| a.0.cmp(&b.0));
            Ok(by(on, Some(MergeKey::Moves(ids_moved))))
        }
        Intent::SetImage {
            frame,
            asset,
            focus,
            ..
        } => {
            let found =
                locate(book, frame).ok_or_else(|| SessionError::NoSuchFrame(frame.0.clone()))?;
            let Frame::Image(_) = found else {
                return Err(SessionError::NotImage(frame.0.clone()));
            };
            ensure_free(found)?;
            if let Some((x, y)) = focus {
                if !x.is_finite() || !y.is_finite() {
                    return Err(SessionError::BadFocus);
                }
            }
            if let Some(asset) = asset {
                if !is_asset_path(asset) {
                    return Err(SessionError::BadAsset(asset.clone()));
                }
            }
            Ok(Plan {
                touched: touched_by(book, frame)?,
                new_spread: None,
                flows: Vec::new(),
                new_flow: None,
                fresh: Vec::new(),
                merge: focus.is_some().then(|| MergeKey::Focus(frame.clone())),
            })
        }
        Intent::SetLanguage { language } => {
            clean_language(language).map_err(SessionError::BadLanguage)?;
            Ok(Plan {
                touched: Vec::new(),
                new_spread: None,
                flows: Vec::new(),
                new_flow: None,
                fresh: Vec::new(),
                merge: None,
            })
        }
        Intent::SetFonts { fonts } => {
            check_fonts(fonts)?;
            Ok(Plan {
                touched: Vec::new(),
                new_spread: None,
                flows: Vec::new(),
                new_flow: None,
                fresh: Vec::new(),
                merge: None,
            })
        }
        Intent::SetParagraphText { flow, para, .. } => {
            let story = story_of(book, flow)?;
            if *para >= story.len() {
                return Err(SessionError::BadStory(format!(
                    "no paragraph {para} in a story of {}",
                    story.len()
                )));
            }
            Ok(Plan {
                touched: Vec::new(),
                new_spread: None,
                flows: vec![flow.clone()],
                new_flow: None,
                fresh: Vec::new(),
                merge: Some(MergeKey::Paragraph(flow.clone(), *para)),
            })
        }
        Intent::SplitParagraph { flow, para, at } => {
            let story = story_of(book, flow)?;
            let length = story
                .get(*para)
                .map(|p| p.text().chars().count())
                .ok_or_else(|| {
                    SessionError::BadStory(format!(
                        "no paragraph {para} in a story of {}",
                        story.len()
                    ))
                })?;
            if *at > length {
                return Err(SessionError::BadStory(format!(
                    "character {at} in a paragraph of {length}"
                )));
            }
            Ok(Plan {
                touched: Vec::new(),
                new_spread: None,
                flows: vec![flow.clone()],
                new_flow: None,
                fresh: Vec::new(),
                merge: None,
            })
        }
        Intent::JoinParagraphs { flow, para } => {
            let story = story_of(book, flow)?;
            if para + 1 >= story.len() {
                return Err(SessionError::BadStory(format!(
                    "no paragraph after {para} in a story of {}",
                    story.len()
                )));
            }
            Ok(Plan {
                touched: Vec::new(),
                new_spread: None,
                flows: vec![flow.clone()],
                new_flow: None,
                fresh: Vec::new(),
                merge: None,
            })
        }
        Intent::SpliceStory {
            flow, at, remove, ..
        } => {
            let story = story_of(book, flow)?;
            if *at > story.len() || at.saturating_add(*remove) > story.len() {
                return Err(SessionError::BadStory(format!(
                    "paragraphs {at}..{} in a story of {}",
                    at.saturating_add(*remove),
                    story.len()
                )));
            }
            Ok(Plan {
                touched: Vec::new(),
                new_spread: None,
                flows: vec![flow.clone()],
                new_flow: None,
                fresh: Vec::new(),
                merge: None,
            })
        }
        Intent::StyleParagraphs {
            flow,
            from,
            to,
            style,
        } => {
            let story = story_of(book, flow)?;
            if from > to || *to >= story.len() {
                return Err(SessionError::BadStory(format!(
                    "paragraphs {from}..={to} in a story of {}",
                    story.len()
                )));
            }
            if style.as_ref().is_some_and(|s| s.0.is_empty()) {
                return Err(SessionError::BadStory("that style has no name".into()));
            }
            Ok(Plan {
                touched: Vec::new(),
                new_spread: None,
                flows: vec![flow.clone()],
                new_flow: None,
                fresh: Vec::new(),
                merge: None,
            })
        }
        Intent::StyleStoryRange {
            flow,
            para,
            start,
            end,
            change,
        } => {
            let story = story_of(book, flow)?;
            let length = story
                .get(*para)
                .map(|p| p.text().chars().count())
                .ok_or_else(|| {
                    SessionError::BadStory(format!(
                        "no paragraph {para} in a story of {}",
                        story.len()
                    ))
                })?;
            if start >= end || *end as usize > length || change.is_empty() {
                return Err(SessionError::BadRange(format!(
                    "{start}..{end} in words of {length} characters"
                )));
            }
            if change
                .colour
                .as_deref()
                .is_some_and(|c| !c.is_empty() && folio_model::Colour::parse(c).is_err())
            {
                return Err(SessionError::BadRange("that colour is not #rrggbb".into()));
            }
            Ok(Plan {
                touched: Vec::new(),
                new_spread: None,
                flows: vec![flow.clone()],
                new_flow: None,
                fresh: Vec::new(),
                merge: None,
            })
        }
        Intent::AddFlowPages { flow, count } => {
            const MOST: usize = 2000;
            let found = book
                .flows
                .iter()
                .find(|f| &f.id == flow)
                .ok_or_else(|| SessionError::NoSuchFlow(flow.0.clone()))?;
            let last = found.frames.last().ok_or_else(|| {
                SessionError::BadStory("that flow has no frame to continue".into())
            })?;
            if !matches!(locate(book, last), Some(Frame::Text(_))) {
                return Err(SessionError::NotText(last.0.clone()));
            }
            if *count == 0 || *count > MOST {
                return Err(SessionError::BadStory(format!(
                    "{count} frames is not between 1 and {MOST}"
                )));
            }
            // A spread, its two pages and a frame for each frame asked for, at most.
            let mut fresh: Vec<String> = Vec::with_capacity(count * 4);
            for _ in 0..*count {
                for prefix in ["sp", "pg", "pg", "fr"] {
                    fresh.push(ids(prefix));
                }
            }
            let mut touched: Vec<SpreadId> = fresh
                .chunks(4)
                .map(|ids| SpreadId(ids[0].clone()))
                .collect();
            touched.extend(touched_by(book, last)?);
            Ok(Plan {
                touched,
                new_spread: None,
                flows: vec![flow.clone()],
                new_flow: None,
                fresh,
                merge: None,
            })
        }
        Intent::TrimFlowPages { flow, keep } => {
            let found = book
                .flows
                .iter()
                .find(|f| &f.id == flow)
                .ok_or_else(|| SessionError::NoSuchFlow(flow.0.clone()))?;
            if *keep == 0 || *keep >= found.frames.len() {
                return Err(SessionError::BadStory(format!(
                    "keeping {keep} of {} frames",
                    found.frames.len()
                )));
            }
            let mut touched = Vec::new();
            for id in &found.frames[*keep..] {
                touched.extend(touched_by(book, id)?);
            }
            touched.sort_by(|a, b| a.0.cmp(&b.0));
            touched.dedup();
            Ok(Plan {
                touched,
                new_spread: None,
                flows: vec![flow.clone()],
                new_flow: None,
                fresh: Vec::new(),
                merge: None,
            })
        }
        Intent::LinkFrame { frame, to } => {
            let linkable = |id: &FrameId| -> Result<(), SessionError> {
                let found =
                    locate(book, id).ok_or_else(|| SessionError::NoSuchFrame(id.0.clone()))?;
                match found {
                    Frame::Text(_) => ensure_free(found),
                    _ => Err(SessionError::NotText(id.0.clone())),
                }
            };
            linkable(frame)?;
            linkable(to)?;
            let slot_of = |id: &FrameId| match locate(book, id) {
                Some(Frame::Text(t)) => t.flow.clone(),
                _ => None,
            };
            if frame == to || slot_of(frame).is_some() {
                return Err(SessionError::BadStory(
                    "that frame is already in a flow, or is the frame it would follow".into(),
                ));
            }
            let (flow, new_flow) = match slot_of(to) {
                Some(slot) => (slot.id, None),
                None => {
                    let id = FlowId(ids("fl"));
                    (id.clone(), Some(id))
                }
            };
            let mut touched = touched_by(book, frame)?;
            touched.extend(touched_by(book, to)?);
            // Every frame already in the flow moves along when the numbering changes.
            if let Some(f) = book.flows.iter().find(|f| f.id == flow) {
                for id in &f.frames {
                    for s in touched_by(book, id)? {
                        if !touched.contains(&s) {
                            touched.push(s);
                        }
                    }
                }
            }
            touched.sort_by(|a, b| a.0.cmp(&b.0));
            touched.dedup();
            Ok(Plan {
                touched,
                new_spread: None,
                flows: vec![flow],
                new_flow,
                fresh: Vec::new(),
                merge: None,
            })
        }
        Intent::UnlinkFrame { frame } => {
            let Some(Frame::Text(t)) = locate(book, frame) else {
                return Err(SessionError::NotText(frame.0.clone()));
            };
            let Some(slot) = t.flow.clone() else {
                return Err(SessionError::BadStory("that frame is not in a flow".into()));
            };
            ensure_free(locate(book, frame).expect("found above"))?;
            let mut touched = touched_by(book, frame)?;
            if let Some(f) = book.flows.iter().find(|f| f.id == slot.id) {
                for id in &f.frames {
                    for s in touched_by(book, id)? {
                        if !touched.contains(&s) {
                            touched.push(s);
                        }
                    }
                }
            }
            Ok(Plan {
                touched,
                new_spread: None,
                flows: vec![slot.id],
                new_flow: None,
                fresh: Vec::new(),
                merge: None,
            })
        }
        Intent::SetBarcode { choice } => {
            if let BarcodeChoice::Own { addon: Some(addon) } = choice {
                if !BarcodeChoice::addon_is_valid(addon) {
                    return Err(SessionError::BadBarcode(
                        "a price add-on is five digits".into(),
                    ));
                }
            }
            Ok(Plan {
                touched: Vec::new(),
                new_spread: None,
                flows: Vec::new(),
                new_flow: None,
                fresh: Vec::new(),
                merge: None,
            })
        }
        Intent::SetPaper { .. } => Ok(Plan {
            touched: Vec::new(),
            new_spread: None,
            flows: Vec::new(),
            new_flow: None,
            fresh: Vec::new(),
            merge: None,
        }),
        Intent::SetSwatch { swatch } => {
            check_swatch(book, swatch)?;
            Ok(Plan {
                touched: Vec::new(),
                new_spread: None,
                flows: Vec::new(),
                new_flow: None,
                fresh: Vec::new(),
                merge: None,
            })
        }
        Intent::RemoveSwatch { id } => {
            if !book.styles.swatches.iter().any(|s| s.id == *id) {
                return Err(SessionError::BadSwatch(format!(
                    "there is no swatch {}",
                    id.0
                )));
            }
            // The spreads that refer to it are the ones that change.
            let reference = format!("swatch:{}", id.0);
            let mut touched = Vec::new();
            for spread in &book.spreads {
                let mut copy = spread.clone();
                let mut uses = false;
                folio_model::visit_spread(&mut copy, &mut |c| uses |= c.0 == reference);
                if uses {
                    touched.push(spread.id.clone());
                }
            }
            Ok(Plan {
                touched,
                new_spread: None,
                flows: Vec::new(),
                new_flow: None,
                fresh: Vec::new(),
                merge: None,
            })
        }
        Intent::SetBackNotes { .. } => Ok(Plan {
            touched: Vec::new(),
            new_spread: None,
            flows: Vec::new(),
            new_flow: None,
            fresh: Vec::new(),
            merge: None,
        }),
        Intent::SetDetails { details } => {
            check_details(details)?;
            Ok(Plan {
                touched: Vec::new(),
                new_spread: None,
                flows: Vec::new(),
                new_flow: None,
                fresh: Vec::new(),
                merge: None,
            })
        }
        Intent::SetPreflightIgnore { keys } => {
            if keys.len() > MAX_IGNORED_ISSUES
                || keys.iter().any(|k| k.is_empty() || k.chars().count() > 240)
            {
                return Err(SessionError::BadIgnore(format!(
                    "at most {MAX_IGNORED_ISSUES} issues, each with a key"
                )));
            }
            Ok(Plan {
                touched: Vec::new(),
                new_spread: None,
                flows: Vec::new(),
                new_flow: None,
                fresh: Vec::new(),
                merge: None,
            })
        }
        Intent::EndUndoStep => Ok(Plan {
            touched: Vec::new(),
            new_spread: None,
            flows: Vec::new(),
            new_flow: None,
            fresh: Vec::new(),
            merge: None,
        }),
        Intent::EnsureMasters => Ok(Plan {
            touched: book.spreads.iter().map(|s| s.id.clone()).collect(),
            new_spread: None,
            flows: Vec::new(),
            new_flow: None,
            fresh: Vec::new(),
            merge: None,
        }),
        Intent::IgnoreWord { word } => {
            let word = Proofing::clean(word).map_err(SessionError::BadWord)?;
            if book.proofing.ignore.contains(&word) {
                return Err(SessionError::BadWord(format!(
                    "`{word}` is already ignored"
                )));
            }
            if book.proofing.ignore.len() >= MAX_IGNORED_WORDS {
                return Err(SessionError::BadWord("the ignore list is full".into()));
            }
            Ok(Plan {
                touched: Vec::new(),
                new_spread: None,
                flows: Vec::new(),
                new_flow: None,
                fresh: Vec::new(),
                merge: None,
            })
        }
        Intent::UnignoreWord { word } => {
            if !book.proofing.ignore.iter().any(|w| w == word.trim()) {
                return Err(SessionError::BadWord(format!("`{word}` is not ignored")));
            }
            Ok(Plan {
                touched: Vec::new(),
                new_spread: None,
                flows: Vec::new(),
                new_flow: None,
                fresh: Vec::new(),
                merge: None,
            })
        }
        Intent::SetSection {
            spread: id,
            section,
        } => {
            let found = spread(book, id)?;
            if found.kind == SpreadKind::Cover {
                return Err(SessionError::BadSection(
                    "the cover cannot start a section".into(),
                ));
            }
            if let Some(section) = section {
                section.checked().map_err(SessionError::BadSection)?;
            }
            // Later pages are numbered from this one, so their words change with it, but their
            // spreads do not: only this one is touched.
            // Trying a name or a number is one step; taking the section away is its own.
            Ok(one(
                id.clone(),
                section.is_some().then(|| MergeKey::Section(id.clone())),
            ))
        }
        Intent::SetPageNumbers { .. } => Ok(Plan {
            touched: book.spreads.iter().map(|s| s.id.clone()).collect(),
            new_spread: None,
            flows: Vec::new(),
            new_flow: None,
            fresh: Vec::new(),
            merge: None,
        }),
        Intent::SetBackground { target, bg } => {
            if let Some(bg) = bg {
                bg.checked(is_asset_path)
                    .map_err(SessionError::BadBackground)?;
                if !folio_model::colours_of(bg)
                    .iter()
                    .all(|c| swatch_is_there(book, c))
                {
                    return Err(SessionError::BadBackground(
                        "that swatch is not in the book".into(),
                    ));
                }
            }
            // Trying colours on one page is one undo step; taking the background off is its own.
            let merge = bg.is_some().then(|| MergeKey::Background(target.clone()));
            match target {
                BackgroundTarget::Page { spread: s, page } => {
                    let found = spread(book, s)?;
                    if !found.pages.iter().any(|p| &p.id == page) {
                        return Err(SessionError::NotOnSpread(page.0.clone(), s.0.clone()));
                    }
                    Ok(one(s.clone(), merge))
                }
                BackgroundTarget::Span { spread: s } => {
                    if spread(book, s)?.pages.len() < 2 {
                        return Err(SessionError::BadBackground(
                            "a background across both pages needs two pages".into(),
                        ));
                    }
                    Ok(one(s.clone(), merge))
                }
                // The masters may be made and given to pages, so every spread is part of it.
                BackgroundTarget::EveryPage => Ok(Plan {
                    touched: book.spreads.iter().map(|s| s.id.clone()).collect(),
                    new_spread: None,
                    flows: Vec::new(),
                    new_flow: None,
                    fresh: Vec::new(),
                    merge,
                }),
            }
        }
        Intent::HideMasterItem {
            spread: s,
            page,
            frame,
            ..
        }
        | Intent::CopyMasterItem {
            spread: s,
            page,
            frame,
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
            // Copying what the page already hides would leave a second copy on top of the first.
            if matches!(intent, Intent::CopyMasterItem { .. })
                && found
                    .master_overrides
                    .iter()
                    .any(|o| &o.page_id == page && o.hide.contains(frame))
            {
                return Err(SessionError::AlreadyHidden(frame.0.clone(), page.0.clone()));
            }
            Ok(one(s.clone(), None))
        }
        Intent::SetShape {
            frame,
            kind,
            fill,
            stroke,
            stroke_width,
            radius,
            flip,
        } => {
            let found =
                locate(book, frame).ok_or_else(|| SessionError::NoSuchFrame(frame.0.clone()))?;
            let Frame::Shape(_) = found else {
                return Err(SessionError::NotShape(frame.0.clone()));
            };
            ensure_free(found)?;
            for paint in [fill, stroke].into_iter().flatten() {
                if let Paint::Colour { colour } = paint {
                    Colour::parse(&colour.0).map_err(|e| SessionError::BadShape(e.to_string()))?;
                    if !swatch_is_there(book, colour) {
                        return Err(SessionError::BadShape(
                            "that swatch is not in the book".into(),
                        ));
                    }
                }
            }
            for (name, length, most) in [
                ("outline width", stroke_width, MAX_STROKE),
                ("corner radius", radius, MAX_RADIUS),
            ] {
                if let Some(Pt(v)) = length {
                    if !v.is_finite() || *v < 0.0 || *v > most {
                        return Err(SessionError::BadShape(format!(
                            "the {name} is between 0 and {most} points"
                        )));
                    }
                }
            }
            // Trying colours, widths and radii on one shape is one undo step. Changing the kind or the
            // way a line runs, or taking a colour away, is a step of its own.
            let trying = kind.is_none()
                && flip.is_none()
                && !matches!(fill, Some(Paint::None))
                && !matches!(stroke, Some(Paint::None));
            Ok(Plan {
                touched: touched_by(book, frame)?,
                new_spread: None,
                flows: Vec::new(),
                new_flow: None,
                fresh: Vec::new(),
                merge: trying.then(|| MergeKey::Shape(frame.clone())),
            })
        }
        Intent::RotateFrame { on, frame, degrees } => {
            let target = find(frames_on(book, on)?, frame)
                .ok_or_else(|| SessionError::NoSuchFrame(frame.0.clone()))?;
            ensure_free(target)?;
            if !degrees.is_finite() {
                return Err(SessionError::BadShape("an angle is a number".into()));
            }
            Ok(by(on, None))
        }
        Intent::AddGuide { on, pos, .. } => {
            guides_on(book, on)?;
            check_guide_position(*pos)?;
            Ok(by(on, None))
        }
        Intent::MoveGuide { on, index, pos } => {
            check_guide_position(*pos)?;
            check_guide_free(book, on, *index)?;
            Ok(by(on, None))
        }
        Intent::LockGuide { on, index, .. } => {
            guide_at(book, on, *index)?;
            Ok(by(on, None))
        }
        Intent::RemoveGuide { on, index } => {
            check_guide_free(book, on, *index)?;
            Ok(by(on, None))
        }
        Intent::PromoteGuide { spread: s, index } => {
            let guide = *guide_at(book, &Surface::Spread(s.clone()), *index)?;
            if guide.locked {
                return Err(SessionError::GuideLocked(*index));
            }
            if !can_promote(book, spread(book, s)?, &guide) {
                return Err(SessionError::CannotPromote);
            }
            // Making the masters gives pages on other spreads theirs, so they are part of the change.
            let unassigned = book.master_pages.is_empty()
                || book
                    .spreads
                    .iter()
                    .filter(|sp| sp.kind != SpreadKind::Cover)
                    .any(|sp| sp.pages.iter().any(|p| p.master.is_none()));
            if unassigned {
                return Ok(Plan {
                    touched: book.spreads.iter().map(|sp| sp.id.clone()).collect(),
                    new_spread: None,
                    flows: Vec::new(),
                    new_flow: None,
                    fresh: Vec::new(),
                    merge: None,
                });
            }
            Ok(one(s.clone(), None))
        }
        Intent::DemoteGuide {
            master: m,
            index,
            spread: s,
        } => {
            let on = Surface::Master(m.clone());
            check_guide_free(book, &on, *index)?;
            if page_using(spread(book, s)?, m).is_none() {
                return Err(SessionError::MasterNotOnSpread(m.0.clone(), s.0.clone()));
            }
            Ok(one(s.clone(), None))
        }
        Intent::ClearGuides { spread: only } => {
            if let Some(s) = only {
                spread(book, s)?;
            }
            Ok(Plan {
                touched: book
                    .spreads
                    .iter()
                    .filter(|sp| only.as_ref().is_none_or(|n| n == &sp.id))
                    .filter(|sp| sp.guides.iter().any(|g| !g.locked))
                    .map(|sp| sp.id.clone())
                    .collect(),
                new_spread: None,
                flows: Vec::new(),
                new_flow: None,
                fresh: Vec::new(),
                merge: None,
            })
        }
        Intent::SetText { frame, .. } => {
            check_text(book, frame)?;
            Ok(Plan {
                touched: touched_by(book, frame)?,
                new_spread: None,
                flows: Vec::new(),
                new_flow: None,
                fresh: Vec::new(),
                merge: Some(MergeKey::Text(frame.clone())),
            })
        }
        Intent::StyleRange {
            frame,
            start,
            end,
            change,
        } => {
            check_text(book, frame)?;
            let length = words_of(book, frame)
                .map(|runs| folio_model::text_of(runs).chars().count())
                .unwrap_or(0);
            if start >= end || *end as usize > length || change.is_empty() {
                return Err(SessionError::BadRange(format!(
                    "{start}..{end} in words of {length} characters"
                )));
            }
            if change.colour.as_deref().is_some_and(|c| {
                !c.is_empty()
                    && folio_model::Colour::parse(c)
                        .map_or(true, |colour| !swatch_is_there(book, &colour))
            }) {
                return Err(SessionError::BadRange(
                    "that colour is not #rrggbb or a swatch of the book".into(),
                ));
            }
            Ok(Plan {
                touched: touched_by(book, frame)?,
                new_spread: None,
                flows: Vec::new(),
                new_flow: None,
                fresh: Vec::new(),
                merge: None,
            })
        }
        Intent::EditTable { frame, edit } => {
            let Some(found) = locate(book, frame) else {
                return Err(SessionError::NoSuchFrame(frame.0.clone()));
            };
            let Frame::Table(table) = found else {
                return Err(SessionError::NotTable(frame.0.clone()));
            };
            ensure_free(found)?;
            let mut after = table.clone();
            edit_table(&mut after, edit);
            after.check().map_err(SessionError::BadTable)?;
            // Edits that run past the ends are refused rather than clamped, so a stale UI is told.
            let (rows, cols) = (table.rows.len(), table.cols.len());
            let in_range = match edit {
                TableEdit::SetCell { row, col, .. } => *row < rows && *col < cols,
                TableEdit::AddRow { at } => *at <= rows,
                TableEdit::RemoveRow { at } => *at < rows,
                TableEdit::AddColumn { at } => *at <= cols,
                TableEdit::RemoveColumn { at } => *at < cols,
                TableEdit::SetHead { .. } | TableEdit::SetLook { .. } => true,
            };
            if !in_range {
                return Err(SessionError::BadTable(
                    "that row or column is not there".into(),
                ));
            }
            Ok(Plan {
                touched: touched_by(book, frame)?,
                new_spread: None,
                flows: Vec::new(),
                new_flow: None,
                fresh: Vec::new(),
                merge: match edit {
                    TableEdit::SetCell { row, col, .. } => {
                        Some(MergeKey::Cell(frame.clone(), *row, *col))
                    }
                    _ => None,
                },
            })
        }
        Intent::SetStyle { frame, .. } => {
            let touched = touched_by(book, frame)?;
            let styled =
                locate(book, frame).is_some_and(|f| matches!(f, Frame::Text(_) | Frame::Shape(_)));
            if !styled {
                return Err(SessionError::NotText(frame.0.clone()));
            }
            if let Some(found) = locate(book, frame) {
                ensure_free(found)?;
            }
            Ok(Plan {
                touched,
                new_spread: None,
                flows: Vec::new(),
                new_flow: None,
                fresh: Vec::new(),
                merge: None,
            })
        }
        Intent::OverrideStyle { frame, .. } => {
            check_text(book, frame).or_else(|e| match e {
                // Overrides sit on a frame whose words come from a flow too.
                SessionError::FlowText(_) => locate(book, frame).map_or(Ok(()), ensure_free),
                other => Err(other),
            })?;
            Ok(Plan {
                touched: touched_by(book, frame)?,
                new_spread: None,
                flows: Vec::new(),
                new_flow: None,
                fresh: Vec::new(),
                merge: Some(MergeKey::Style(frame.clone())),
            })
        }
        Intent::AddFrame { on, frame } => {
            frames_on(book, on)?;
            if let Frame::Table(table) = frame {
                table.check().map_err(SessionError::BadTable)?;
            }
            let mut new_ids = Vec::new();
            every_id(frame, &mut new_ids);
            for id in new_ids {
                if locate(book, &id).is_some() {
                    return Err(SessionError::DuplicateFrame(id.0));
                }
            }
            Ok(by(on, None))
        }
        Intent::DeleteFrame { on, frame } => {
            let target = find(frames_on(book, on)?, frame)
                .ok_or_else(|| SessionError::NoSuchFrame(frame.0.clone()))?;
            ensure_free(target)?;
            if matches!(on, Surface::Master(_)) {
                if crate::masters::is_number(target) {
                    return Err(SessionError::PageNumber(frame.0.clone()));
                }
                // Pages that hid the frame forget it, so they are part of the change.
                let mut plan = by(on, None);
                plan.touched = book
                    .spreads
                    .iter()
                    .filter(|s| s.master_overrides.iter().any(|o| o.hide.contains(frame)))
                    .map(|s| s.id.clone())
                    .collect();
                return Ok(plan);
            }
            let mut plan = by(on, None);
            // A frame in a flow leaves its chain, and the frames after it are numbered again.
            if let Frame::Text(t) = target {
                if let Some(slot) = &t.flow {
                    plan.flows.push(slot.id.clone());
                    if let Some(f) = book.flows.iter().find(|f| f.id == slot.id) {
                        for id in &f.frames {
                            for s in touched_by(book, id)? {
                                if !plan.touched.contains(&s) {
                                    plan.touched.push(s);
                                }
                            }
                        }
                    }
                }
            }
            Ok(plan)
        }
        Intent::DuplicateFrame { on, frame } | Intent::LockFrame { on, frame, .. } => {
            let frames = frames_on(book, on)?;
            if !frames.iter().any(|f| &f.base().id == frame) {
                return Err(SessionError::NotOnSpread(frame.0.clone(), named(on)));
            }
            Ok(by(on, None))
        }
        Intent::Reorder { on, frame, .. } => {
            let target = frames_on(book, on)?
                .iter()
                .find(|f| &f.base().id == frame)
                .ok_or_else(|| SessionError::NotOnSpread(frame.0.clone(), named(on)))?;
            ensure_free(target)?;
            Ok(by(on, None))
        }
        Intent::AddSpread { after, .. } => {
            spread(book, after)?;
            let id = SpreadId(ids("sp"));
            Ok(Plan {
                touched: vec![id.clone()],
                new_spread: Some(id),
                flows: Vec::new(),
                new_flow: None,
                fresh: Vec::new(),
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
                flows: Vec::new(),
                new_flow: None,
                fresh: Vec::new(),
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
                flows: Vec::new(),
                new_flow: None,
                fresh: Vec::new(),
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
        Intent::AddSpread { .. }
            | Intent::DeleteSpread { .. }
            | Intent::MoveSpread { .. }
            | Intent::AddFlowPages { .. }
            | Intent::TrimFlowPages { .. }
    );
    run_one(book, intent, plan, ids);
    if restructures {
        apply_page_numbers(book);
    }
}

fn run_one(book: &mut Book, intent: Intent, plan: &Plan, ids: &mut IdSource) {
    match intent {
        Intent::MoveFrame { on, frame, rect } => {
            if let Some(f) = frames_on_mut(book, &on).and_then(|frames| find_mut(frames, &frame)) {
                f.base_mut().rect = rect;
            }
        }
        Intent::SetImage {
            frame,
            asset,
            fit,
            focus,
        } => {
            if let Some(Frame::Image(img)) = locate_mut(book, &frame) {
                if let Some(asset) = asset {
                    img.image.asset = Some(asset);
                }
                if let Some(fit) = fit {
                    img.image.fit = fit;
                }
                if let Some((x, y)) = focus {
                    img.image.focus = (x.clamp(0.0, 1.0), y.clamp(0.0, 1.0));
                }
            }
        }
        Intent::SetShape {
            frame,
            kind,
            fill,
            stroke,
            stroke_width,
            radius,
            flip,
        } => {
            if let Some(Frame::Shape(shape)) = locate_mut(book, &frame) {
                let colour = |paint: Paint| match paint {
                    Paint::None => None,
                    Paint::Colour { colour } => Some(Colour::parse(&colour.0).unwrap_or(colour)),
                };
                if let Some(kind) = kind {
                    shape.shape = kind;
                    // A line has no inside, so words it was holding would be hidden and unreachable.
                    if !kind.holds_text() {
                        shape.content = None;
                        shape.overrides = None;
                    }
                }
                if let Some(fill) = fill {
                    shape.fill = colour(fill);
                }
                if let Some(stroke) = stroke {
                    shape.stroke = colour(stroke);
                }
                if stroke_width.is_some() {
                    shape.stroke_width = stroke_width;
                }
                if radius.is_some() {
                    shape.radius = radius;
                }
                if let Some(flip) = flip {
                    shape.flip = flip;
                }
            }
        }
        Intent::RotateFrame { on, frame, degrees } => {
            if let Some(f) = frames_on_mut(book, &on).and_then(|frames| find_mut(frames, &frame)) {
                f.base_mut().rotation = degrees.rem_euclid(360.0);
            }
        }
        Intent::EnsureMasters => ensure_masters(book),
        Intent::SetLanguage { language } => {
            if let Ok(language) = clean_language(&language) {
                book.language = language;
            }
        }
        Intent::SetFonts { fonts } => book.fonts = fonts,
        Intent::SetBarcode { choice } => book.barcode = choice,
        Intent::SetPaper { paper } => book.paper = paper,
        Intent::SetParagraphText { flow, para, text } => {
            if let Some(p) = flow_mut(book, &flow).and_then(|f| f.paragraphs.get_mut(para)) {
                p.inline = folio_model::retext(&p.inline, &text);
            }
        }
        Intent::SplitParagraph { flow, para, at } => {
            if let Some(f) = flow_mut(book, &flow) {
                if let Some(p) = f.paragraphs.get(para) {
                    let (before, after) = folio_model::split_runs(&p.inline, at);
                    let style = p.style.clone();
                    f.paragraphs[para].inline = before;
                    f.paragraphs.insert(
                        para + 1,
                        StoryPara {
                            style,
                            inline: after,
                        },
                    );
                }
            }
        }
        Intent::JoinParagraphs { flow, para } => {
            if let Some(f) = flow_mut(book, &flow) {
                if para + 1 < f.paragraphs.len() {
                    let next = f.paragraphs.remove(para + 1);
                    f.paragraphs[para].inline =
                        folio_model::join_runs(&f.paragraphs[para].inline, &next.inline);
                }
            }
        }
        Intent::SpliceStory {
            flow,
            at,
            remove,
            insert,
        } => {
            if let Some(f) = flow_mut(book, &flow) {
                f.paragraphs.splice(at..at + remove, insert);
            }
        }
        Intent::StyleParagraphs {
            flow,
            from,
            to,
            style,
        } => {
            if let Some(f) = flow_mut(book, &flow) {
                for p in &mut f.paragraphs[from..=to] {
                    p.style = style.clone();
                }
            }
        }
        Intent::StyleStoryRange {
            flow,
            para,
            start,
            end,
            change,
        } => {
            if let Some(p) = flow_mut(book, &flow).and_then(|f| f.paragraphs.get_mut(para)) {
                p.inline =
                    folio_model::style_range(&p.inline, start as usize, end as usize, &change);
            }
        }
        Intent::TrimFlowPages { flow, keep } => trim_flow_pages(book, &flow, keep),
        Intent::AddFlowPages { flow, count } => {
            add_flow_pages(book, &flow, count, &plan.fresh);
        }
        Intent::LinkFrame { frame, to } => {
            // The frame's own words go to the end of the story.
            let own = match locate_mut(book, &frame) {
                Some(Frame::Text(t)) => t.content.take().map(|c| c.inline).unwrap_or_default(),
                _ => Vec::new(),
            };
            let flow_id = match plan.new_flow.clone() {
                Some(id) => {
                    // `to` starts the flow, with its words as the story.
                    let words = match locate_mut(book, &to) {
                        Some(Frame::Text(t)) => {
                            t.content.take().map(|c| c.inline).unwrap_or_default()
                        }
                        _ => Vec::new(),
                    };
                    book.flows.push(folio_model::Flow {
                        id: id.clone(),
                        paragraphs: StoryPara::from_runs(&words),
                        frames: vec![to.clone()],
                        auto_add_pages: true,
                    });
                    id
                }
                None => plan.flows[0].clone(),
            };
            if let Some(f) = flow_mut(book, &flow_id) {
                let at = f
                    .frames
                    .iter()
                    .position(|id| *id == to)
                    .map_or(f.frames.len(), |i| i + 1);
                f.frames.insert(at, frame.clone());
                if own.iter().any(|r| !r.text.is_empty()) {
                    f.paragraphs.extend(StoryPara::from_runs(&own));
                }
            }
            renumber(book, &flow_id);
        }
        Intent::UnlinkFrame { frame } => {
            let flow_id = plan.flows[0].clone();
            let alone = flow_mut(book, &flow_id).is_some_and(|f| f.frames.len() <= 1);
            let story = if alone {
                book.flows
                    .iter()
                    .position(|f| f.id == flow_id)
                    .map(|i| book.flows.remove(i).runs())
            } else {
                if let Some(f) = flow_mut(book, &flow_id) {
                    f.frames.retain(|id| *id != frame);
                }
                None
            };
            if let Some(Frame::Text(t)) = locate_mut(book, &frame) {
                t.flow = None;
                if let Some(runs) = story {
                    t.content = Some(InlineContent { inline: runs });
                }
            }
            renumber(book, &flow_id);
        }
        Intent::SetBackNotes { notes } => book.back_notes = notes,
        Intent::SetSwatch { mut swatch } => {
            if let Some(ink) = swatch.ink {
                let [r, g, b] = ink.to_rgb();
                swatch.display = Colour(format!("#{r:02x}{g:02x}{b:02x}"));
            }
            match book.styles.swatches.iter_mut().find(|s| s.id == swatch.id) {
                Some(slot) => *slot = swatch,
                None => book.styles.swatches.push(swatch),
            }
        }
        Intent::RemoveSwatch { id } => {
            if let Some(at) = book.styles.swatches.iter().position(|s| s.id == id) {
                let gone = book.styles.swatches.remove(at);
                let reference = format!("swatch:{}", id.0);
                folio_model::visit_colours(book, &mut |c| {
                    if c.0 == reference {
                        *c = gone.display.clone();
                    }
                });
            }
        }
        Intent::SetDetails { details } => details.apply(book),
        Intent::SetPreflightIgnore { keys } => book.preflight_ignore = keys,
        Intent::EndUndoStep => {}
        Intent::IgnoreWord { word } => {
            if let Ok(word) = Proofing::clean(&word) {
                if !book.proofing.ignore.contains(&word) {
                    book.proofing.ignore.push(word);
                    book.proofing.ignore.sort();
                }
            }
        }
        Intent::UnignoreWord { word } => {
            let word = word.trim();
            book.proofing.ignore.retain(|w| w != word);
        }
        Intent::SetSection {
            spread: id,
            section,
        } => {
            let section = section.map(|s| s.checked().unwrap_or(s));
            if let Some(sp) = book.spreads.iter_mut().find(|sp| sp.id == id) {
                sp.section = section;
            }
        }
        Intent::SetPageNumbers { position } => {
            book.page_numbers = position;
            apply_page_numbers(book);
        }
        Intent::SetBackground { target, bg } => {
            // The plan checked it; store it the way it is kept.
            let bg = bg.map(|b| b.checked(is_asset_path).unwrap_or(b));
            match target {
                BackgroundTarget::Page { spread: s, page } => {
                    if let Some(p) = book
                        .spreads
                        .iter_mut()
                        .find(|sp| sp.id == s)
                        .and_then(|sp| sp.pages.iter_mut().find(|p| p.id == page))
                    {
                        p.bg = bg;
                    }
                }
                BackgroundTarget::Span { spread: s } => {
                    if let Some(sp) = book.spreads.iter_mut().find(|sp| sp.id == s) {
                        sp.bg_span = bg;
                    }
                }
                BackgroundTarget::EveryPage => {
                    // Clearing needs no master that is not already there.
                    if bg.is_some() {
                        ensure_masters(book);
                    }
                    let used: Vec<_> = book
                        .spreads
                        .iter()
                        .flat_map(|sp| sp.pages.iter().filter_map(|p| p.master.clone()))
                        .collect();
                    for m in book
                        .master_pages
                        .iter_mut()
                        .filter(|m| used.contains(&m.id))
                    {
                        m.bg = bg.clone();
                    }
                }
            }
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
        Intent::CopyMasterItem {
            spread: s,
            page,
            frame,
        } => {
            let Some(at) = book.spreads.iter().position(|sp| sp.id == s) else {
                return;
            };
            let Some(index) = book.spreads[at].pages.iter().position(|p| p.id == page) else {
                return;
            };
            let source = book.spreads[at].pages[index]
                .master
                .as_ref()
                .and_then(|m| book.master_pages.iter().find(|mp| &mp.id == m))
                .and_then(|mp| mp.frames.iter().find(|f| f.base().id == frame))
                .cloned();
            let Some(source) = source else {
                return;
            };
            let mut copy = renewed(&source, ids);
            let base = copy.base_mut();
            // A master frame sits at a point on its page; the copy sits at the same point on the spread.
            base.rect.x = Pt(base.rect.x.0 + book.trim.w.0 * index as f64);
            base.locked = false;
            base.from_master = None;
            if let Frame::Text(t) = &mut copy {
                t.flow = None;
            }
            let sp = &mut book.spreads[at];
            // The page's own copy goes where the master's item was drawn, beneath the page's frames.
            sp.frames.insert(0, copy);
            match sp.master_overrides.iter_mut().find(|o| o.page_id == page) {
                Some(o) => {
                    if !o.hide.contains(&frame) {
                        o.hide.push(frame);
                    }
                }
                None => sp.master_overrides.push(folio_model::MasterOverride {
                    page_id: page,
                    hide: vec![frame],
                }),
            }
        }
        Intent::AddGuide { on, axis, pos } => {
            if let Some(guides) = guides_on_mut(book, &on) {
                guides.push(Guide {
                    axis,
                    pos,
                    locked: false,
                });
            }
        }
        Intent::MoveGuide { on, index, pos } => {
            if let Some(g) = guides_on_mut(book, &on).and_then(|g| g.get_mut(index)) {
                g.pos = pos;
            }
        }
        Intent::LockGuide { on, index, locked } => {
            if let Some(g) = guides_on_mut(book, &on).and_then(|g| g.get_mut(index)) {
                g.locked = locked;
            }
        }
        Intent::RemoveGuide { on, index } => {
            if let Some(guides) = guides_on_mut(book, &on) {
                if index < guides.len() {
                    guides.remove(index);
                }
            }
        }
        Intent::PromoteGuide { spread: s, index } => {
            promote_guide(book, &s, index);
        }
        Intent::DemoteGuide {
            master: m,
            index,
            spread: s,
        } => {
            demote_guide(book, &m, index, &s);
        }
        Intent::ClearGuides { spread: only } => {
            for sp in book
                .spreads
                .iter_mut()
                .filter(|sp| only.as_ref().is_none_or(|n| n == &sp.id))
            {
                sp.guides.retain(|g| g.locked);
            }
        }
        Intent::StyleRange {
            frame,
            start,
            end,
            change,
        } => {
            let styled = |content: &mut Option<InlineContent>| {
                if let Some(content) = content {
                    content.inline = folio_model::style_range(
                        &content.inline,
                        start as usize,
                        end as usize,
                        &change,
                    );
                }
            };
            match locate_mut(book, &frame) {
                Some(Frame::Text(t)) => styled(&mut t.content),
                Some(Frame::Shape(s)) => styled(&mut s.content),
                _ => {}
            }
        }
        Intent::SetText { frame, text } => {
            // Words already there keep the look of the stretches that were not touched.
            let old: Vec<Run> = words_of(book, &frame)
                .map(<[Run]>::to_vec)
                .unwrap_or_default();
            let content = InlineContent {
                inline: folio_model::retext(&old, &text),
            };
            match locate_mut(book, &frame) {
                Some(Frame::Text(t)) => t.content = Some(content),
                // Emptied words are no words, so a bubble with none draws and lays out as bare.
                Some(Frame::Shape(s)) => {
                    s.content =
                        Some(content).filter(|c| c.inline.iter().any(|r| !r.text.is_empty()));
                }
                _ => {}
            }
        }
        Intent::EditTable { frame, edit } => {
            if let Some(Frame::Table(table)) = locate_mut(book, &frame) {
                edit_table(table, &edit);
            }
        }
        Intent::SetStyle { frame, style } => {
            if let Some(f) = locate_mut(book, &frame) {
                match f {
                    Frame::Text(t) => t.style = style,
                    Frame::Shape(sh) => sh.style = Some(style),
                    _ => {}
                }
            }
        }
        Intent::OverrideStyle { frame, overrides } => match locate_mut(book, &frame) {
            Some(Frame::Text(t)) => t.overrides = Some(overrides),
            Some(Frame::Shape(s)) => s.overrides = Some(overrides),
            _ => {}
        },
        Intent::AddFrame { on, frame } => {
            if let Some(frames) = frames_on_mut(book, &on) {
                frames.push(frame);
            }
        }
        Intent::DeleteFrame { on, frame } => {
            let flow = match locate(book, &frame) {
                Some(Frame::Text(t)) => t.flow.as_ref().map(|s| s.id.clone()),
                _ => None,
            };
            if let Some(frames) = frames_on_mut(book, &on) {
                remove(frames, &frame);
            }
            if let Some(flow) = flow {
                if let Some(f) = flow_mut(book, &flow) {
                    f.frames.retain(|id| *id != frame);
                }
                // A flow with no frame left has nowhere to be shown.
                book.flows.retain(|f| f.id != flow || !f.frames.is_empty());
                renumber(book, &flow);
            }
            if matches!(on, Surface::Master(_)) {
                for spread in &mut book.spreads {
                    for o in &mut spread.master_overrides {
                        o.hide.retain(|f| f != &frame);
                    }
                    spread.master_overrides.retain(|o| !o.hide.is_empty());
                }
            }
        }
        Intent::MoveFrames { on, moves } => {
            if let Some(frames) = frames_on_mut(book, &on) {
                for m in moves {
                    if let Some(f) = find_mut(frames, &m.frame) {
                        f.base_mut().rect = m.rect;
                    }
                }
            }
        }
        Intent::DuplicateFrame { on, frame } => {
            if let Some(frames) = frames_on_mut(book, &on) {
                if let Some(source) = frames.iter().find(|f| f.base().id == frame) {
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
                    frames.push(copy);
                }
            }
        }
        Intent::LockFrame { on, frame, locked } => {
            if let Some(f) = frames_on_mut(book, &on).and_then(|frames| find_mut(frames, &frame)) {
                f.base_mut().locked = locked;
            }
        }
        Intent::Reorder { on, frame, to } => {
            if let Some(frames) = frames_on_mut(book, &on) {
                if let Some(at) = frames.iter().position(|f| f.base().id == frame) {
                    let moved = frames.remove(at);
                    let last = frames.len();
                    let target = match to {
                        ReorderTo::Front => last,
                        ReorderTo::Back => 0,
                        ReorderTo::Index(i) => (i as usize).min(last),
                    };
                    frames.insert(target, moved);
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
                            bg: p.bg.clone(),
                        })
                        .collect(),
                    frames: source.frames.iter().map(|f| renewed(f, ids)).collect(),
                    guides: source.guides.clone(),
                    master_overrides: Vec::new(),
                    bg_span: source.bg_span.clone(),
                    section: None,
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
                            bg: None,
                        })
                        .collect(),
                    frames: Vec::new(),
                    guides: Vec::new(),
                    master_overrides: Vec::new(),
                    bg_span: None,
                    section: None,
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
    let found = locate(book, frame).ok_or_else(|| SessionError::NoSuchFrame(frame.0.clone()))?;
    match found {
        Frame::Text(t) if t.flow.is_some() => Err(SessionError::FlowText(frame.0.clone())),
        Frame::Text(_) => ensure_free(found),
        Frame::Shape(s) if s.shape.holds_text() => ensure_free(found),
        _ => Err(SessionError::NotText(frame.0.clone())),
    }
}

/// Applies `edit` to a table, leaving an edit that does not fit to the caller's check. The caller
/// checks the result with `TableFrame::check` and that the row or column is there.
fn edit_table(table: &mut TableFrame, edit: &TableEdit) {
    let rows = table.rows.len();
    let cols = table.cols.len();
    match edit {
        TableEdit::SetCell { row, col, text } => {
            if let Some(cell) = table.rows.get_mut(*row).and_then(|r| r.get_mut(*col)) {
                cell.content = (!text.is_empty()).then(|| InlineContent {
                    inline: vec![Run {
                        text: text.clone(),
                        style: None,
                        bold: false,
                        italic: false,
                        font: None,
                        colour: None,
                    }],
                });
            }
        }
        TableEdit::AddRow { at } if *at <= rows => {
            let row_h = table.base.rect.h.0 / rows.max(1) as f64;
            table.rows.insert(*at, vec![TableCell::default(); cols]);
            table.base.rect.h = Pt(row_h * (rows + 1) as f64);
            // A row put in above the header rows is not one of them.
            if *at < usize::from(table.head) {
                table.head = table.head.saturating_add(1);
            }
        }
        TableEdit::RemoveRow { at } if *at < rows => {
            let row_h = table.base.rect.h.0 / rows as f64;
            table.rows.remove(*at);
            table.base.rect.h = Pt(row_h * (rows - 1) as f64);
            if *at < usize::from(table.head) {
                table.head -= 1;
            }
            table.head = table.head.min(table.rows.len().min(255) as u8);
        }
        TableEdit::AddColumn { at } if *at <= cols => {
            let share = 1.0 / (cols + 1) as f64;
            for w in &mut table.cols {
                *w *= 1.0 - share;
            }
            table.cols.insert(*at, share);
            for row in &mut table.rows {
                row.insert(*at, TableCell::default());
            }
        }
        TableEdit::RemoveColumn { at } if *at < cols => {
            table.cols.remove(*at);
            let sum: f64 = table.cols.iter().sum();
            for w in &mut table.cols {
                *w /= sum;
            }
            for row in &mut table.rows {
                row.remove(*at);
            }
        }
        TableEdit::SetHead { head } => table.head = *head,
        TableEdit::SetLook { look } => table.look = *look,
        _ => {}
    }
}

/// Refuses details a book cannot have: no title, text that is too long, or an ISBN that is not an
/// ISBN-13 with a correct check digit.
fn check_details(details: &folio_model::Details) -> Result<(), SessionError> {
    let bad = |what: &str| Err(SessionError::BadDetails(what.to_owned()));
    if details.title.trim().is_empty() || details.title.chars().count() > MAX_TITLE {
        return bad("the title must be there and no longer than 300 characters");
    }
    if details
        .subtitle
        .as_deref()
        .is_some_and(|s| s.chars().count() > MAX_TITLE)
    {
        return bad("the subtitle is too long");
    }
    if details.description.chars().count() > MAX_DESCRIPTION {
        return bad("the description is too long");
    }
    if details.authors.len() > MAX_AUTHORS
        || details
            .authors
            .iter()
            .any(|a| a.name.trim().is_empty() || a.name.chars().count() > MAX_TITLE)
    {
        return bad("each author needs a name, and there can be at most 20");
    }
    for isbn in [
        &details.isbn.paperback,
        &details.isbn.hardcover,
        &details.isbn.ebook,
    ]
    .into_iter()
    .flatten()
    {
        if !isbn.is_empty() && !folio_model::is_valid_isbn13(isbn) {
            return bad("an ISBN is not a valid ISBN-13");
        }
    }
    Ok(())
}

/// The runs of the words a text frame or shape holds.
fn words_of<'a>(book: &'a Book, frame: &FrameId) -> Option<&'a [Run]> {
    match locate(book, frame)? {
        Frame::Text(t) => t.content.as_ref().map(|c| c.inline.as_slice()),
        Frame::Shape(s) => s.content.as_ref().map(|c| c.inline.as_slice()),
        _ => None,
    }
}

/// The paragraphs of a flow's story, or why there are none.
fn story_of<'a>(book: &'a Book, flow: &FlowId) -> Result<&'a [StoryPara], SessionError> {
    book.flows
        .iter()
        .find(|f| &f.id == flow)
        .map(|f| f.paragraphs.as_slice())
        .ok_or_else(|| SessionError::NoSuchFlow(flow.0.clone()))
}

fn flow_mut<'a>(book: &'a mut Book, flow: &FlowId) -> Option<&'a mut folio_model::Flow> {
    book.flows.iter_mut().find(|f| &f.id == flow)
}

/// Numbers the frames of a flow by their place in its chain.
fn renumber(book: &mut Book, flow: &FlowId) {
    let Some(frames) = book
        .flows
        .iter()
        .find(|f| &f.id == flow)
        .map(|f| f.frames.clone())
    else {
        return;
    };
    for (index, id) in frames.iter().enumerate() {
        if let Some(Frame::Text(t)) = locate_mut(book, id) {
            t.flow = Some(folio_model::FlowSlot {
                id: flow.clone(),
                index: index as u32,
            });
        }
    }
}

/// Adds `count` text frames to a flow, each like the last and after it, using `fresh` for the ids
/// of the spreads, pages and frames it makes (a spread, two pages and a frame for each).
fn add_flow_pages(book: &mut Book, flow: &FlowId, count: usize, fresh: &[String]) {
    let page_w = book.trim.w.0;
    let mut next = fresh.chunks(4);
    for _ in 0..count {
        let Some(ids) = next.next() else { return };
        let Some(last_id) = book
            .flows
            .iter()
            .find(|f| &f.id == flow)
            .and_then(|f| f.frames.last().cloned())
        else {
            return;
        };
        let Some(at) = book
            .spreads
            .iter()
            .position(|s| find(&s.frames, &last_id).is_some())
        else {
            return;
        };
        let Some(Frame::Text(last)) = find(&book.spreads[at].frames, &last_id).cloned() else {
            return;
        };
        let rect = last.base.rect;
        let on_left = rect.x.0 + rect.w.0 / 2.0 < page_w;
        // The frame's twin on the other page, as the pages face each other across the gutter.
        let mirrored = Rect {
            x: Pt(2.0 * page_w - rect.x.0 - rect.w.0),
            ..rect
        };
        let twin_free = |spread: &Spread| {
            !spread.frames.iter().any(|f| {
                let r = f.base().rect;
                r.x.0 < mirrored.x.0 + mirrored.w.0
                    && mirrored.x.0 < r.x.0 + r.w.0
                    && r.y.0 < mirrored.y.0 + mirrored.h.0
                    && mirrored.y.0 < r.y.0 + r.h.0
            })
        };
        let make = |rect: Rect| {
            let mut frame = last.clone();
            frame.base.id = FrameId(ids[3].clone());
            frame.base.rect = rect;
            frame.content = None;
            frame.flow = None;
            frame
        };
        let (target, rect) =
            if on_left && book.spreads[at].pages.len() == 2 && twin_free(&book.spreads[at]) {
                (at, mirrored)
            } else {
                book.spreads.insert(
                    at + 1,
                    Spread {
                        id: SpreadId(ids[0].clone()),
                        kind: folio_model::SpreadKind::Spread,
                        pages: ids[1..3]
                            .iter()
                            .map(|id| Page {
                                id: folio_model::PageId(id.clone()),
                                master: None,
                                bg: None,
                            })
                            .collect(),
                        frames: Vec::new(),
                        guides: Vec::new(),
                        master_overrides: Vec::new(),
                        bg_span: None,
                        section: None,
                    },
                );
                // The first frame of a new spread is on its left page: like the last when that was on
                // the left, and its twin when that was on the right.
                (at + 1, if on_left { rect } else { mirrored })
            };
        let frame = make(rect);
        let id = frame.base.id.clone();
        book.spreads[target].frames.push(Frame::Text(frame));
        if let Some(f) = flow_mut(book, flow) {
            f.frames.push(id);
        }
        renumber(book, flow);
    }
}

/// Takes the frames of a flow after the first `keep` off their spreads and out of the flow, and
/// removes a spread they leave with nothing on it (no frame, background or guide). The cover and the
/// last spread of the book are never removed.
fn trim_flow_pages(book: &mut Book, flow: &FlowId, keep: usize) {
    let gone: Vec<FrameId> = match book.flows.iter().find(|f| &f.id == flow) {
        Some(f) if keep < f.frames.len() => f.frames[keep..].to_vec(),
        _ => return,
    };
    // The spreads that held the frames: the only ones that may be emptied by taking them off.
    let mut held: Vec<SpreadId> = Vec::new();
    for id in &gone {
        for spread in &mut book.spreads {
            if remove(&mut spread.frames, id) && !held.contains(&spread.id) {
                held.push(spread.id.clone());
            }
        }
    }
    if let Some(f) = flow_mut(book, flow) {
        f.frames.truncate(keep);
    }
    let empty = |s: &Spread| {
        s.frames.is_empty()
            && s.guides.is_empty()
            && s.bg_span.is_none()
            && s.pages.iter().all(|p| p.bg.is_none())
            && s.kind != SpreadKind::Cover
    };
    for id in held {
        if let Some(at) = book.spreads.iter().position(|s| s.id == id) {
            if book.spreads.len() > 1 && empty(&book.spreads[at]) {
                book.spreads.remove(at);
            }
        }
    }
    renumber(book, flow);
}

/// Whether a colour is a hex colour or refers to a swatch the book has.
fn swatch_is_there(book: &Book, colour: &Colour) -> bool {
    colour
        .swatch_id()
        .is_none_or(|id| book.styles.swatches.iter().any(|s| s.id.0 == id))
}

/// The most swatches a book can keep.
const MAX_SWATCHES: usize = 200;

/// Refuses a swatch with no usable id or name, a display colour that is not hex, or ink that is not
/// percents; and a new one when the book has too many.
fn check_swatch(book: &Book, swatch: &folio_model::BookSwatch) -> Result<(), SessionError> {
    let bad = |what: &str| Err(SessionError::BadSwatch(what.to_owned()));
    let id_ok = !swatch.id.0.is_empty()
        && swatch.id.0.len() <= 64
        && swatch
            .id
            .0
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-');
    if !id_ok {
        return bad("a swatch id is lower case letters, digits and dashes");
    }
    if swatch.name.trim().is_empty() || swatch.name.chars().count() > 64 {
        return bad("a swatch needs a name of at most 64 characters");
    }
    if swatch.display.rgb().is_none() {
        return bad("a swatch shows as a #rrggbb colour");
    }
    if swatch.ink.is_some_and(|ink| !ink.is_valid()) {
        return bad("ink is a percent of each plate, 0 to 100");
    }
    let new = !book.styles.swatches.iter().any(|s| s.id == swatch.id);
    if new && book.styles.swatches.len() >= MAX_SWATCHES {
        return bad("a book keeps at most 200 swatches");
    }
    Ok(())
}
