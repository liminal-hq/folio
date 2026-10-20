// Glyph fallback: a character the font lacks is set in one that has it
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! A font covers some of the characters a book might hold. Where it has none for one, the shaper
//! draws an empty box. Before shaping, each span is split at the characters its font lacks, and
//! those are set in the first bundled font that has them, nearest in kind to the one asked for
//! (sans for sans, serif for serif), so a Greek or Cyrillic word in a Latin font still reads. What
//! no font covers is left and reported by preflight.

use crate::fonts::{FontBook, FontRequest, FALLBACK_SANS, FALLBACK_SERIF};
use crate::paragraph::Span;

/// The bundled families to try, nearest first, for a family that is serif or sans.
fn order(sans: bool) -> [&'static str; 4] {
    if sans {
        [
            FALLBACK_SANS,
            "Atkinson Hyperlegible",
            FALLBACK_SERIF,
            "Source Serif 4",
        ]
    } else {
        [
            FALLBACK_SERIF,
            "Source Serif 4",
            "Atkinson Hyperlegible",
            FALLBACK_SANS,
        ]
    }
}

/// Whether `family` looks like a sans, as `FontBook::resolve` judges a stand-in.
fn is_sans(family: &str) -> bool {
    let name = family.to_ascii_lowercase();
    name.contains("sans") || name == "inter" || name.contains("atkinson")
}

/// Characters that need no glyph of their own.
fn needs_no_glyph(c: char) -> bool {
    c.is_whitespace()
        || c.is_control()
        || matches!(c, '\u{200b}'..='\u{200f}' | '\u{2060}' | '\u{feff}')
}

/// A font's coverage, read once for the characters of a span.
struct Coverage<'a> {
    face: ttf_parser::Face<'a>,
}

impl Coverage<'_> {
    fn has(&self, c: char) -> bool {
        needs_no_glyph(c) || self.face.glyph_index(c).is_some_and(|g| g.0 != 0)
    }
}

/// `span` split at the characters its font lacks, the pieces that a fallback covers in that
/// fallback's family. A span whose font covers everything comes back whole.
pub fn split_with_fallback(fonts: &FontBook, span: &Span) -> Vec<Span> {
    let style = &span.style;
    let Ok(base) = fonts.resolve(&FontRequest::new(&style.family, style.weight, style.italic))
    else {
        return vec![span.clone()];
    };
    let Ok(face) = ttf_parser::Face::parse(base.font.data(), base.font.index()) else {
        return vec![span.clone()];
    };
    let own = Coverage { face };
    if span.text.chars().all(|c| own.has(c)) {
        return vec![span.clone()];
    }
    // The fallbacks, loaded once for this span.
    let mut fallbacks = Vec::new();
    for family in order(is_sans(&style.family)) {
        if family.eq_ignore_ascii_case(&style.family) {
            continue;
        }
        if let Ok(found) = fonts.resolve(&FontRequest::new(family, style.weight, style.italic)) {
            if !found.family_substituted {
                fallbacks.push(found.font);
            }
        }
    }
    let covers: Vec<Option<Coverage<'_>>> = fallbacks
        .iter()
        .map(|f| {
            ttf_parser::Face::parse(f.data(), f.index())
                .ok()
                .map(|face| Coverage { face })
        })
        .collect();
    // Which family sets each character: `None` is the span's own, `Some(i)` the i-th fallback.
    let pick = |c: char| -> Option<usize> {
        if own.has(c) {
            return None;
        }
        covers
            .iter()
            .position(|cov| cov.as_ref().is_some_and(|cov| cov.has(c)))
    };
    let mut out: Vec<Span> = Vec::new();
    let mut current: Option<(Option<usize>, String)> = None;
    let flush = |current: &mut Option<(Option<usize>, String)>, out: &mut Vec<Span>| {
        if let Some((which, text)) = current.take() {
            let mut piece = span.style.clone();
            if let Some(i) = which {
                piece.family = fallbacks[i].family().to_string();
            }
            out.push(Span::new(&text, piece));
        }
    };
    for c in span.text.chars() {
        // A character nobody covers stays in the span's own font, which draws its empty box.
        let which = pick(c);
        // A space stays with whatever is around it, so a word is not cut at it.
        let sticky = needs_no_glyph(c);
        match &mut current {
            Some((was, text)) if *was == which || sticky => text.push(c),
            _ => {
                flush(&mut current, &mut out);
                current = Some((which, c.to_string()));
            }
        }
    }
    flush(&mut current, &mut out);
    out
}

/// The characters of `text` that neither `family` nor any bundled fallback covers.
pub fn uncovered(fonts: &FontBook, family: &str, weight: u16, italic: bool, text: &str) -> String {
    let spans = split_with_fallback(
        fonts,
        &Span::new(
            text,
            crate::paragraph::TextStyle {
                weight,
                italic,
                ..crate::paragraph::TextStyle::new(family, 12.0)
            },
        ),
    );
    let mut missing = String::new();
    for span in spans {
        let Ok(found) = fonts.resolve(&FontRequest::new(&span.style.family, weight, italic)) else {
            continue;
        };
        let Ok(face) = ttf_parser::Face::parse(found.font.data(), found.font.index()) else {
            continue;
        };
        let cover = Coverage { face };
        for c in span.text.chars() {
            if !cover.has(c) && !missing.contains(c) {
                missing.push(c);
            }
        }
    }
    missing
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::paragraph::TextStyle;

    fn span(family: &str, text: &str) -> Span {
        Span::new(text, TextStyle::new(family, 12.0))
    }

    #[test]
    fn a_span_the_font_covers_comes_back_whole() {
        let fonts = FontBook::bundled();
        let pieces = split_with_fallback(
            &fonts,
            &span("Literata", "Down by the reeds, it’s 9 o’clock."),
        );
        assert_eq!(pieces.len(), 1);
        assert_eq!(pieces[0].style.family, "Literata");
    }

    #[test]
    fn cyrillic_in_a_font_without_it_is_set_in_a_font_that_has_it_and_the_rest_stays() {
        let fonts = FontBook::bundled();
        let pieces =
            split_with_fallback(&fonts, &span("Atkinson Hyperlegible", "Hello Привет world"));
        let text: String = pieces.iter().map(|p| p.text.as_str()).collect();
        assert_eq!(text, "Hello Привет world", "no character is lost");
        let cyrillic = pieces.iter().find(|p| p.text.contains("Привет")).unwrap();
        assert_ne!(cyrillic.style.family, "Atkinson Hyperlegible");
        assert_eq!(pieces[0].style.family, "Atkinson Hyperlegible");
        assert_eq!(pieces.last().unwrap().style.family, "Atkinson Hyperlegible");
    }

    #[test]
    fn a_sans_falls_back_to_a_sans_first() {
        let fonts = FontBook::bundled();
        let pieces = split_with_fallback(&fonts, &span("Atkinson Hyperlegible", "Привет"));
        assert_eq!(pieces.len(), 1);
        assert_eq!(pieces[0].style.family, "Inter");
    }

    #[test]
    fn what_no_bundled_font_covers_is_reported_and_what_one_does_is_not() {
        let fonts = FontBook::bundled();
        assert_eq!(
            uncovered(&fonts, "Literata", 400, false, "Hello 中 world 中"),
            "中"
        );
        assert_eq!(
            uncovered(&fonts, "Atkinson Hyperlegible", 400, false, "Привет"),
            ""
        );
        assert_eq!(uncovered(&fonts, "Literata", 400, false, "plain"), "");
    }
}
