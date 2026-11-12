// Flow: story text poured through a chain of frames and their columns
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

// Paragraphs fill a column from the top; when the next line will not fit, the rest carries on in
// the next column, in this frame or the next one in the chain. Widows and orphans are kept apart,
// a baseline grid can hold lines to its rules, and what does not fit anywhere is reported as
// overset.

use crate::error::LayoutError;
use crate::fonts::FontBook;
use std::borrow::Cow;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex};

use crate::paragraph::{layout_paragraph, Align, Line, ParagraphLayout, ParagraphSpec, Span, Wrap};

/// The room a frame gives text, in points from the frame's top-left.
#[derive(Debug, Clone, PartialEq)]
pub struct FrameBox {
    pub width: f64,
    pub height: f64,
    /// Space kept clear inside every edge.
    pub inset: f64,
    pub columns: u8,
    /// Space between columns.
    pub gutter: f64,
    /// Whether the frame is on a right-hand page; none when that is not known.
    pub recto: Option<bool>,
}

impl FrameBox {
    pub fn new(width: f64, height: f64) -> Self {
        Self {
            width,
            height,
            inset: 0.0,
            columns: 1,
            gutter: 12.0,
            recto: None,
        }
    }
}

/// Where a paragraph may begin.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Start {
    /// Straight after the paragraph before.
    #[default]
    Anywhere,
    /// At the top of a new page.
    Page,
    /// At the top of a right-hand page.
    Recto,
}

/// A paragraph of the story, with the spacing around it.
#[derive(Debug, Clone, PartialEq)]
pub struct StoryParagraph {
    pub spans: Vec<Span>,
    pub align: Align,
    pub first_indent: f64,
    pub hang_indent: f64,
    /// How far every line is moved in from the left and from the right.
    pub indent: f64,
    pub indent_right: f64,
    pub wrap: Wrap,
    pub hyphenate: bool,
    pub space_before: f64,
    pub space_after: f64,
    pub start: Start,
}

impl StoryParagraph {
    pub fn new(spans: Vec<Span>) -> Self {
        Self {
            spans,
            align: Align::Left,
            first_indent: 0.0,
            hang_indent: 0.0,
            indent: 0.0,
            indent_right: 0.0,
            wrap: Wrap::Reflow,
            hyphenate: true,
            space_before: 0.0,
            space_after: 0.0,
            start: Start::Anywhere,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FlowOptions {
    /// The fewest lines of a paragraph that may be left alone at the foot of a column.
    pub orphans: usize,
    /// The fewest lines of a paragraph that may begin a column.
    pub widows: usize,
    /// Hold baselines to a grid: `(increment, offset)` from the frame's top.
    pub baseline_grid: Option<(f64, f64)>,
    pub optimal: bool,
}

impl Default for FlowOptions {
    fn default() -> Self {
        Self {
            orphans: 2,
            widows: 2,
            baseline_grid: None,
            optimal: true,
        }
    }
}

/// A line in its place. Coordinates are from the frame's top-left.
#[derive(Debug, Clone, PartialEq)]
pub struct PlacedLine {
    pub paragraph: usize,
    /// Byte offset into the paragraph's text where this line's paragraph layout began (non-zero
    /// when the paragraph carried over from an earlier column).
    pub offset: usize,
    pub frame: usize,
    pub column: usize,
    /// Left edge of the line's content.
    pub x: f64,
    /// Top of the line box, and the baseline within it.
    pub top: f64,
    pub baseline: f64,
    pub line: Line,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Flowed {
    pub lines: Vec<PlacedLine>,
    /// Text is left over that no column had room for.
    pub overset: bool,
    /// Where the leftover text begins: the paragraph and its byte offset.
    pub overset_from: Option<(usize, usize)>,
    /// A preserved line (a line of verse) was wider than its column and wrapped.
    pub overlong: bool,
    /// The tallest the text reached in each frame, to size a frame to its text.
    pub used_height: Vec<f64>,
    /// How many frames the placed lines reach: the last frame holding a line, counted from one;
    /// none when no line was placed.
    pub frames_reached: usize,
}

struct Column {
    frame: usize,
    index: usize,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

fn columns_of(frames: &[FrameBox]) -> Vec<Column> {
    let mut out = Vec::new();
    for (frame, f) in frames.iter().enumerate() {
        let count = usize::from(f.columns.max(1));
        let inner_w = (f.width - 2.0 * f.inset).max(1.0);
        let width = ((inner_w - (count - 1) as f64 * f.gutter) / count as f64).max(1.0);
        for index in 0..count {
            out.push(Column {
                frame,
                index,
                x: f.inset + index as f64 * (width + f.gutter),
                y: f.inset,
                width,
                height: (f.height - 2.0 * f.inset).max(0.0),
            });
        }
    }
    out
}

/// `spans` from byte `from` of their joined text on.
fn clip_spans(spans: &[Span], from: usize) -> Vec<Span> {
    let mut start = 0;
    let mut out = Vec::new();
    for span in spans {
        let end = start + span.text.len();
        // An empty paragraph still has a line, and the space around it.
        if end > from || (from == 0 && end == 0 && out.is_empty()) {
            let cut = from.saturating_sub(start);
            out.push(Span {
                text: span.text[cut..].to_string(),
                style: span.style.clone(),
            });
        }
        start = end;
    }
    out
}

fn half_leading(line: &Line) -> f64 {
    ((line.height - line.ascent - line.descent) / 2.0).max(0.0)
}

/// Paragraphs already shaped and broken, kept between layouts, so a keystroke in one paragraph of a
/// long story shapes that paragraph again and takes the rest as they were.
///
/// A paragraph is found by everything its lines depend on: its words and their looks, its
/// alignment, indents, wrap and hyphenation, the language, whether breaks are optimal and the width
/// of the column. The fonts are not part of the key: the owner makes a new cache when the fonts
/// the book can use change.
#[derive(Default)]
pub struct ParagraphCache {
    laid: Mutex<HashMap<u64, Arc<ParagraphLayout>>>,
    /// Runs whose font covers every character they hold, found by what the run says and the font
    /// it asks for, so a story's runs are not checked again character by character.
    covered: Mutex<std::collections::HashSet<u64>>,
}

/// The most paragraphs kept; past it the cache starts again.
const PARAGRAPHS_KEPT: usize = 16_384;

impl ParagraphCache {
    pub fn new() -> Self {
        Self::default()
    }

    /// How many paragraphs are kept.
    pub fn len(&self) -> usize {
        self.laid.lock().expect("the cache is not poisoned").len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// `span` split at the characters its font lacks, as [`crate::fallback::split_with_fallback`]
    /// does it; a run found to need no split is not checked again.
    pub fn split_with_fallback(&self, fonts: &FontBook, span: &Span) -> Vec<Span> {
        let mut h = std::collections::hash_map::DefaultHasher::new();
        span.text.hash(&mut h);
        span.style.family.hash(&mut h);
        span.style.weight.hash(&mut h);
        span.style.italic.hash(&mut h);
        let key = h.finish();
        if self
            .covered
            .lock()
            .expect("the cache is not poisoned")
            .contains(&key)
        {
            return vec![span.clone()];
        }
        let pieces = crate::fallback::split_with_fallback(fonts, span);
        if pieces.len() == 1 && pieces[0] == *span {
            let mut covered = self.covered.lock().expect("the cache is not poisoned");
            if covered.len() >= PARAGRAPHS_KEPT {
                covered.clear();
            }
            covered.insert(key);
        }
        pieces
    }

    fn lay(
        &self,
        book: &FontBook,
        spec: &ParagraphSpec<'_>,
        measure: f64,
    ) -> Result<Arc<ParagraphLayout>, LayoutError> {
        let key = paragraph_key(spec, measure);
        if let Some(hit) = self
            .laid
            .lock()
            .expect("the cache is not poisoned")
            .get(&key)
        {
            return Ok(Arc::clone(hit));
        }
        let made = Arc::new(layout_paragraph(book, spec, measure)?);
        let mut laid = self.laid.lock().expect("the cache is not poisoned");
        if laid.len() >= PARAGRAPHS_KEPT {
            laid.clear();
        }
        laid.insert(key, Arc::clone(&made));
        Ok(made)
    }
}

fn paragraph_key(spec: &ParagraphSpec<'_>, measure: f64) -> u64 {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    for span in spec.spans {
        span.text.hash(&mut h);
        let st = &span.style;
        st.family.hash(&mut h);
        st.lang.hash(&mut h);
        st.size_pt.to_bits().hash(&mut h);
        st.weight.hash(&mut h);
        st.italic.hash(&mut h);
        st.tracking_em.to_bits().hash(&mut h);
        st.features.hash(&mut h);
        st.line_height.to_bits().hash(&mut h);
    }
    (spec.align as u8).hash(&mut h);
    spec.first_indent.to_bits().hash(&mut h);
    spec.hang_indent.to_bits().hash(&mut h);
    spec.language.hash(&mut h);
    spec.hyphenate.hash(&mut h);
    (spec.wrap as u8).hash(&mut h);
    spec.max_consecutive_hyphens.hash(&mut h);
    spec.optimal.hash(&mut h);
    measure.to_bits().hash(&mut h);
    h.finish()
}

/// Pours `story` through `frames` in order.
pub fn flow(
    book: &FontBook,
    story: &[StoryParagraph],
    frames: &[FrameBox],
    language: &str,
    options: &FlowOptions,
) -> Result<Flowed, LayoutError> {
    flow_cached(book, story, frames, language, options, None)
}

/// As [`flow`], taking paragraphs from `cache` when it has them and leaving the ones it shaped.
pub fn flow_cached(
    book: &FontBook,
    story: &[StoryParagraph],
    frames: &[FrameBox],
    language: &str,
    options: &FlowOptions,
    cache: Option<&ParagraphCache>,
) -> Result<Flowed, LayoutError> {
    flow_with(book, story, frames, language, options, cache, true)
}

/// As [`flow_cached`], placing every line but keeping none: `lines` is empty and the rest of the
/// answer (`overset`, `frames_reached`, `used_height`) is what a full flow gives. It is what asking
/// how much room a story needs wants, without copying a long story's lines to throw them away.
pub fn flow_extent(
    book: &FontBook,
    story: &[StoryParagraph],
    frames: &[FrameBox],
    language: &str,
    options: &FlowOptions,
    cache: Option<&ParagraphCache>,
) -> Result<Flowed, LayoutError> {
    flow_with(book, story, frames, language, options, cache, false)
}

fn flow_with(
    book: &FontBook,
    story: &[StoryParagraph],
    frames: &[FrameBox],
    language: &str,
    options: &FlowOptions,
    cache: Option<&ParagraphCache>,
    keep_lines: bool,
) -> Result<Flowed, LayoutError> {
    let columns = columns_of(frames);
    let mut out = Flowed {
        lines: Vec::new(),
        overset: false,
        overset_from: None,
        overlong: false,
        used_height: vec![0.0; frames.len()],
        frames_reached: 0,
    };
    let mut col = 0;
    let mut y = 0.0;
    for (p, para) in story.iter().enumerate() {
        if para.start != Start::Anywhere {
            // A chapter opens at the top of a page, or of a right-hand one: the frames between are
            // left empty.
            let suits = |c: &Column| {
                c.index == 0 && (para.start == Start::Page || frames[c.frame].recto == Some(true))
            };
            let here = y == 0.0 && columns.get(col).is_some_and(suits);
            if !here {
                match (col + 1..columns.len()).find(|&i| suits(&columns[i])) {
                    Some(next) => {
                        col = next;
                        y = 0.0;
                    }
                    None => {
                        out.overset = true;
                        out.overset_from = Some((p, 0));
                        return Ok(out);
                    }
                }
            }
        }
        let mut from = 0;
        // Space before a paragraph, except at the top of a column.
        let mut before = para.space_before;
        loop {
            let Some(column) = columns.get(col) else {
                out.overset = true;
                out.overset_from = Some((p, from));
                return Ok(out);
            };
            // A paragraph from its start is its spans as they are (but for a run of empty ones at the
            // head, which `clip_spans` drops); only the rest of one that carried over is cut.
            let whole = from == 0
                && !(para.spans.len() > 1
                    && para.spans[0].text.is_empty()
                    && para.spans[1].text.is_empty());
            let clipped: Cow<'_, [Span]> = if whole {
                Cow::Borrowed(&para.spans)
            } else {
                Cow::Owned(clip_spans(&para.spans, from))
            };
            let measure = (column.width - para.indent - para.indent_right).max(1.0);
            let mut spec = ParagraphSpec::new(&clipped, language);
            spec.align = para.align;
            spec.first_indent = if from == 0 { para.first_indent } else { 0.0 };
            spec.hang_indent = para.hang_indent;
            spec.wrap = para.wrap;
            spec.hyphenate = para.hyphenate;
            spec.optimal = options.optimal;
            let laid: Arc<ParagraphLayout> = match cache {
                Some(cache) => cache.lay(book, &spec, measure)?,
                None => Arc::new(layout_paragraph(book, &spec, measure)?),
            };
            out.overlong |= laid.overlong;
            if laid.lines.is_empty() {
                break;
            }
            let top_gap = if y == 0.0 { 0.0 } else { before };
            let total = laid.lines.len();

            // How many lines fit below the cursor.
            let mut cursor = y + top_gap;
            let mut placed: Vec<(f64, f64)> = Vec::new(); // (top, baseline) of each line that fits
            for line in &laid.lines {
                let mut top = cursor;
                let mut baseline = top + half_leading(line) + line.ascent;
                if let Some((increment, offset)) = options
                    .baseline_grid
                    .filter(|(increment, _)| *increment > 0.0)
                {
                    let absolute = column.y + baseline - offset;
                    let snapped = (absolute / increment - 1e-9).ceil() * increment + offset;
                    let shift = snapped - (column.y + baseline);
                    top += shift;
                    baseline += shift;
                }
                if top + line.height > column.height + 1e-6 {
                    break;
                }
                placed.push((top, baseline));
                cursor = top + line.height;
            }
            let mut fit = placed.len();
            if fit < total {
                // Keep orphans and widows apart: a paragraph that cannot leave enough on either
                // side of the break moves whole.
                if fit < options.orphans.min(total) {
                    fit = 0;
                } else if total - fit < options.widows {
                    fit = total.saturating_sub(options.widows);
                    if fit < options.orphans.min(total) {
                        fit = 0;
                    }
                }
            }
            let later_is_roomier = columns[col + 1..]
                .iter()
                .any(|c| c.height > column.height + 1e-6);
            if fit == 0 && y == 0.0 && !placed.is_empty() && !later_is_roomier {
                // The rules want more room than this column has even when empty, and no later column
                // has more: keep what fits, since moving the paragraph on would never help.
                fit = placed.len();
            }
            if fit == 0 && y == 0.0 {
                // Nothing fits even at the top of an empty column: more room will not help.
                // Try the next column, and report overset when none is left.
                col += 1;
                y = 0.0;
                if col >= columns.len() {
                    out.overset = true;
                    out.overset_from = Some((p, from));
                    return Ok(out);
                }
                continue;
            }
            for (k, line) in laid.lines.iter().take(fit).enumerate() {
                let (top, baseline) = placed[k];
                out.used_height[column.frame] =
                    out.used_height[column.frame].max(column.y + top + line.height);
                out.frames_reached = out.frames_reached.max(column.frame + 1);
                if keep_lines {
                    out.lines.push(PlacedLine {
                        paragraph: p,
                        offset: from,
                        frame: column.frame,
                        column: column.index,
                        x: column.x + para.indent + line.x,
                        top: column.y + top,
                        baseline: column.y + baseline,
                        line: line.clone(),
                    });
                }
            }
            if fit == total {
                y = placed.last().map_or(y, |(top, _)| *top)
                    + laid.lines.last().map_or(0.0, |l| l.height)
                    + para.space_after;
                if y > column.height {
                    y = column.height;
                }
                break;
            }
            if fit > 0 {
                // Carry on after the last line placed.
                from += laid.lines[fit - 1].text.end;
                // The text a line covers stops at its last word: skip the space that follows.
                let rest = &laid.text[laid.lines[fit - 1].text.end..];
                from += rest.len() - rest.trim_start().len();
            }
            before = 0.0;
            col += 1;
            y = 0.0;
        }
    }
    Ok(out)
}
