// Find and replace: where words are in a book, and the edits that change them
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! A search is a pure function of the book. It reads every place words live — the story of each
//! flow, a text frame's own words, a shape's, each table cell's, and the frames of the master
//! pages — and answers with where the query is, as a [`TextTarget`] and a paragraph and code point
//! range (D-53), so a match is something an `EditText` can be sent for (D-62).

use crate::wire::{FindMatch, FindReport, FindRequest};
use folio_model::{Book, FlowId, Frame, FrameBase, StoryPara, TextTarget};
use regex::{Regex, RegexBuilder};
use std::cmp::Ordering;

/// The longest query accepted, in characters.
const MAX_QUERY: usize = 1_000;

/// The most memory a compiled expression may take. The `regex` engine runs in time linear in the
/// words searched, so the only way to make a pattern costly is to make it large, and this refuses
/// that (`(a{1000}){1000}` is refused, `(a+)+$` is cheap).
const SIZE_LIMIT: usize = 1 << 20;

/// The most matches one search reports. A search that reaches it says it was cut short.
pub const MAX_MATCHES: usize = 10_000;

/// How many characters of the paragraph are shown on each side of a match as its context.
const CONTEXT: usize = 24;

/// Why a search could not be made.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BadSearch(pub String);

impl std::fmt::Display for BadSearch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for BadSearch {}

/// Where a story's words are on the pages, so a match in it is ordered with the rest. `None` falls
/// back to where the story's first frame is.
pub type StorySlots<'a> = &'a dyn Fn(&FlowId, u32, u32) -> Option<Slot>;

/// A place in the book's reading order: the spread (masters come after every spread), the page of
/// it, and the top-left of the frame.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Slot {
    pub spread: usize,
    pub page: usize,
    pub y: f64,
    pub x: f64,
}

impl Slot {
    fn order(&self, other: &Slot) -> Ordering {
        self.spread
            .cmp(&other.spread)
            .then(self.page.cmp(&other.page))
            .then(self.y.total_cmp(&other.y))
            .then(self.x.total_cmp(&other.x))
    }
}

/// The words of a book that can be searched, one paragraph at a time.
struct Place {
    target: TextTarget,
    slot: Slot,
    spread: Option<String>,
    master: Option<String>,
    locked: bool,
    paragraphs: Vec<String>,
}

/// A query ready to run.
struct Matcher {
    regex: Regex,
    whole_word: bool,
    /// Whether `$1` in a replacement means a capture group.
    expands: bool,
}

/// One match in a paragraph: bytes, and the replacement when it was asked for.
struct Hit {
    from: usize,
    to: usize,
    replacement: Option<String>,
}

fn is_word(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

fn is_apostrophe(c: char) -> bool {
    c == '\'' || c == '\u{2019}'
}

/// Whether `c`, with `neighbour` on its far side, is part of a word: an apostrophe between two
/// letters is (so "don" is not a whole word of "don't").
fn in_word(c: char, neighbour: Option<char>) -> bool {
    is_word(c) || (is_apostrophe(c) && neighbour.is_some_and(is_word))
}

impl Matcher {
    fn new(request: &FindRequest) -> Result<Self, BadSearch> {
        if request.query.chars().count() > MAX_QUERY {
            return Err(BadSearch(format!(
                "a search can be at most {MAX_QUERY} characters"
            )));
        }
        let pattern = if request.regex {
            request.query.clone()
        } else {
            regex::escape(&request.query)
        };
        let regex = RegexBuilder::new(&pattern)
            .case_insensitive(!request.match_case)
            .size_limit(SIZE_LIMIT)
            .dfa_size_limit(SIZE_LIMIT)
            .nest_limit(64)
            .build()
            .map_err(|e| {
                BadSearch(match e {
                    regex::Error::CompiledTooBig(_) => "that pattern is too large".to_owned(),
                    other => format!("that is not a pattern Folio can search for: {other}"),
                })
            })?;
        Ok(Self {
            regex,
            whole_word: request.whole_word,
            expands: request.regex,
        })
    }

    /// Whether the match `from..to` of `text` starts and ends at the edge of a word.
    fn whole(&self, text: &str, from: usize, to: usize) -> bool {
        let inside = &text[from..to];
        let (Some(first), Some(last)) = (inside.chars().next(), inside.chars().next_back()) else {
            return true;
        };
        let mut before = text[..from].chars().rev();
        let mut after = text[to..].chars();
        let starts = !is_word(first) || {
            let prev = before.next();
            !prev.is_some_and(|c| in_word(c, before.next()))
        };
        let ends = !is_word(last) || {
            let next = after.next();
            !next.is_some_and(|c| in_word(c, after.next()))
        };
        starts && ends
    }

    /// Every match in `text`, left to right and not overlapping. An empty match is no match. With
    /// `replacement`, each says what it is to be replaced with.
    fn scan(&self, text: &str, replacement: Option<&str>) -> Vec<Hit> {
        let mut out = Vec::new();
        let mut at = 0;
        while at <= text.len() {
            let found = if let (true, Some(with)) = (self.expands, replacement) {
                self.regex.captures_at(text, at).map(|caps| {
                    let whole = caps.get(0).expect("group 0 is the match");
                    let mut expanded = String::new();
                    caps.expand(with, &mut expanded);
                    (whole.start(), whole.end(), Some(expanded))
                })
            } else {
                self.regex
                    .find_at(text, at)
                    .map(|m| (m.start(), m.end(), replacement.map(str::to_owned)))
            };
            let Some((from, to, replacement)) = found else {
                break;
            };
            let step = |from: usize| {
                from + text[from..]
                    .chars()
                    .next()
                    .map_or(1, |c| c.len_utf8().max(1))
            };
            if from == to {
                at = step(to);
                continue;
            }
            if self.whole_word && !self.whole(text, from, to) {
                // Not at the edge of a word, so look again from the next character: a later match
                // may start inside this one.
                at = step(from);
                continue;
            }
            out.push(Hit {
                from,
                to,
                replacement,
            });
            at = to;
        }
        out
    }
}

pub fn slot_of(spread: usize, base: &FrameBase, pages: usize, page_width: f64) -> Slot {
    let rect = base.rect;
    let page = if pages > 1 && page_width > 0.0 {
        (((rect.x.0 + rect.w.0 / 2.0) / page_width).floor().max(0.0) as usize).min(pages - 1)
    } else {
        0
    };
    Slot {
        spread,
        page,
        y: rect.y.0,
        x: rect.x.0,
    }
}

fn paragraphs_text(paragraphs: &[StoryPara]) -> Vec<String> {
    paragraphs
        .iter()
        .map(|p| folio_model::text_of(&p.inline))
        .collect()
}

#[derive(Clone, Copy)]
struct Walk<'a> {
    spread: usize,
    spread_id: Option<&'a str>,
    master: Option<&'a str>,
    pages: usize,
    page_width: f64,
    parent: Option<&'a FrameBase>,
}

fn walk_frames(frames: &[Frame], at: &Walk, out: &mut Vec<Place>) {
    for frame in frames {
        let base = frame.base();
        let slot = slot_of(
            at.spread,
            at.parent.unwrap_or(base),
            at.pages,
            at.page_width,
        );
        let locked = base.locked || base.from_master.is_some();
        let mut place = |target: TextTarget, paragraphs: Vec<StoryPara>| {
            out.push(Place {
                target,
                slot,
                spread: at.spread_id.map(str::to_owned),
                master: at.master.map(str::to_owned),
                locked,
                paragraphs: paragraphs_text(&paragraphs),
            });
        };
        match frame {
            Frame::Text(t) if t.flow.is_none() => {
                if let Some(content) = &t.content {
                    place(
                        TextTarget::Frame {
                            frame: base.id.clone(),
                        },
                        folio_model::paragraphs_of(Some(content)),
                    );
                }
            }
            Frame::Shape(s) => {
                if let Some(content) = &s.content {
                    place(
                        TextTarget::Shape {
                            frame: base.id.clone(),
                        },
                        folio_model::paragraphs_of(Some(content)),
                    );
                }
            }
            Frame::Table(t) => {
                for (row, cells) in t.rows.iter().enumerate() {
                    for (col, cell) in cells.iter().enumerate() {
                        if let Some(content) = &cell.content {
                            place(
                                TextTarget::Cell {
                                    frame: base.id.clone(),
                                    row: row as u32,
                                    col: col as u32,
                                },
                                folio_model::paragraphs_of(Some(content)),
                            );
                        }
                    }
                }
            }
            Frame::Group { children, .. } => {
                walk_frames(
                    children,
                    &Walk {
                        parent: Some(at.parent.unwrap_or(base)),
                        ..*at
                    },
                    out,
                );
            }
            _ => {}
        }
    }
}

/// The first frame of a flow on a spread: where its story is when the pages have not been laid out.
fn first_slot(book: &Book, flow: &FlowId) -> Slot {
    let frames = book
        .flows
        .iter()
        .find(|f| &f.id == flow)
        .map(|f| f.frames.as_slice())
        .unwrap_or(&[]);
    for (index, spread) in book.spreads.iter().enumerate() {
        for frame in &spread.frames {
            if frames.contains(&frame.base().id) {
                return slot_of(index, frame.base(), spread.pages.len(), book.trim.w.0);
            }
        }
    }
    Slot {
        spread: book.spreads.len(),
        page: 0,
        y: 0.0,
        x: 0.0,
    }
}

fn places(book: &Book) -> Vec<Place> {
    let mut out = Vec::new();
    let width = book.trim.w.0;
    for (index, spread) in book.spreads.iter().enumerate() {
        let mut found = Vec::new();
        walk_frames(
            &spread.frames,
            &Walk {
                spread: index,
                spread_id: Some(&spread.id.0),
                master: None,
                pages: spread.pages.len(),
                page_width: width,
                parent: None,
            },
            &mut found,
        );
        out.extend(found);
    }
    for (index, master) in book.master_pages.iter().enumerate() {
        let mut found = Vec::new();
        walk_frames(
            &master.frames,
            &Walk {
                spread: book.spreads.len() + index,
                spread_id: None,
                master: Some(&master.id.0),
                pages: 1,
                page_width: width,
                parent: None,
            },
            &mut found,
        );
        out.extend(found);
    }
    out
}

/// One match, before it is ordered.
struct Found {
    matched: FindMatch,
    slot: Slot,
}

fn context(text: &str, from: usize, to: usize) -> String {
    let chars: Vec<char> = text.chars().collect();
    let lo = from.saturating_sub(CONTEXT);
    let hi = (to + CONTEXT).min(chars.len());
    let mut out = String::new();
    if lo > 0 {
        out.push('…');
    }
    out.extend(&chars[lo..hi]);
    if hi < chars.len() {
        out.push('…');
    }
    out
}

/// Every match of `request` in the book, in reading order: by spread, then page, then the frame's
/// place on the page (top to bottom, left to right), then the words' place in the frame. Masters
/// come last. A story's matches are put where `story_slots` says the words are on the pages.
pub fn find(
    book: &Book,
    request: &FindRequest,
    story_slots: StorySlots,
) -> Result<FindReport, BadSearch> {
    let hits = search(book, request, story_slots, None)?;
    let truncated = hits.len() > MAX_MATCHES;
    Ok(FindReport {
        matches: hits
            .into_iter()
            .take(MAX_MATCHES)
            .map(|(found, _)| found.matched)
            .collect(),
        truncated,
    })
}

/// The matches with the replacement each would take, in reading order.
fn search(
    book: &Book,
    request: &FindRequest,
    story_slots: StorySlots,
    replacement: Option<&str>,
) -> Result<Vec<(Found, Option<String>)>, BadSearch> {
    if request.query.is_empty() {
        return Ok(Vec::new());
    }
    let matcher = Matcher::new(request)?;
    let mut out: Vec<(Found, Option<String>)> = Vec::new();
    let scan = |out: &mut Vec<(Found, Option<String>)>,
                target: &TextTarget,
                slot_for: &dyn Fn(u32, u32) -> Slot,
                (spread, master, locked): (&Option<String>, &Option<String>, bool),
                paragraphs: &[String]| {
        for (para, text) in paragraphs.iter().enumerate() {
            let mut counted = (0usize, 0usize);
            for hit in matcher.scan(text, replacement) {
                // Bytes to code points, counting on from the last match.
                let from = counted.1 + text[counted.0..hit.from].chars().count();
                let to = from + text[hit.from..hit.to].chars().count();
                counted = (hit.to, to);
                out.push((
                    Found {
                        slot: slot_for(para as u32, from as u32),
                        matched: FindMatch {
                            target: target.clone(),
                            para: para as u32,
                            from: from as u32,
                            to: to as u32,
                            spread: spread.clone(),
                            master: master.clone(),
                            context: context(text, from, to),
                            locked,
                        },
                    },
                    hit.replacement,
                ));
            }
        }
    };
    for place in places(book) {
        let slot = place.slot;
        scan(
            &mut out,
            &place.target,
            &|_, _| slot,
            (&place.spread, &place.master, place.locked),
            &place.paragraphs,
        );
    }
    for flow in &book.flows {
        let texts = paragraphs_text(&flow.paragraphs);
        let home = first_slot(book, &flow.id);
        // A story is as locked as the frames it runs through are, all of them.
        let locked = !flow.frames.is_empty()
            && flow.frames.iter().all(|id| {
                book.spreads
                    .iter()
                    .flat_map(|s| s.frames.iter())
                    .find(|f| &f.base().id == id)
                    .is_some_and(|f| f.base().locked)
            });
        let slot_for = |para: u32, offset: u32| story_slots(&flow.id, para, offset).unwrap_or(home);
        let before = out.len();
        scan(
            &mut out,
            &TextTarget::Story {
                flow: flow.id.clone(),
            },
            &slot_for,
            (&None, &None, locked),
            &texts,
        );
        for (found, _) in &mut out[before..] {
            found.matched.spread = book.spreads.get(found.slot.spread).map(|s| s.id.0.clone());
        }
    }
    // Stable, so matches of one frame stay in the order they were found in (by paragraph, then
    // position); a story's frames are told apart by the slot of the line each match is on.
    out.sort_by(|(a, _), (b, _)| a.slot.order(&b.slot));
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Value};

    fn text_frame(id: &str, x: f64, y: f64, words: &str) -> Value {
        json!({
            "type": "text", "id": id, "box": { "x": x, "y": y, "w": 100, "h": 50 },
            "style": "body", "content": { "inline": [{ "text": words }] }
        })
    }

    fn book(spreads: Vec<Value>, flows: Vec<Value>, masters: Vec<Value>) -> Book {
        serde_json::from_value(json!({
            "schema": 1, "id": "b", "title": "T", "authors": [], "language": "en", "kind": "picture",
            "layout": "fixed", "trim": { "w": 400, "h": 600 }, "bleed": 0,
            "margins": { "top": 0, "bottom": 0, "inside": 0, "outside": 0 },
            "grid": { "columns": 1, "gutter": 0, "baseline": null },
            "spreads": spreads, "flows": flows, "masterPages": masters,
            "metadata": { "isbn": { "paperback": null, "hardcover": null, "ebook": null }, "description": "" }
        }))
        .unwrap()
    }

    fn spread(id: &str, pages: usize, frames: Vec<Value>) -> Value {
        let pages: Vec<Value> = (0..pages)
            .map(|i| json!({ "id": format!("{id}p{i}"), "master": null }))
            .collect();
        json!({ "id": id, "kind": "spread", "pages": pages, "frames": frames })
    }

    fn ask(query: &str) -> FindRequest {
        FindRequest {
            query: query.into(),
            match_case: false,
            whole_word: false,
            regex: false,
        }
    }

    fn none() -> impl Fn(&FlowId, u32, u32) -> Option<Slot> {
        |_, _, _| None
    }

    fn found(book: &Book, request: &FindRequest) -> Vec<FindMatch> {
        find(book, request, &none()).unwrap().matches
    }

    fn one_frame(words: &str) -> Book {
        book(
            vec![spread("s", 1, vec![text_frame("t", 0.0, 0.0, words)])],
            vec![],
            vec![],
        )
    }

    fn spans(matches: &[FindMatch]) -> Vec<(u32, u32, u32)> {
        matches.iter().map(|m| (m.para, m.from, m.to)).collect()
    }

    #[test]
    fn a_match_is_a_range_of_code_points_of_a_paragraph() {
        let b = one_frame("one cat\ntwo cats");
        let m = found(&b, &ask("cat"));
        assert_eq!(spans(&m), vec![(0, 4, 7), (1, 4, 7)]);
        assert_eq!(m[0].spread.as_deref(), Some("s"));
        assert_eq!(
            m[0].target,
            TextTarget::Frame {
                frame: folio_model::FrameId("t".into())
            }
        );
        assert_eq!(m[1].context, "two cats");
    }

    #[test]
    fn offsets_count_code_points_not_bytes_or_utf16_units() {
        // A combining acute (two code points), a letter outside the BMP (one code point, two UTF-16
        // units) and a multi-byte letter come before the match.
        let b = one_frame("e\u{301}😀é cat");
        let m = found(&b, &ask("cat"));
        assert_eq!(spans(&m), vec![(0, 5, 8)]);
        // A match of a combining mark alone is found as a mark.
        let m = found(&b, &ask("\u{301}"));
        assert_eq!(spans(&m), vec![(0, 1, 2)]);
        let astral = found(&b, &ask("😀"));
        assert_eq!(spans(&astral), vec![(0, 2, 3)]);
    }

    #[test]
    fn case_matters_only_when_asked() {
        let b = one_frame("Cat cat CAT");
        assert_eq!(found(&b, &ask("cat")).len(), 3);
        let mut exact = ask("cat");
        exact.match_case = true;
        assert_eq!(spans(&found(&b, &exact)), vec![(0, 4, 7)]);
        // Folding is Unicode's, not only ASCII's.
        let accents = one_frame("ÉCOLE école");
        assert_eq!(found(&accents, &ask("école")).len(), 2);
    }

    #[test]
    fn a_whole_word_stops_at_the_edge_of_a_word_and_not_at_an_apostrophe() {
        let b = one_frame("cat concat cats cat's cat. (cat) cat_x cat");
        let mut whole = ask("cat");
        whole.whole_word = true;
        assert_eq!(
            spans(&found(&b, &whole)),
            vec![(0, 0, 3), (0, 22, 25), (0, 28, 31), (0, 39, 42)]
        );
        // A query that starts or ends with punctuation needs no word edge there.
        let mut dotted = ask("cat.");
        dotted.whole_word = true;
        assert_eq!(spans(&found(&b, &dotted)), vec![(0, 22, 26)]);
        // Found again from inside a rejected stretch.
        let again = one_frame("aaa aa");
        let mut aa = ask("aa");
        aa.whole_word = true;
        assert_eq!(spans(&found(&again, &aa)), vec![(0, 4, 6)]);
    }

    #[test]
    fn matches_do_not_overlap_and_an_empty_match_is_no_match() {
        let b = one_frame("aaaaa");
        assert_eq!(spans(&found(&b, &ask("aa"))), vec![(0, 0, 2), (0, 2, 4)]);
        assert!(found(&b, &ask("")).is_empty());
        let mut star = ask("x*");
        star.regex = true;
        assert!(found(&b, &star).is_empty());
        let mut some = ask("a*");
        some.regex = true;
        assert_eq!(spans(&found(&b, &some)), vec![(0, 0, 5)]);
        let mixed = one_frame("xaax");
        assert_eq!(spans(&found(&mixed, &some)), vec![(0, 1, 3)]);
    }

    #[test]
    fn a_regular_expression_is_only_read_as_one_when_asked() {
        let b = one_frame("a.c abc a1c");
        assert_eq!(spans(&found(&b, &ask("a.c"))), vec![(0, 0, 3)]);
        let mut pattern = ask("a.c");
        pattern.regex = true;
        assert_eq!(found(&b, &pattern).len(), 3);
        let mut digits = ask(r"\d");
        digits.regex = true;
        assert_eq!(spans(&found(&b, &digits)), vec![(0, 9, 10)]);
    }

    #[test]
    fn a_pattern_that_does_not_parse_or_is_too_large_is_refused() {
        let b = one_frame("x");
        let mut bad = ask("(unclosed");
        bad.regex = true;
        assert!(find(&b, &bad, &none()).is_err());
        let mut big = ask("((a{1000}){1000}){1000}");
        big.regex = true;
        assert!(find(&b, &big, &none()).is_err());
        let long = ask(&"x".repeat(MAX_QUERY + 1));
        assert!(find(&b, &long, &none()).is_err());
    }

    #[test]
    fn a_pattern_that_backtracks_in_other_engines_is_quick_here() {
        let b = one_frame(&format!("{}!", "a".repeat(5_000)));
        let mut evil = ask("(a+)+$");
        evil.regex = true;
        let started = std::time::Instant::now();
        let m = found(&b, &evil);
        assert!(m.is_empty());
        assert!(started.elapsed() < std::time::Duration::from_secs(5));
    }

    #[test]
    fn every_kind_of_words_is_searched() {
        let shape = json!({
            "type": "shape", "id": "sh", "box": { "x": 0, "y": 100, "w": 50, "h": 50 },
            "shape": "speech", "fill": null, "style": null,
            "content": { "inline": [{ "text": "shape word" }] }
        });
        let table = json!({
            "type": "table", "id": "tb", "box": { "x": 0, "y": 200, "w": 100, "h": 50 },
            "rows": [[{ "content": { "inline": [{ "text": "no" }] } },
                      { "content": { "inline": [{ "text": "a word\nsecond word" }] } }]],
            "cols": [1, 1], "header": false, "look": "lined"
        });
        let group = json!({
            "type": "group", "id": "g", "box": { "x": 0, "y": 300, "w": 50, "h": 50 },
            "children": [text_frame("gt", 0.0, 0.0, "grouped word")]
        });
        let story_frame = json!({
            "type": "text", "id": "sf", "box": { "x": 0, "y": 400, "w": 100, "h": 50 },
            "style": "body", "flow": { "id": "f", "index": 0 }
        });
        let story = json!({
            "id": "f", "frames": ["sf"],
            "paragraphs": [{ "inline": [{ "text": "story word" }] }]
        });
        let b = book(
            vec![spread(
                "s",
                1,
                vec![
                    text_frame("t", 0.0, 0.0, "plain word"),
                    shape,
                    table,
                    group,
                    story_frame,
                ],
            )],
            vec![story],
            vec![],
        );
        let m = found(&b, &ask("word"));
        let targets: Vec<&TextTarget> = m.iter().map(|m| &m.target).collect();
        assert_eq!(m.len(), 6, "{targets:?}");
        assert!(matches!(targets[0], TextTarget::Frame { .. }));
        assert!(matches!(targets[1], TextTarget::Shape { .. }));
        assert!(matches!(
            targets[2],
            TextTarget::Cell { row: 0, col: 1, .. }
        ));
        assert_eq!((m[2].para, m[3].para), (0, 1));
        assert!(matches!(targets[4], TextTarget::Frame { frame } if frame.0 == "gt"));
        assert!(matches!(targets[5], TextTarget::Story { .. }));
        assert_eq!(m[5].spread.as_deref(), Some("s"));
    }

    #[test]
    fn matches_are_in_reading_order_by_spread_then_page_then_top_to_bottom() {
        let b = book(
            vec![
                spread(
                    "one",
                    2,
                    vec![
                        text_frame("right", 450.0, 10.0, "x right"),
                        text_frame("low", 10.0, 300.0, "x low"),
                        text_frame("high", 10.0, 10.0, "x high"),
                        text_frame("highright", 200.0, 10.0, "x highright"),
                    ],
                ),
                spread("two", 1, vec![text_frame("later", 0.0, 0.0, "x later")]),
            ],
            vec![],
            vec![],
        );
        let order: Vec<String> = found(&b, &ask("x"))
            .iter()
            .map(|m| match &m.target {
                TextTarget::Frame { frame } => frame.0.clone(),
                other => panic!("{other:?}"),
            })
            .collect();
        assert_eq!(order, ["high", "highright", "low", "right", "later"]);
    }

    #[test]
    fn a_story_is_put_where_its_words_are_laid_out() {
        let frame = |id: &str, index: u32, y: f64| {
            json!({
                "type": "text", "id": id, "box": { "x": 0, "y": y, "w": 100, "h": 50 },
                "style": "body", "flow": { "id": "f", "index": index }
            })
        };
        let b = book(
            vec![
                spread("one", 1, vec![frame("a", 0, 0.0)]),
                spread(
                    "two",
                    1,
                    vec![frame("b", 1, 0.0), text_frame("free", 0.0, 100.0, "needle")],
                ),
            ],
            vec![json!({
                "id": "f", "frames": ["a", "b"],
                "paragraphs": [{ "inline": [{ "text": "needle" }] }, { "inline": [{ "text": "needle" }] }]
            })],
            vec![],
        );
        // Without a layout a story is at its first frame.
        let m = found(&b, &ask("needle"));
        assert_eq!(m[0].spread.as_deref(), Some("one"));
        // With one, the second paragraph is on the second spread, ahead of the free frame there.
        let place = |_: &FlowId, para: u32, _: u32| {
            Some(Slot {
                spread: para as usize,
                page: 0,
                y: 0.0,
                x: 0.0,
            })
        };
        let m = find(&b, &ask("needle"), &place).unwrap().matches;
        let seen: Vec<(Option<&str>, u32)> =
            m.iter().map(|m| (m.spread.as_deref(), m.para)).collect();
        assert_eq!(
            seen,
            vec![(Some("one"), 0), (Some("two"), 1), (Some("two"), 0)]
        );
        assert!(matches!(m[2].target, TextTarget::Frame { .. }));
    }

    #[test]
    fn words_on_a_master_page_come_last_and_belong_to_no_spread() {
        let master = json!({
            "id": "m", "name": "A",
            "frames": [text_frame("mf", 0.0, 0.0, "needle on the master")]
        });
        let b = book(
            vec![spread("s", 1, vec![text_frame("t", 0.0, 0.0, "needle")])],
            vec![],
            vec![master],
        );
        let m = found(&b, &ask("needle"));
        assert_eq!(m.len(), 2);
        assert_eq!(m[0].master, None);
        assert_eq!(m[1].master.as_deref(), Some("m"));
        assert_eq!(m[1].spread, None);
    }
}
