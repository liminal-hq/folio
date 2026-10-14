// Laying out a spread: the text of every frame on it, and the checks
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use folio_model::{
    Book, Flow, Frame, FrameId, InlineContent, Spread, SpreadId, StyleId, TextFrame,
};

use crate::error::LayoutError;
use crate::flow::{flow, FlowOptions, Flowed, FrameBox, PlacedLine, StoryParagraph};
use crate::fonts::{FontBook, FontRequest};
use crate::paragraph::{Span, Wrap};
use crate::preflight::geometry_checks;
use crate::styles::{ResolvedStyle, StyleSheet};
use crate::wire::{FrameLayout, LayoutLine, LayoutRun, PreflightItem, PreflightKind, SpreadLayout};

/// Lays out the text of a book. Cheap to make; holds nothing but what it was given.
pub struct Layouter<'a> {
    pub book: &'a Book,
    pub styles: &'a StyleSheet,
    pub fonts: &'a FontBook,
    /// Optimal breaks (slower) or first-fit while the person is typing.
    pub optimal: bool,
}

/// Inset of a text frame's text from its edges, in points. Frames carry no padding of their own
/// yet, so this is the one value for all of them.
const INSET: f64 = 0.0;

impl<'a> Layouter<'a> {
    pub fn new(book: &'a Book, styles: &'a StyleSheet, fonts: &'a FontBook) -> Self {
        Self {
            book,
            styles,
            fonts,
            optimal: true,
        }
    }

    fn options(&self) -> FlowOptions {
        FlowOptions {
            baseline_grid: self.book.grid.baseline.map(|b| (b.increment.0, b.offset.0)),
            optimal: self.optimal,
            ..FlowOptions::default()
        }
    }

    /// The layout of the spread `id`.
    pub fn layout_spread(&self, id: &SpreadId) -> Result<Option<SpreadLayout>, LayoutError> {
        let Some(spread) = self.book.spreads.iter().find(|s| &s.id == id) else {
            return Ok(None);
        };
        self.spread(spread).map(Some)
    }

    fn spread(&self, spread: &Spread) -> Result<SpreadLayout, LayoutError> {
        let mut frames = Vec::new();
        let mut preflight = geometry_checks(self.book, spread);
        let mut flowed: Vec<FlowLayout> = Vec::new();
        for frame in &spread.frames {
            let Frame::Text(text) = frame else {
                continue;
            };
            let layout = match &text.flow {
                None => {
                    let (laid, overlong) = self.own_text(text)?;
                    if overlong {
                        preflight.push(self.poem_item(spread, text));
                    }
                    laid
                }
                Some(slot) => {
                    let Some(chain) = self.book.flows.iter().find(|f| f.id == slot.id) else {
                        // The flow is gone: the frame shows its own words, if it has any.
                        let own = self.own_text(text)?;
                        if own.1 {
                            preflight.push(self.poem_item(spread, text));
                        }
                        if own.0.overset {
                            preflight.push(PreflightItem {
                                kind: PreflightKind::OversetText,
                                spread: spread.id.clone(),
                                frame: text.base.id.clone(),
                            });
                        }
                        frames.push(own.0);
                        continue;
                    };
                    if !flowed.iter().any(|f| f.id == chain.id.0) {
                        flowed.push(self.flow_chain(chain)?);
                    }
                    let FlowLayout {
                        flowed: laid,
                        order,
                        styles,
                        ..
                    } = flowed
                        .iter()
                        .find(|f| f.id == chain.id.0)
                        .expect("pushed above");
                    let index = order.iter().position(|f| *f == text.base.id);
                    if laid.overlong && index == Some(0) {
                        preflight.push(self.poem_item(spread, text));
                    }
                    flow_frame(&text.base.id, laid, index, styles)
                }
            };
            if layout.overset {
                preflight.push(PreflightItem {
                    kind: PreflightKind::OversetText,
                    spread: spread.id.clone(),
                    frame: text.base.id.clone(),
                });
            }
            frames.push(layout.clone());
        }
        Ok(SpreadLayout {
            spread: spread.id.clone(),
            frames,
            preflight,
        })
    }

    /// A text frame that holds its own words.
    fn own_text(&self, text: &TextFrame) -> Result<(FrameLayout, bool), LayoutError> {
        let story = self.story_of(text.content.as_ref(), &text.style, text);
        let wide = text.base.rect.w.0;
        let mut frame = FrameBox::new(wide, text.base.rect.h.0);
        frame.inset = INSET;
        frame.columns = text.columns.max(1);
        frame.gutter = self.book.grid.gutter.0;
        let mut options = self.options();
        if !text.baseline_lock {
            options.baseline_grid = None;
        }
        let laid = flow(
            self.fonts,
            &story,
            std::slice::from_ref(&frame),
            &self.book.language,
            &options,
        )?;
        Ok((
            frame_layout(&text.base.id, &laid, 0, &self.runs_info(&story)),
            laid.overlong,
        ))
    }

    fn poem_item(&self, spread: &Spread, text: &TextFrame) -> PreflightItem {
        PreflightItem {
            kind: PreflightKind::PoemLineTooLong,
            spread: spread.id.clone(),
            frame: text.base.id.clone(),
        }
    }

    /// The paragraphs of a text frame: one per line break, or the whole text for a style that
    /// keeps its breaks.
    fn story_of(
        &self,
        content: Option<&InlineContent>,
        style: &StyleId,
        text: &TextFrame,
    ) -> Vec<StoryParagraph> {
        let base = self.styles.resolve(style, text.overrides.as_ref());
        let Some(content) = content else {
            return Vec::new();
        };
        // Each run carries its own emphasis and, optionally, its own style.
        let mut spans: Vec<(Span, ResolvedStyle)> = Vec::new();
        for run in &content.inline {
            let resolved = match &run.style {
                Some(id) => self.styles.resolve(id, text.overrides.as_ref()),
                None => base.clone(),
            };
            let mut text_style = resolved.text.clone();
            if run.bold {
                text_style.weight = 700;
            }
            if run.italic {
                text_style.italic = true;
            }
            spans.push((Span::new(&run.text, text_style), resolved));
        }
        if base.wrap == Wrap::Preserve {
            return vec![paragraph_from(
                spans.into_iter().map(|(s, _)| s).collect(),
                &base,
            )];
        }
        // Split the runs at every line break into paragraphs.
        let mut paragraphs: Vec<Vec<Span>> = vec![Vec::new()];
        for (span, _) in spans {
            let mut parts = span.text.split('\n');
            if let Some(first) = parts.next() {
                paragraphs
                    .last_mut()
                    .expect("one is always open")
                    .push(Span::new(first, span.style.clone()));
            }
            for part in parts {
                paragraphs.push(vec![Span::new(part, span.style.clone())]);
            }
        }
        paragraphs
            .into_iter()
            .map(|spans| paragraph_from(spans, &base))
            .collect()
    }

    /// A flow's text poured through its chain of frames, and the order of the frames.
    fn flow_chain(&self, chain: &Flow) -> Result<FlowLayout, LayoutError> {
        let mut boxes = Vec::new();
        let mut order = Vec::new();
        let mut style = StyleId("body".to_string());
        let mut overrides = None;
        let mut lock = false;
        for id in &chain.frames {
            let Some(text) = self.find_text_frame(id) else {
                continue;
            };
            if order.is_empty() {
                style = text.style.clone();
                overrides = text.overrides.as_ref();
                lock = text.baseline_lock;
            }
            let mut b = FrameBox::new(text.base.rect.w.0, text.base.rect.h.0);
            b.columns = text.columns.max(1);
            b.gutter = self.book.grid.gutter.0;
            boxes.push(b);
            order.push(id.clone());
        }
        let resolved = self.styles.resolve(&style, overrides);
        // A style that keeps its breaks (verse) is one paragraph, as in a frame of its own.
        let story: Vec<StoryParagraph> = if resolved.wrap == Wrap::Preserve {
            vec![paragraph_from(
                vec![Span::new(&chain.source, resolved.text.clone())],
                &resolved,
            )]
        } else {
            chain
                .source
                .split('\n')
                .map(|line| paragraph_from(vec![Span::new(line, resolved.text.clone())], &resolved))
                .collect()
        };
        let mut options = self.options();
        if !lock {
            options.baseline_grid = None;
        }
        let laid = flow(self.fonts, &story, &boxes, &self.book.language, &options)?;
        let styles = self.runs_info(&story);
        Ok(FlowLayout {
            id: chain.id.0.clone(),
            flowed: laid,
            order,
            styles,
        })
    }

    fn find_text_frame(&self, id: &FrameId) -> Option<&TextFrame> {
        self.book
            .spreads
            .iter()
            .flat_map(|s| s.frames.iter())
            .find_map(|f| match f {
                Frame::Text(t) if t.base.id == *id => Some(t),
                _ => None,
            })
    }

    /// For the wire: the family, size and so on of a run come from its span.
    fn runs_info(&self, story: &[StoryParagraph]) -> Vec<Vec<RunStyle>> {
        story
            .iter()
            .map(|p| {
                p.spans
                    .iter()
                    .map(|s| {
                        // What is drawn is what was measured: the family that stood in, and the
                        // weight the font can do.
                        let found = self.fonts.resolve(&FontRequest::new(
                            &s.style.family,
                            s.style.weight,
                            s.style.italic,
                        ));
                        let (family, weight, italic) = match &found {
                            Ok(r) => (
                                r.font.family().to_string(),
                                r.font.weight_for(s.style.weight),
                                r.font.italic(),
                            ),
                            Err(_) => (s.style.family.clone(), s.style.weight, s.style.italic),
                        };
                        RunStyle {
                            family,
                            size: s.style.size_pt,
                            weight,
                            italic,
                            tracking: s.style.tracking_em,
                        }
                    })
                    .collect()
            })
            .collect()
    }
}

/// A flow's text poured through its frames: the frames in order, and the style of each run.
struct FlowLayout {
    id: String,
    flowed: Flowed,
    order: Vec<FrameId>,
    styles: Vec<Vec<RunStyle>>,
}

#[derive(Clone)]
struct RunStyle {
    family: String,
    size: f64,
    weight: u16,
    italic: bool,
    tracking: f64,
}

fn paragraph_from(spans: Vec<Span>, style: &ResolvedStyle) -> StoryParagraph {
    let mut p = StoryParagraph::new(spans);
    p.align = style.align;
    p.first_indent = style.first_indent;
    p.space_before = style.space_before;
    p.space_after = style.space_after;
    p.wrap = style.wrap;
    p.hyphenate = style.hyphenate;
    p
}

fn wire_line(placed: &PlacedLine, styles: &[Vec<RunStyle>]) -> LayoutLine {
    let runs = placed
        .line
        .runs
        .iter()
        .map(|r| {
            let s = styles
                .get(placed.paragraph)
                .and_then(|p| p.get(r.span))
                .cloned()
                .unwrap_or(RunStyle {
                    family: String::new(),
                    size: 0.0,
                    weight: 400,
                    italic: false,
                    tracking: 0.0,
                });
            LayoutRun {
                text: r.text.clone(),
                x: r.x,
                width: r.width,
                family: s.family,
                size: s.size,
                weight: s.weight,
                italic: s.italic,
                tracking: s.tracking,
                hyphen: r.hyphen,
            }
        })
        .collect();
    LayoutLine {
        x: placed.x,
        top: placed.top,
        baseline: placed.baseline,
        width: placed.line.width,
        height: placed.line.height,
        runs,
    }
}

fn frame_layout(
    id: &FrameId,
    flowed: &Flowed,
    frame: usize,
    styles: &[Vec<RunStyle>],
) -> FrameLayout {
    FrameLayout {
        frame: id.clone(),
        lines: flowed
            .lines
            .iter()
            .filter(|l| l.frame == frame)
            .map(|l| wire_line(l, styles))
            .collect(),
        overset: flowed.overset && frame + 1 == flowed.used_height.len().max(1),
    }
}

/// The lines of the `index`th frame of a flow.
fn flow_frame(
    id: &FrameId,
    laid: &Flowed,
    index: Option<usize>,
    styles: &[Vec<RunStyle>],
) -> FrameLayout {
    let Some(index) = index else {
        return FrameLayout {
            frame: id.clone(),
            lines: Vec::new(),
            overset: false,
        };
    };
    FrameLayout {
        frame: id.clone(),
        lines: laid
            .lines
            .iter()
            .filter(|l| l.frame == index)
            .map(|l| wire_line(l, styles))
            .collect(),
        overset: laid.overset && index + 1 == laid.used_height.len(),
    }
}
