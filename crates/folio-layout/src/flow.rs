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

/// Pours `story` through `frames` in order.
pub fn flow(
    book: &FontBook,
    story: &[StoryParagraph],
    frames: &[FrameBox],
    language: &str,
    options: &FlowOptions,
) -> Result<Flowed, LayoutError> {
    let columns = columns_of(frames);
    let mut out = Flowed {
        lines: Vec::new(),
        overset: false,
        overset_from: None,
        overlong: false,
        used_height: vec![0.0; frames.len()],
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
            let clipped = clip_spans(&para.spans, from);
            let mut spec = ParagraphSpec::new(&clipped, language);
            spec.align = para.align;
            spec.first_indent = if from == 0 { para.first_indent } else { 0.0 };
            spec.hang_indent = para.hang_indent;
            spec.wrap = para.wrap;
            spec.hyphenate = para.hyphenate;
            spec.optimal = options.optimal;
            let laid: ParagraphLayout = layout_paragraph(book, &spec, column.width)?;
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
                out.lines.push(PlacedLine {
                    paragraph: p,
                    offset: from,
                    frame: column.frame,
                    column: column.index,
                    x: column.x + line.x,
                    top: column.y + top,
                    baseline: column.y + baseline,
                    line: line.clone(),
                });
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
