// A paragraph laid out into lines: shaping, hyphenation, breaking and placing the words
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::ops::Range;
use std::sync::Arc;

use rustybuzz::Feature;

use crate::breaker::{break_lines, first_fit, Break, Item, Params, INFINITE};
use crate::error::LayoutError;
use crate::fonts::{Font, FontBook, FontRequest};
use crate::hyphenate::hyphenation_points;
use crate::shape::{feature, shape, Glyph, Run, Shaped};

/// What a stretch of text looks like.
#[derive(Debug, Clone, PartialEq)]
pub struct TextStyle {
    pub family: String,
    pub size_pt: f64,
    pub weight: u16,
    pub italic: bool,
    /// Extra space after every glyph, in ems.
    pub tracking_em: f64,
    /// OpenType features beyond the defaults, as `(tag, value)`: `("smcp", 1)`, `("liga", 0)`.
    pub features: Vec<(String, u32)>,
    /// Line height as a multiple of the size.
    pub line_height: f64,
}

impl TextStyle {
    pub fn new(family: &str, size_pt: f64) -> Self {
        Self {
            family: family.to_string(),
            size_pt,
            weight: 400,
            italic: false,
            tracking_em: 0.0,
            features: Vec::new(),
            line_height: 1.3,
        }
    }
}

/// A run of text in one style.
#[derive(Debug, Clone, PartialEq)]
pub struct Span {
    pub text: String,
    pub style: TextStyle,
}

impl Span {
    pub fn new(text: &str, style: TextStyle) -> Self {
        Self {
            text: text.to_string(),
            style,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Align {
    #[default]
    Left,
    Center,
    Right,
    Justify,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Wrap {
    /// Words flow from line to line.
    #[default]
    Reflow,
    /// Every line break in the text stays (poetry). A line too long for the measure wraps and
    /// hangs, and the layout says so.
    Preserve,
}

#[derive(Debug, Clone)]
pub struct ParagraphSpec<'a> {
    pub spans: &'a [Span],
    pub align: Align,
    /// Indent of the first line, in points.
    pub first_indent: f64,
    /// How far lines that wrap out of a preserved line hang in, in points.
    pub hang_indent: f64,
    /// A BCP 47 tag such as `en-CA`.
    pub language: &'a str,
    pub hyphenate: bool,
    pub wrap: Wrap,
    /// The most lines in a row that may end in a hyphen.
    pub max_consecutive_hyphens: usize,
    /// Optimal (Knuth–Plass) breaks, or first-fit while the person is typing.
    pub optimal: bool,
}

impl<'a> ParagraphSpec<'a> {
    pub fn new(spans: &'a [Span], language: &'a str) -> Self {
        Self {
            spans,
            align: Align::Left,
            first_indent: 0.0,
            hang_indent: 0.0,
            language,
            hyphenate: true,
            wrap: Wrap::Reflow,
            max_consecutive_hyphens: 2,
            optimal: true,
        }
    }
}

/// A word, or a piece of one, in one style, placed on a line. `x` is from the line's start.
#[derive(Debug, Clone, PartialEq)]
pub struct LineRun {
    /// Index of the span the text is from.
    pub span: usize,
    pub text: String,
    pub x: f64,
    pub width: f64,
    pub glyphs: Vec<Glyph>,
    /// The hyphen drawn at a line's end; it is not part of the text.
    pub hyphen: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Line {
    /// Where the first run starts, from the left of the measure.
    pub x: f64,
    /// The width of the content, glue included.
    pub width: f64,
    pub runs: Vec<LineRun>,
    /// The text this line shows, as a range of `ParagraphLayout::text`. Spaces that end the line
    /// are not in it.
    pub text: Range<usize>,
    /// The line's tallest ascent and descent and its leading, in points.
    pub ascent: f64,
    pub descent: f64,
    pub height: f64,
    /// How far the glue was stretched (negative: squeezed).
    pub ratio: f64,
    pub hyphenated: bool,
    /// Too wide for the measure even with its spaces squeezed as far as they go.
    pub overfull: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParagraphLayout {
    /// All the spans' text joined.
    pub text: String,
    pub lines: Vec<Line>,
    /// The Knuth–Plass tolerance that worked, or `None` when first-fit had to break it.
    pub tolerance: Option<f64>,
    /// A preserved line wrapped because it was too long for the measure.
    pub overlong: bool,
}

/// One piece of a word: what it shows, in what span, shaped.
struct Piece {
    span: usize,
    range: Range<usize>,
    glyphs: Vec<Glyph>,
    width: f64,
}

/// The things a break can be: pieces of words, spaces between them, and hyphens.
enum Slot {
    Word(Vec<Piece>),
    Space { width: f64 },
    Hyphen { span: usize, width: f64 },
}

struct SpanFont {
    font: Arc<Font>,
    weight: u16,
    features: Vec<Feature>,
}

fn span_fonts(book: &FontBook, spans: &[Span]) -> Result<Vec<SpanFont>, LayoutError> {
    spans
        .iter()
        .map(|span| {
            let style = &span.style;
            let found =
                book.resolve(&FontRequest::new(&style.family, style.weight, style.italic))?;
            Ok(SpanFont {
                weight: found.font.weight_for(style.weight),
                font: found.font,
                features: style
                    .features
                    .iter()
                    .filter_map(|(tag, value)| feature(tag, *value))
                    .collect(),
            })
        })
        .collect()
}

struct Shaper<'a> {
    spans: &'a [Span],
    fonts: Vec<SpanFont>,
    language: &'a str,
}

impl Shaper<'_> {
    fn shape(&self, span: usize, text: &str) -> Shaped {
        let style = &self.spans[span].style;
        let f = &self.fonts[span];
        let mut run = Run::new(text, style.size_pt);
        run.weight = f.weight;
        run.tracking_em = style.tracking_em;
        run.features = &f.features;
        run.language = Some(self.language);
        shape(&f.font, &run)
    }

    fn width(&self, span: usize, text: &str) -> f64 {
        self.shape(span, text).width
    }
}

/// The span each byte of the joined text belongs to, as the end offset of each span.
fn span_ends(spans: &[Span]) -> Vec<usize> {
    spans
        .iter()
        .scan(0, |end, s| {
            *end += s.text.len();
            Some(*end)
        })
        .collect()
}

fn span_at(ends: &[usize], byte: usize) -> usize {
    ends.partition_point(|&end| end <= byte).min(ends.len() - 1)
}

/// Words and spaces in `range` of `text`: `(is_space, range)`. A run of spaces counts as one.
fn tokens(text: &str, range: Range<usize>) -> Vec<(bool, Range<usize>)> {
    let mut out: Vec<(bool, Range<usize>)> = Vec::new();
    for (offset, c) in text[range.clone()].char_indices() {
        let at = range.start + offset;
        let space = c.is_whitespace();
        match out.last_mut() {
            Some((was_space, r)) if *was_space == space => r.end = at + c.len_utf8(),
            _ => out.push((space, at..at + c.len_utf8())),
        }
    }
    out
}

/// A word split at `points`, each piece shaped in the spans it crosses.
fn word_slots(
    shaper: &Shaper<'_>,
    text: &str,
    ends: &[usize],
    word: Range<usize>,
    hyphenate: bool,
) -> Vec<Slot> {
    // Shape each stretch of the word that is one style, once.
    let mut fragments: Vec<(usize, Range<usize>, Shaped)> = Vec::new();
    let mut start = word.start;
    while start < word.end {
        let span = span_at(ends, start);
        let end = ends[span].min(word.end);
        fragments.push((span, start..end, shaper.shape(span, &text[start..end])));
        start = end;
    }

    // Where the word may break, keeping only the points a glyph starts at (a ligature can't be cut).
    let mut points: Vec<usize> = if hyphenate {
        hyphenation_points(&text[word.clone()], shaper.language)
            .into_iter()
            .map(|p| word.start + p)
            .filter(|&p| {
                fragments.iter().any(|(_, range, shaped)| {
                    p == range.start
                        || (range.contains(&p)
                            && shaped
                                .glyphs
                                .iter()
                                .any(|g| range.start + g.cluster as usize == p))
                })
            })
            .collect()
    } else {
        Vec::new()
    };
    points.insert(0, word.start);
    points.push(word.end);
    points.dedup();

    let mut slots = Vec::new();
    for (k, bounds) in points.windows(2).enumerate() {
        if k > 0 {
            let span = span_at(ends, bounds[0] - 1);
            let width = shaper.width(span, "-");
            slots.push(Slot::Hyphen { span, width });
        }
        let pieces = fragments
            .iter()
            .filter_map(|(span, range, shaped)| {
                let from = bounds[0].max(range.start);
                let to = bounds[1].min(range.end);
                if from >= to {
                    return None;
                }
                let (lo, hi) = (from - range.start, to - range.start);
                let first = shaped.glyphs.partition_point(|g| (g.cluster as usize) < lo);
                let last = shaped.glyphs.partition_point(|g| (g.cluster as usize) < hi);
                let glyphs = shaped.glyphs[first..last].to_vec();
                let width = glyphs.iter().map(|g| g.advance).sum();
                Some(Piece {
                    span: *span,
                    range: from..to,
                    glyphs,
                    width,
                })
            })
            .collect();
        slots.push(Slot::Word(pieces));
    }
    slots
}

const HYPHEN_PENALTY: f64 = 50.0;

/// Items for the breaker from the slots, and the slot each item came from (`None` for the final
/// glue and break).
fn items_for(slots: &[Slot], justify: bool) -> (Vec<Item>, Vec<Option<usize>>) {
    // Ragged text may leave a lot of room at a line's end, so its spaces stretch far; justified
    // text stretches by half a space and shrinks by a third (TeX's values).
    let stretch_of = |width: f64| if justify { width * 0.5 } else { width * 4.0 };
    let mut items = Vec::new();
    let mut origin = Vec::new();
    for (i, slot) in slots.iter().enumerate() {
        match slot {
            Slot::Word(pieces) => {
                items.push(Item::Box {
                    width: pieces.iter().map(|p| p.width).sum(),
                });
                origin.push(Some(i));
            }
            Slot::Space { width } => {
                items.push(Item::glue(*width, stretch_of(*width), width / 3.0));
                origin.push(Some(i));
            }
            Slot::Hyphen { width, .. } => {
                items.push(Item::Penalty {
                    width: *width,
                    penalty: HYPHEN_PENALTY,
                    flagged: true,
                });
                origin.push(Some(i));
            }
        }
    }
    for item in Item::forced_break() {
        items.push(item);
        origin.push(None);
    }
    (items, origin)
}

/// Breaks at which `max` or more lines in a row end in a hyphen: the third and later of each run.
fn hyphen_runs_too_long(items: &[Item], breaks: &[Break], max: usize) -> Option<usize> {
    let mut run = 0;
    for b in breaks {
        if matches!(items[b.at], Item::Penalty { flagged: true, .. }) {
            run += 1;
            if run > max {
                return Some(b.at);
            }
        } else {
            run = 0;
        }
    }
    None
}

struct Chunk {
    range: Range<usize>,
    first_indent: f64,
    hang: f64,
    hyphenate: bool,
    optimal: bool,
    max_hyphens: usize,
    align: Align,
    measure: f64,
    /// The span whose style sizes a line with no text in it.
    span: usize,
}

fn layout_chunk(
    shaper: &Shaper<'_>,
    text: &str,
    ends: &[usize],
    chunk: &Chunk,
) -> (Vec<Line>, Option<f64>) {
    // Slots: words and the spaces between them; leading and trailing space is dropped.
    let mut slots: Vec<Slot> = Vec::new();
    let toks = tokens(text, chunk.range.clone());
    for (n, (space, range)) in toks.iter().enumerate() {
        if *space {
            if n == 0 || n == toks.len() - 1 {
                continue;
            }
            let span = span_at(ends, range.start);
            slots.push(Slot::Space {
                width: shaper.width(span, " "),
            });
        } else {
            slots.extend(word_slots(
                shaper,
                text,
                ends,
                range.clone(),
                chunk.hyphenate,
            ));
        }
    }
    let (mut items, origin) = items_for(&slots, chunk.align == Align::Justify);
    let measure = chunk.measure;
    let width_of = |line: usize| {
        if line == 0 {
            (measure - chunk.first_indent).max(1.0)
        } else {
            (measure - chunk.hang).max(1.0)
        }
    };
    let params = Params::default();
    let (breaks, tolerance) = if chunk.optimal {
        // A third hyphen in a row is forbidden at its break, and the paragraph is broken again.
        let mut result = break_lines(&items, &width_of, &params);
        for _ in 0..32 {
            let Some(at) = hyphen_runs_too_long(&items, &result.breaks, chunk.max_hyphens) else {
                break;
            };
            if let Item::Penalty { penalty, .. } = &mut items[at] {
                *penalty = INFINITE;
            }
            result = break_lines(&items, &width_of, &params);
        }
        (result.breaks, result.tolerance)
    } else {
        (first_fit(&items, &width_of), None)
    };

    let mut lines = Vec::with_capacity(breaks.len());
    let mut from = 0;
    for (n, b) in breaks.iter().enumerate() {
        let avail = width_of(n);
        let indent = if n == 0 {
            chunk.first_indent
        } else {
            chunk.hang
        };
        lines.push(place_line(
            shaper,
            text,
            &slots,
            &items,
            &origin,
            from..b.at,
            b,
            avail,
            indent,
            chunk,
        ));
        from = b.at + 1;
    }
    (lines, tolerance)
}

#[allow(clippy::too_many_arguments)]
fn place_line(
    shaper: &Shaper<'_>,
    text: &str,
    slots: &[Slot],
    items: &[Item],
    origin: &[Option<usize>],
    span: Range<usize>,
    end: &Break,
    avail: f64,
    indent: f64,
    chunk: &Chunk,
) -> Line {
    let justify = chunk.align == Align::Justify;
    // Ragged lines keep their spaces as they are, except that a line the breaker had to squeeze is
    // squeezed: it must not run past the measure.
    let ratio = if justify {
        end.ratio
    } else {
        end.ratio.min(0.0)
    };
    let mut runs: Vec<LineRun> = Vec::new();
    let mut x = 0.0;
    let mut first: Option<usize> = None;
    let mut last = 0;
    let (mut ascent, mut descent, mut height) = (0.0f64, 0.0f64, 0.0f64);
    let mut measure_span = |span: usize| {
        let style = &shaper.spans[span].style;
        let m = shaper.fonts[span].font.metrics(style.size_pt);
        ascent = ascent.max(m.ascent);
        descent = descent.max(m.descent);
        height = height.max(style.size_pt * style.line_height);
    };
    let mut natural = 0.0;
    let mut squeezable = 0.0;
    for (idx, item) in items[span.clone()].iter().enumerate() {
        let Some(slot) = origin[span.start + idx] else {
            continue;
        };
        match &slots[slot] {
            Slot::Word(pieces) => {
                for p in pieces {
                    measure_span(p.span);
                    first.get_or_insert(p.range.start);
                    last = p.range.end;
                    runs.push(LineRun {
                        span: p.span,
                        text: text[p.range.clone()].to_string(),
                        x,
                        width: p.width,
                        glyphs: p.glyphs.clone(),
                        hyphen: false,
                    });
                    x += p.width;
                    natural += p.width;
                }
            }
            Slot::Space { width } => {
                let Item::Glue {
                    stretch, shrink, ..
                } = *item
                else {
                    continue;
                };
                let adjusted = if ratio >= 0.0 {
                    width + ratio * stretch
                } else {
                    width + ratio.max(-1.0) * shrink
                };
                x += adjusted;
                natural += width;
                squeezable += shrink;
            }
            Slot::Hyphen { .. } => {}
        }
    }
    // A line that ends at a hyphen draws one.
    let hyphenated = matches!(items[end.at], Item::Penalty { flagged: true, .. });
    if hyphenated {
        if let Some(slot) = origin[end.at] {
            if let Slot::Hyphen { span, width } = slots[slot] {
                let shaped = shaper.shape(span, "-");
                runs.push(LineRun {
                    span,
                    text: "-".to_string(),
                    x,
                    width,
                    glyphs: shaped.glyphs,
                    hyphen: true,
                });
                x += width;
                natural += width;
                measure_span(span);
            }
        }
    }
    if runs.is_empty() {
        measure_span(chunk.span);
    }
    let used = x;
    let offset = match chunk.align {
        Align::Left | Align::Justify => 0.0,
        Align::Center => ((avail - used) / 2.0).max(0.0),
        Align::Right => (avail - used).max(0.0),
    };
    let overfull = natural - squeezable > avail + 1e-6;
    Line {
        x: indent + offset,
        width: used,
        runs,
        text: first.unwrap_or(0)..last.max(first.unwrap_or(0)),
        ascent,
        descent,
        height,
        ratio: end.ratio,
        hyphenated,
        overfull,
    }
}

/// Lays `spec` out into lines `measure` points wide.
pub fn layout_paragraph(
    book: &FontBook,
    spec: &ParagraphSpec<'_>,
    measure: f64,
) -> Result<ParagraphLayout, LayoutError> {
    let text: String = spec.spans.iter().map(|s| s.text.as_str()).collect();
    let fonts = span_fonts(book, spec.spans)?;
    let shaper = Shaper {
        spans: spec.spans,
        fonts,
        language: spec.language,
    };
    let ends = span_ends(spec.spans);
    if ends.is_empty() {
        return Ok(ParagraphLayout {
            text,
            lines: Vec::new(),
            tolerance: Some(Params::default().tolerance),
            overlong: false,
        });
    }

    let preserve = spec.wrap == Wrap::Preserve;
    let ranges: Vec<Range<usize>> = if preserve {
        let mut out = Vec::new();
        let mut start = 0;
        for line in text.split('\n') {
            out.push(start..start + line.len());
            start += line.len() + 1;
        }
        out
    } else {
        std::iter::once(0..text.len()).collect()
    };

    let mut lines = Vec::new();
    let mut tolerance = Some(Params::default().tolerance);
    let mut overlong = false;
    for range in ranges {
        let chunk = Chunk {
            range: range.clone(),
            first_indent: if preserve { 0.0 } else { spec.first_indent },
            hang: if preserve { spec.hang_indent } else { 0.0 },
            hyphenate: spec.hyphenate && !preserve,
            optimal: spec.optimal && !preserve,
            max_hyphens: spec.max_consecutive_hyphens,
            align: if preserve && spec.align == Align::Justify {
                Align::Left
            } else {
                spec.align
            },
            measure,
            span: span_at(&ends, range.start.min(text.len().saturating_sub(1))),
        };
        let (mut made, tol) = layout_chunk(&shaper, &text, &ends, &chunk);
        if made.len() > 1 && preserve {
            overlong = true;
        }
        if made.is_empty() {
            made.push(empty_line(
                &shaper,
                span_at(&ends, range.start.min(text.len().saturating_sub(1))),
            ));
        }
        tolerance = match (tolerance, tol) {
            (Some(a), Some(b)) => Some(a.max(b)),
            _ => None,
        };
        lines.extend(made);
    }
    Ok(ParagraphLayout {
        text,
        lines,
        tolerance,
        overlong,
    })
}

fn empty_line(shaper: &Shaper<'_>, span: usize) -> Line {
    let style = &shaper.spans[span].style;
    let m = shaper.fonts[span].font.metrics(style.size_pt);
    Line {
        x: 0.0,
        width: 0.0,
        runs: Vec::new(),
        text: 0..0,
        ascent: m.ascent,
        descent: m.descent,
        height: style.size_pt * style.line_height,
        ratio: 0.0,
        hyphenated: false,
        overfull: false,
    }
}
