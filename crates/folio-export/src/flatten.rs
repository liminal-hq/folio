// Flattening: what is behind a see-through picture, and whether it is plain enough to blend with
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! A picture with see-through parts (a PNG with feathered edges) is blended onto what is behind it
//! when a file is flattened, so the file holds no transparent picture. That is only right when
//! what is behind it is known exactly: the page's own flat colour, or the bare paper. Anything
//! else underneath (another picture, a shape, a table, words, a gradient, a texture) is not
//! guessed at, and the picture stays see-through there; the export says so.

use std::collections::{BTreeMap, BTreeSet};

use folio_layout::scene::{PageScene, SceneKind, SceneRect};
use folio_model::{Background, BookSwatch, Swatches};

/// A colour in sRGB.
pub type Rgb = [u8; 3];

/// What the file shows where nothing is drawn: a PDF page has no paper of its own, so a viewer or
/// a press shows white.
pub const PAPER: Rgb = [255, 255, 255];

/// Why a picture was left see-through in a flattened file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Left {
    /// Another picture, a shape, a table or words are behind it.
    OnOtherThings,
    /// The page's background is a gradient, a texture or a picture, not one flat colour.
    OnBackground,
    /// The page's colour has its own ink, which an image would only approximate.
    OnInk,
}

impl Left {
    /// The sentence that tells a person why, without the pages.
    pub fn because(self) -> &'static str {
        match self {
            Self::OnOtherThings => {
                "A see-through picture sits on other pictures or shapes, so it was left see-through"
            }
            Self::OnBackground => {
                "A see-through picture sits on a gradient, texture or picture background, so it was left see-through"
            }
            Self::OnInk => {
                "A see-through picture sits on a colour with its own ink, so it was left see-through"
            }
        }
    }
}

/// What to do with the picture an item draws.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Plan {
    /// Blend onto this colour and write it opaque.
    Flatten(Rgb),
    /// Leave it see-through, for this reason.
    Live(Left),
}

/// The box that holds a rectangle turned by `degrees` about its middle.
pub fn turned_bounds(rect: &SceneRect, degrees: f64) -> SceneRect {
    if degrees == 0.0 {
        return *rect;
    }
    let (sin, cos) = degrees.to_radians().sin_cos();
    let (hw, hh) = (rect.w / 2.0, rect.h / 2.0);
    let (cx, cy) = (rect.x + hw, rect.y + hh);
    let w = (hw * cos).abs() * 2.0 + (hh * sin).abs() * 2.0;
    let h = (hw * sin).abs() * 2.0 + (hh * cos).abs() * 2.0;
    SceneRect {
        x: cx - w / 2.0,
        y: cy - h / 2.0,
        w,
        h,
    }
}

/// Whether two boxes share any area worth speaking of (more than a hundredth of a point each way).
fn overlaps(a: &SceneRect, b: &SceneRect) -> bool {
    const SLIVER: f64 = 0.01;
    a.x + SLIVER < b.x + b.w
        && b.x + SLIVER < a.x + a.w
        && a.y + SLIVER < b.y + b.h
        && b.y + SLIVER < a.y + a.h
}

/// Whether the item draws anything inside `area`.
fn draws_within(kind: &SceneKind, area: &SceneRect) -> bool {
    match kind {
        SceneKind::Background { .. } => false,
        SceneKind::Picture { rect, rotation, .. } => {
            overlaps(&turned_bounds(rect, *rotation), area)
        }
        SceneKind::Shape {
            rect,
            rotation,
            fill,
            stroke,
            ..
        } => {
            (fill.is_some() || stroke.is_some()) && overlaps(&turned_bounds(rect, *rotation), area)
        }
        SceneKind::Table { tints, rules, .. } => {
            tints.iter().any(|(rect, _)| overlaps(rect, area))
                || rules.iter().any(|rule| {
                    let line = SceneRect {
                        x: rule.x1.min(rule.x2) - rule.width / 2.0,
                        y: rule.y1.min(rule.y2) - rule.width / 2.0,
                        w: (rule.x1 - rule.x2).abs() + rule.width,
                        h: (rule.y1 - rule.y2).abs() + rule.width,
                    };
                    overlaps(&line, area)
                })
        }
        SceneKind::Text {
            at,
            size,
            rotation,
            lines,
        } => {
            !lines.is_empty()
                && overlaps(
                    &turned_bounds(
                        &SceneRect {
                            x: at.0,
                            y: at.1,
                            w: size.0,
                            h: size.1,
                        },
                        *rotation,
                    ),
                    area,
                )
        }
    }
}

/// What to do with the picture that `page.items[at]` draws, or `None` when that item is not a
/// picture. Only what is drawn before it can show through it: words and art drawn after it sit on
/// top and change nothing. `ink` is set for a file whose colours are ink.
pub fn plan(page: &PageScene, at: usize, swatches: &[BookSwatch], ink: bool) -> Option<Plan> {
    let item = page.items.get(at)?;
    let area = match &item.kind {
        // The page's own picture has nothing but paper behind it.
        SceneKind::Background {
            bg: Background::Picture { .. },
            ..
        } => return Some(Plan::Flatten(PAPER)),
        SceneKind::Picture { rect, rotation, .. } => turned_bounds(rect, *rotation),
        _ => return None,
    };
    let below = &page.items[..at];
    if below.iter().any(|b| draws_within(&b.kind, &area)) {
        return Some(Plan::Live(Left::OnOtherThings));
    }
    // The page's background is the last one drawn under the picture.
    let background = below.iter().rev().find_map(|b| match &b.kind {
        SceneKind::Background { bg, .. } => Some(bg),
        _ => None,
    });
    match background {
        None => Some(Plan::Flatten(PAPER)),
        Some(Background::Colour { colour }) => {
            let paint = Swatches(swatches).resolve(colour);
            if ink && paint.ink.is_some() {
                Some(Plan::Live(Left::OnInk))
            } else {
                Some(Plan::Flatten(paint.rgb))
            }
        }
        Some(_) => Some(Plan::Live(Left::OnBackground)),
    }
}

/// The pages on which pictures were left see-through, by reason, for the notes of an export.
#[derive(Debug, Default)]
pub struct Kept(BTreeMap<Left, BTreeSet<usize>>);

impl Kept {
    pub fn add(&mut self, why: Left, page: usize) {
        self.0.entry(why).or_default().insert(page);
    }

    /// One line for each reason, naming the pages (counted from 1).
    pub fn notes(&self) -> Vec<String> {
        self.0
            .iter()
            .map(|(why, pages)| {
                let list: Vec<String> = pages.iter().map(usize::to_string).collect();
                let which = if list.len() == 1 { "page" } else { "pages" };
                format!("{} ({which} {})", why.because(), list.join(", "))
            })
            .collect()
    }
}

/// Blends straight-alpha `rgb` (three bytes a pixel) onto `backdrop`, source over, in the gamma
/// space of the pixels, the way a viewer blends a soft mask.
#[allow(clippy::chunks_exact_to_as_chunks)]
pub fn composite(rgb: &mut [u8], alpha: &[u8], backdrop: Rgb) {
    for (pixel, &a) in rgb.chunks_exact_mut(3).zip(alpha) {
        if a == 255 {
            continue;
        }
        let a = u32::from(a);
        for (c, &b) in pixel.iter_mut().zip(&backdrop) {
            *c = ((u32::from(*c) * a + u32::from(b) * (255 - a) + 127) / 255) as u8;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_turned_box_is_held_by_its_corners() {
        let r = SceneRect {
            x: 0.0,
            y: 0.0,
            w: 100.0,
            h: 50.0,
        };
        let t = turned_bounds(&r, 90.0);
        assert!((t.w - 50.0).abs() < 1e-9 && (t.h - 100.0).abs() < 1e-9);
        assert!((t.x - 25.0).abs() < 1e-9 && (t.y + 25.0).abs() < 1e-9);
        assert_eq!(turned_bounds(&r, 0.0), r);
    }

    #[test]
    fn the_blend_is_the_picture_over_the_backdrop() {
        let mut rgb = [200, 100, 0, 10, 20, 30, 1, 2, 3];
        composite(&mut rgb, &[255, 0, 128], [255, 192, 203]);
        assert_eq!(&rgb[..3], &[200, 100, 0]);
        assert_eq!(&rgb[3..6], &[255, 192, 203]);
        // Half way, rounded.
        assert_eq!(rgb[6], ((128 + 255 * 127 + 127) / 255) as u8);
    }

    #[test]
    fn notes_name_the_pages() {
        let mut kept = Kept::default();
        kept.add(Left::OnOtherThings, 7);
        kept.add(Left::OnOtherThings, 4);
        let notes = kept.notes();
        assert_eq!(notes.len(), 1);
        assert!(notes[0].ends_with("(pages 4, 7)"), "{notes:?}");
    }
}
