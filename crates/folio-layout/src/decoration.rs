// Decoration: the frames that add nothing for a reader who cannot see the page
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! A tagged PDF marks decoration as an artifact so a screen reader skips it, and preflight does not
//! ask for a description of it. Which frames those are is decided here, once, from the model alone:
//!
//! - a frame the person marked `decorative` (any kind of frame);
//! - a **page number**: a small text frame whose whole text is a number (digits, roman numerals or
//!   the `{page}` token) in the bottom band of its page, at a position repeated on other pages;
//! - a **page-number backdrop**: a small picture with no description under such a frame (a pumpkin
//!   behind a number), because a picture the person described is content whatever it sits under.
//!
//! An empty description never means decorative: that is what the alt-text check is there to find.

use std::collections::HashMap;

use folio_model::{Book, Frame, FrameId, InlineContent, Rect, Spread, SpreadId, SpreadKind};

use crate::preflight::page_rects;

/// The bottom part of a page, as a share of its height, that a page number sits in.
const BOTTOM_BAND: f64 = 0.15;
/// The most of a page's width and height a page number's frame takes.
const NUMBER_MAX_WIDTH: f64 = 0.4;
const NUMBER_MAX_HEIGHT: f64 = 0.12;
/// The longest side a backdrop may have, as a share of the page's shorter side.
const BACKDROP_MAX_SIDE: f64 = 0.2;
/// How far apart, in points, two numbers' boxes may be and still be the same place on a page.
const SAME_PLACE_PT: f64 = 2.0;

/// Why a frame is decoration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Decoration {
    /// The person marked it.
    Marked,
    /// A number alone in the page-number position.
    PageNumber,
    /// A small picture with no description behind a page number.
    PageNumberBackdrop,
}

/// The decoration of a whole book, found once.
#[derive(Debug, Clone, Default)]
pub struct Decorations {
    found: HashMap<(SpreadId, FrameId), Decoration>,
}

impl Decorations {
    /// The decoration in `book`'s spreads. Masters are not looked at: everything on a master is
    /// furniture already.
    pub fn of_book(book: &Book) -> Self {
        let mut found = HashMap::new();
        let mut numbers: Vec<Number> = Vec::new();
        // Every spread, with the frames it holds in stacking order.
        for spread in &book.spreads {
            if spread.kind == SpreadKind::Cover {
                // The cover is the wrap, not a page with a number.
                for frame in folio_model::leaves(&spread.frames) {
                    if frame.base().decorative {
                        found.insert(
                            (spread.id.clone(), frame.base().id.clone()),
                            Decoration::Marked,
                        );
                    }
                }
                continue;
            }
            let pages = page_rects(book, spread);
            for (at, frame) in folio_model::leaves(&spread.frames).into_iter().enumerate() {
                let base = frame.base();
                if base.decorative {
                    found.insert((spread.id.clone(), base.id.clone()), Decoration::Marked);
                }
                if let Frame::Text(text) = frame {
                    if text.base.from_master.is_some() {
                        continue;
                    }
                    let is_number = text
                        .content
                        .as_ref()
                        .is_some_and(|c| is_page_number_text(&plain(c)));
                    if !is_number {
                        continue;
                    }
                    let bounds = base.rect.turned_bounds(base.rotation);
                    let Some((index, page)) = page_of(&pages, &bounds) else {
                        continue;
                    };
                    let (px, py, pw, ph) =
                        (page.rect.x.0, page.rect.y.0, page.rect.w.0, page.rect.h.0);
                    let centre_y = bounds.y.0 + bounds.h.0 / 2.0;
                    let small =
                        bounds.w.0 <= pw * NUMBER_MAX_WIDTH && bounds.h.0 <= ph * NUMBER_MAX_HEIGHT;
                    let low = centre_y >= py + ph * (1.0 - BOTTOM_BAND);
                    if small && low {
                        numbers.push(Number {
                            spread: spread.id.clone(),
                            frame: base.id.clone(),
                            at,
                            page: index,
                            recto: page.recto,
                            rel: [bounds.x.0 - px, bounds.y.0 - py, bounds.w.0, bounds.h.0],
                        });
                    }
                }
            }
        }
        // A number counts only where the same place holds a number on another page too.
        let places: Vec<bool> = numbers
            .iter()
            .map(|n| {
                numbers.iter().any(|o| {
                    (o.spread != n.spread || o.page != n.page)
                        && o.recto == n.recto
                        && o.rel
                            .iter()
                            .zip(n.rel.iter())
                            .all(|(a, b)| (a - b).abs() <= SAME_PLACE_PT)
                })
            })
            .collect();
        let mut page_numbers: Vec<&Number> = Vec::new();
        for (n, repeated) in numbers.iter().zip(places) {
            let key = (n.spread.clone(), n.frame.clone());
            if repeated {
                found.entry(key).or_insert(Decoration::PageNumber);
                page_numbers.push(n);
            } else if found.contains_key(&key) {
                // Marked by the person, so its backdrop is one too.
                page_numbers.push(n);
            }
        }
        // A number the person marked is a page number for its backdrop even when it is not
        // repeated; those were pushed above only when they are in the band, which is the case
        // that matters.
        for spread in &book.spreads {
            if spread.kind == SpreadKind::Cover {
                continue;
            }
            let pages = page_rects(book, spread);
            let here: Vec<&&Number> = page_numbers
                .iter()
                .filter(|n| n.spread == spread.id)
                .collect();
            if here.is_empty() {
                continue;
            }
            for (at, frame) in folio_model::leaves(&spread.frames).into_iter().enumerate() {
                let Frame::Image(image) = frame else { continue };
                let base = &image.base;
                if base.from_master.is_some()
                    || base
                        .description
                        .as_deref()
                        .is_some_and(|d| !d.trim().is_empty())
                {
                    continue;
                }
                let bounds = base.rect.turned_bounds(base.rotation);
                let Some((_, page)) = page_of(&pages, &bounds) else {
                    continue;
                };
                let shorter = page.rect.w.0.min(page.rect.h.0);
                if bounds.w.0.max(bounds.h.0) > shorter * BACKDROP_MAX_SIDE {
                    continue;
                }
                let under = here.iter().any(|n| {
                    n.at > at && {
                        // The number's centre lies inside the picture.
                        let cx = page.rect.x.0 + n.rel[0] + n.rel[2] / 2.0;
                        let cy = page.rect.y.0 + n.rel[1] + n.rel[3] / 2.0;
                        cx >= bounds.x.0
                            && cx <= bounds.x.0 + bounds.w.0
                            && cy >= bounds.y.0
                            && cy <= bounds.y.0 + bounds.h.0
                    }
                });
                if under {
                    found
                        .entry((spread.id.clone(), base.id.clone()))
                        .or_insert(Decoration::PageNumberBackdrop);
                }
            }
        }
        Decorations { found }
    }

    /// Why a frame is decoration, or `None` when it is content.
    pub fn of(&self, spread: &SpreadId, frame: &FrameId) -> Option<Decoration> {
        self.found.get(&(spread.clone(), frame.clone())).copied()
    }

    /// Whether the frame is decoration.
    pub fn is_decoration(&self, spread: &Spread, frame: &FrameId) -> bool {
        self.of(&spread.id, frame).is_some()
    }

    pub fn is_empty(&self) -> bool {
        self.found.is_empty()
    }
}

struct Number {
    spread: SpreadId,
    frame: FrameId,
    /// Its place in the spread's stacking order.
    at: usize,
    page: usize,
    recto: bool,
    /// Its turned box in the page's own space: x, y, width and height.
    rel: [f64; 4],
}

/// The page of a spread that holds the middle of `bounds`, and its number.
fn page_of<'a>(
    pages: &'a [crate::preflight::PageRect],
    bounds: &Rect,
) -> Option<(usize, &'a crate::preflight::PageRect)> {
    let cx = bounds.x.0 + bounds.w.0 / 2.0;
    pages
        .iter()
        .enumerate()
        .find(|(_, p)| cx >= p.rect.x.0 && cx < p.rect.x.0 + p.rect.w.0)
}

fn plain(content: &InlineContent) -> String {
    content.inline.iter().map(|r| r.text.as_str()).collect()
}

/// Whether `text` is a page number and nothing else: digits, roman numerals or the `{page}` token,
/// with the dashes, dots or brackets that sometimes surround one.
pub fn is_page_number_text(text: &str) -> bool {
    const AROUND: &str = "-\u{2013}\u{2014}\u{2212}\u{00B7}\u{2022}.,:;()[]~*|";
    let core = text.trim_matches(|c: char| c.is_whitespace() || AROUND.contains(c));
    if core.is_empty() || core.contains(char::is_whitespace) {
        return false;
    }
    if core == "{page}" {
        return true;
    }
    if core.len() <= 4 && core.chars().all(|c| c.is_ascii_digit()) {
        return true;
    }
    roman_value(core).is_some()
}

/// The value of a roman numeral in one case, if it is written the usual way.
fn roman_value(text: &str) -> Option<u32> {
    let upper = text.chars().all(|c| "IVXLCDM".contains(c));
    let lower = text.chars().all(|c| "ivxlcdm".contains(c));
    if text.len() > 8 || !(upper || lower) {
        return None;
    }
    let digit = |c: char| match c.to_ascii_uppercase() {
        'I' => 1,
        'V' => 5,
        'X' => 10,
        'L' => 50,
        'C' => 100,
        'D' => 500,
        _ => 1000,
    };
    let values: Vec<u32> = text.chars().map(digit).collect();
    let mut total = 0;
    for (i, v) in values.iter().enumerate() {
        if values.get(i + 1).is_some_and(|next| next > v) {
            total -= *v as i64;
        } else {
            total += *v as i64;
        }
    }
    let total = u32::try_from(total)
        .ok()
        .filter(|t| (1..4000).contains(t))?;
    // Only the usual spelling counts: "iiii" and "vx" are not numbers.
    (roman_of(total).eq_ignore_ascii_case(text)).then_some(total)
}

fn roman_of(mut value: u32) -> String {
    const TABLE: [(u32, &str); 13] = [
        (1000, "M"),
        (900, "CM"),
        (500, "D"),
        (400, "CD"),
        (100, "C"),
        (90, "XC"),
        (50, "L"),
        (40, "XL"),
        (10, "X"),
        (9, "IX"),
        (5, "V"),
        (4, "IV"),
        (1, "I"),
    ];
    let mut out = String::new();
    for (n, s) in TABLE {
        while value >= n {
            out.push_str(s);
            value -= n;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbers_are_digits_roman_numerals_or_the_page_token() {
        for yes in [
            "7",
            "12",
            " 12 ",
            "- 12 -",
            "\u{2013} 3 \u{2013}",
            "iv",
            "XIV",
            "{page}",
            "(9)",
        ] {
            assert!(is_page_number_text(yes), "{yes:?}");
        }
        for no in [
            "",
            "page 3",
            "12345",
            "iiii",
            "vx",
            "Mixed",
            "3 little pigs",
            "one",
            "7\n8",
            "-",
        ] {
            assert!(!is_page_number_text(no), "{no:?}");
        }
    }
}
