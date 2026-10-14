// Shaping: turning a run of text in one font into positioned glyphs
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::str::FromStr;

use rustybuzz::{Direction, Feature, Language, UnicodeBuffer};
use ttf_parser::Tag;

use crate::fonts::Font;

/// A run of text in one font, size and weight.
#[derive(Debug, Clone)]
pub struct Run<'a> {
    pub text: &'a str,
    pub size_pt: f64,
    pub weight: u16,
    /// Extra space after every glyph, in ems (letter spacing).
    pub tracking_em: f64,
    /// OpenType features beyond the defaults (kerning and ligatures are on already).
    pub features: &'a [Feature],
    /// A BCP 47 language such as `en-CA`, which picks language-specific forms.
    pub language: Option<&'a str>,
}

impl<'a> Run<'a> {
    pub fn new(text: &'a str, size_pt: f64) -> Self {
        Self {
            text,
            size_pt,
            weight: 400,
            tracking_em: 0.0,
            features: &[],
            language: None,
        }
    }
}

/// An OpenType feature switched on (`1`) or off (`0`) for the whole run, such as `smcp` or `liga`.
pub fn feature(tag: &str, value: u32) -> Option<Feature> {
    let bytes: [u8; 4] = tag.as_bytes().try_into().ok()?;
    Some(Feature::new(Tag::from_bytes(&bytes), value, ..))
}

/// One glyph placed in the run. Distances are in points.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Glyph {
    pub id: u16,
    /// Byte offset into the run's text of the first character this glyph stands for.
    pub cluster: u32,
    pub advance: f64,
    pub dx: f64,
    pub dy: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Shaped {
    pub glyphs: Vec<Glyph>,
    /// The sum of the advances.
    pub width: f64,
}

/// Shapes `run` in `font`, left to right, with kerning and ligatures.
pub fn shape(font: &Font, run: &Run<'_>) -> Shaped {
    if run.text.is_empty() {
        return Shaped {
            glyphs: Vec::new(),
            width: 0.0,
        };
    }
    let face = font.face(run.size_pt, run.weight);
    let mut buffer = UnicodeBuffer::new();
    buffer.push_str(run.text);
    buffer.set_direction(Direction::LeftToRight);
    buffer.guess_segment_properties();
    if let Some(language) = run.language.and_then(|l| Language::from_str(l).ok()) {
        buffer.set_language(language);
    }
    let out = rustybuzz::shape(&face, run.features, buffer);
    let scale = run.size_pt / f64::from(face.units_per_em());
    let tracking = run.tracking_em * run.size_pt;
    let mut width = 0.0;
    let glyphs = out
        .glyph_infos()
        .iter()
        .zip(out.glyph_positions())
        .map(|(info, pos)| {
            let advance = f64::from(pos.x_advance) * scale + tracking;
            width += advance;
            Glyph {
                id: info.glyph_id as u16,
                cluster: info.cluster,
                advance,
                dx: f64::from(pos.x_offset) * scale,
                dy: f64::from(pos.y_offset) * scale,
            }
        })
        .collect();
    Shaped { glyphs, width }
}
