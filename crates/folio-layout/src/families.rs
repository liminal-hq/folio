// The typeface families a book asks for, wherever it asks
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::collections::BTreeSet;

use folio_model::{Book, Frame, InlineContent, StyleOverrides};

use crate::fonts::{FALLBACK_SANS, FALLBACK_SERIF};

/// Every family the book names: the look's typeface, a style's own, a frame's override and a run's.
/// A style marked `sans` asks for the sans fallback. Sorted, one of each; a family the book never
/// names but a style falls back to is not listed.
pub fn families_used(book: &Book) -> Vec<String> {
    let mut found: BTreeSet<String> = BTreeSet::new();
    let styles = &book.styles;
    found.insert(if styles.font_family.is_empty() {
        FALLBACK_SERIF.to_string()
    } else {
        styles.font_family.clone()
    });
    for def in styles.styles.values() {
        if let Some(own) = &def.family {
            found.insert(own.clone());
        } else if def.sans {
            found.insert(FALLBACK_SANS.to_string());
        }
    }
    let frames = book
        .spreads
        .iter()
        .flat_map(|s| s.frames.iter())
        .chain(book.master_pages.iter().flat_map(|m| m.frames.iter()));
    for frame in frames {
        match frame {
            Frame::Text(t) => {
                overrides(&mut found, t.overrides.as_ref());
                runs(&mut found, t.content.as_ref());
            }
            Frame::Shape(s) => {
                overrides(&mut found, s.overrides.as_ref());
                runs(&mut found, s.content.as_ref());
            }
            Frame::Table(t) => {
                overrides(&mut found, t.overrides.as_ref());
                for cell in t.rows.iter().flatten() {
                    runs(&mut found, cell.content.as_ref());
                }
            }
            _ => {}
        }
    }
    found.into_iter().collect()
}

fn overrides(found: &mut BTreeSet<String>, overrides: Option<&StyleOverrides>) {
    if let Some(font) = overrides.and_then(|o| o.font.as_ref()) {
        found.insert(font.clone());
    }
}

fn runs(found: &mut BTreeSet<String>, content: Option<&InlineContent>) {
    for run in content.map(|c| c.inline.as_slice()).unwrap_or_default() {
        if let Some(font) = &run.font {
            found.insert(font.clone());
        }
    }
}
