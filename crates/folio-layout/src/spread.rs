// Laying out a spread: the text of every frame on it, and the checks
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use folio_model::{
    page_labels, Book, Flow, Frame, FrameId, InlineContent, MasterId, PageLabel, Pt, ShapeFrame,
    Spread, SpreadId, StyleId, TableFrame, TextFrame,
};

use crate::assets::AssetFacts;
use crate::error::LayoutError;
use crate::flow::{flow, FlowOptions, Flowed, FrameBox, PlacedLine, StoryParagraph};
use crate::fonts::{FontBook, FontRequest};
use crate::paragraph::{Span, Wrap};
use crate::preflight::{asset_checks, geometry_checks};
use crate::styles::{ResolvedStyle, StyleSheet};
use crate::wire::{
    FrameLayout, LayoutLine, LayoutRect, LayoutRun, Origin, PreflightItem, PreflightKind,
    SpreadLayout,
};

/// Lays out the text of a book. Cheap to make; holds nothing but what it was given.
pub struct Layouter<'a> {
    pub book: &'a Book,
    pub styles: &'a StyleSheet,
    pub fonts: &'a FontBook,
    /// Optimal breaks (slower) or first-fit while the person is typing.
    pub optimal: bool,
    /// What is known of the book's pictures, for preflight. Without it, pictures are not checked.
    pub assets: Option<&'a AssetFacts>,
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
            assets: None,
        }
    }

    /// Checks the pictures against what is known of them too.
    pub fn with_assets(mut self, assets: &'a AssetFacts) -> Self {
        self.assets = Some(assets);
        self
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

    /// The text of the master `id` on a page of its own, with `{page}` and the other tokens as they
    /// are written, so the master can be edited. `spread` in the answer is the master's id.
    pub fn layout_master(&self, id: &MasterId) -> Result<Option<SpreadLayout>, LayoutError> {
        let Some(master) = self.book.master_pages.iter().find(|m| &m.id == id) else {
            return Ok(None);
        };
        let mut frames = Vec::new();
        for frame in &master.frames {
            match frame {
                Frame::Text(text) => frames.push(self.own_text(text)?.0),
                Frame::Shape(shape) => frames.extend(self.shape_text(shape)?),
                _ => {}
            }
        }
        Ok(Some(SpreadLayout {
            spread: SpreadId(id.0.clone()),
            frames,
            preflight: Vec::new(),
        }))
    }

    fn spread(&self, spread: &Spread) -> Result<SpreadLayout, LayoutError> {
        let mut frames = Vec::new();
        let mut preflight = geometry_checks(self.book, spread);
        if let Some(facts) = self.assets {
            preflight.extend(asset_checks(self.book, spread, facts));
        }
        let mut flowed: Vec<FlowLayout> = Vec::new();
        for frame in &spread.frames {
            if let Frame::Shape(shape) = frame {
                if let Some(laid) = self.shape_text(shape)? {
                    if laid.overset {
                        preflight.push(PreflightItem::of_frame(
                            PreflightKind::OversetText,
                            &spread.id,
                            &shape.base.id,
                        ));
                    }
                    frames.push(laid);
                }
                continue;
            }
            if let Frame::Table(table) = frame {
                let cells = self.table_text(table)?;
                if cells.iter().any(|c| c.overset) {
                    preflight.push(PreflightItem::of_frame(
                        PreflightKind::OversetText,
                        &spread.id,
                        &table.base.id,
                    ));
                }
                frames.extend(cells);
                continue;
            }
            let Frame::Text(text) = frame else {
                continue;
            };
            let layout = match &text.flow {
                None => {
                    let (laid, overlong) = self.own_text(&self.with_tokens(spread, text))?;
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
                            preflight.push(PreflightItem::of_frame(
                                PreflightKind::OversetText,
                                &spread.id,
                                &text.base.id,
                            ));
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
                preflight.push(PreflightItem::of_frame(
                    PreflightKind::OversetText,
                    &spread.id,
                    &text.base.id,
                ));
            }
            frames.push(layout.clone());
        }
        frames.extend(self.master_frames(spread)?);
        Ok(SpreadLayout {
            spread: spread.id.clone(),
            frames,
            preflight,
        })
    }

    /// The text frames of the master pages the spread's pages use, laid out where they sit on the
    /// spread, with `{page}` and `{book.title}` filled in.
    fn master_frames(&self, spread: &Spread) -> Result<Vec<FrameLayout>, LayoutError> {
        let mut out = Vec::new();
        let labels = self.labels_of(spread);
        for (index, page) in spread.pages.iter().enumerate() {
            let Some(master) = page
                .master
                .as_ref()
                .and_then(|id| self.book.master_pages.iter().find(|m| &m.id == id))
            else {
                continue;
            };
            let hidden = spread
                .master_overrides
                .iter()
                .find(|o| o.page_id == page.id)
                .map(|o| o.hide.as_slice())
                .unwrap_or(&[]);
            let label = labels.get(index).cloned().unwrap_or_default();
            let left = self.book.trim.w.0 * index as f64;
            for frame in &master.frames {
                if let Frame::Shape(shape) = frame {
                    if hidden.contains(&shape.base.id) {
                        continue;
                    }
                    let mut copy = shape.clone();
                    copy.base.rect.x = Pt(copy.base.rect.x.0 + left);
                    copy.base.id = FrameId(format!("{}@{}", shape.base.id.0, page.id.0));
                    self.fill_shape_tokens(&mut copy, &label);
                    if let Some(mut laid) = self.shape_text(&copy)? {
                        laid.origin = Some(Origin {
                            x: copy.base.rect.x.0,
                            y: copy.base.rect.y.0,
                        });
                        out.push(laid);
                    }
                    continue;
                }
                let Frame::Text(text) = frame else {
                    continue;
                };
                if hidden.contains(&text.base.id) {
                    continue;
                }
                let mut copy = text.clone();
                copy.base.rect.x = Pt(copy.base.rect.x.0 + left);
                copy.base.id = FrameId(format!("{}@{}", text.base.id.0, page.id.0));
                self.fill_tokens(&mut copy, &label);
                let (mut laid, _) = self.own_text(&copy)?;
                laid.origin = Some(Origin {
                    x: copy.base.rect.x.0,
                    y: copy.base.rect.y.0,
                });
                out.push(laid);
            }
        }
        Ok(out)
    }

    /// `{page}`, `{section}` and `{book.title}` in some words, filled in for a page.
    fn tokens(&self, words: &str, label: &PageLabel) -> String {
        words
            .replace("{page}", &label.label)
            .replace("{section}", &label.section)
            .replace("{book.title}", &self.book.title)
    }

    /// The tokens in a text frame's words, filled in for a page.
    fn fill_tokens(&self, text: &mut TextFrame, label: &PageLabel) {
        if let Some(content) = &mut text.content {
            for run in &mut content.inline {
                run.text = self.tokens(&run.text, label);
            }
        }
    }

    /// The tokens in a shape's words, filled in for a page.
    fn fill_shape_tokens(&self, shape: &mut ShapeFrame, label: &PageLabel) {
        if let Some(content) = &mut shape.content {
            for run in &mut content.inline {
                run.text = self.tokens(&run.text, label);
            }
        }
    }

    /// A text frame of the spread with its tokens filled in, for the page its middle is over. A
    /// frame the person made, or copied from a master, can carry them as well as a master's own.
    fn with_tokens(&self, spread: &Spread, text: &TextFrame) -> TextFrame {
        let mut copy = text.clone();
        let middle = text.base.rect.x.0 + text.base.rect.w.0 / 2.0;
        let page = (middle / self.book.trim.w.0).floor().max(0.0) as usize;
        let page = page.min(spread.pages.len().saturating_sub(1));
        let label = self
            .labels_of(spread)
            .get(page)
            .cloned()
            .unwrap_or_default();
        self.fill_tokens(&mut copy, &label);
        copy
    }

    /// What each page of `spread` is called, from the book's sections: its number as written, and
    /// the name of its section.
    fn labels_of(&self, spread: &Spread) -> Vec<PageLabel> {
        page_labels(&self.book.spreads)
            .into_iter()
            .zip(&self.book.spreads)
            .find(|(_, s)| s.id == spread.id)
            .map(|(labels, _)| labels)
            .unwrap_or_default()
    }

    /// The words of a shape, laid out in the body of its kind and centred top to bottom in it. A
    /// shape that can hold words always has a layout, even with none, so an editor knows where to go.
    fn shape_text(&self, shape: &ShapeFrame) -> Result<Option<FrameLayout>, LayoutError> {
        if !shape.shape.holds_text() {
            return Ok(None);
        }
        let inset = shape.shape.text_inset();
        let (w, h) = (shape.base.rect.w.0, shape.base.rect.h.0);
        let (x, y) = (inset.left * w, inset.top * h);
        let inner_w = (w * (1.0 - inset.left - inset.right)).max(0.0);
        let inner_h = (h * (1.0 - inset.top - inset.bottom)).max(0.0);
        let mut base = shape.base.clone();
        base.rect = folio_model::Rect {
            x: Pt(0.0),
            y: Pt(0.0),
            w: Pt(inner_w),
            h: Pt(inner_h),
        };
        let words = TextFrame {
            base,
            style: shape
                .style
                .clone()
                .unwrap_or_else(|| StyleId(shape.shape.default_style().to_string())),
            content: shape.content.clone(),
            flow: None,
            columns: 1,
            baseline_lock: false,
            overrides: shape.overrides.clone(),
        };
        let resolved = self.styles.resolve(&words.style, words.overrides.as_ref());
        let (mut laid, _) = self.own_text(&words)?;
        laid.text_align = Some(match resolved.align {
            crate::paragraph::Align::Left => folio_model::Align::Left,
            crate::paragraph::Align::Center => folio_model::Align::Center,
            crate::paragraph::Align::Right => folio_model::Align::Right,
            crate::paragraph::Align::Justify => folio_model::Align::Justify,
        });
        let used = laid
            .lines
            .iter()
            .map(|l| l.top + l.height)
            .fold(0.0, f64::max);
        let shift = y + ((inner_h - used) / 2.0).max(0.0);
        for line in &mut laid.lines {
            line.x += x;
            line.top += shift;
            line.baseline += shift;
        }
        laid.text_box = Some(LayoutRect {
            x,
            y,
            w: inner_w,
            h: inner_h,
        });
        Ok(Some(laid))
    }

    /// The words of each cell of a table, in the cell's room and from the top of it. A cell's layout
    /// is named `<table>#<row>.<column>` and its lines are placed from the table's top-left, with
    /// `text_box` the cell's room, so the canvas draws and edits a cell where it is. Header rows are
    /// set bold.
    fn table_text(&self, table: &TableFrame) -> Result<Vec<FrameLayout>, LayoutError> {
        const PAD_X: f64 = 4.0;
        const PAD_Y: f64 = 3.0;
        let rows = table.rows.len().max(1) as f64;
        let row_h = table.base.rect.h.0 / rows;
        let style = table
            .style
            .clone()
            .unwrap_or_else(|| StyleId("cell".to_string()));
        let mut out = Vec::new();
        for (r, row) in table.rows.iter().enumerate() {
            let mut x = 0.0;
            for (c, cell) in row.iter().enumerate() {
                let w = table.base.rect.w.0 * table.cols.get(c).copied().unwrap_or(0.0);
                let (inner_w, inner_h) =
                    ((w - 2.0 * PAD_X).max(0.0), (row_h - 2.0 * PAD_Y).max(0.0));
                let mut overrides = table.overrides.clone();
                if r < usize::from(table.head) {
                    overrides.get_or_insert_with(Default::default).bold = Some(true);
                }
                let mut base = table.base.clone();
                base.id = FrameId(format!("{}#{r}.{c}", table.base.id.0));
                base.rect = folio_model::Rect {
                    x: Pt(0.0),
                    y: Pt(0.0),
                    w: Pt(inner_w),
                    h: Pt(inner_h),
                };
                let words = TextFrame {
                    base,
                    style: style.clone(),
                    content: cell.content.clone(),
                    flow: None,
                    columns: 1,
                    baseline_lock: false,
                    overrides,
                };
                let resolved = self.styles.resolve(&words.style, words.overrides.as_ref());
                let (mut laid, _) = self.own_text(&words)?;
                laid.text_align = Some(match resolved.align {
                    crate::paragraph::Align::Left => folio_model::Align::Left,
                    crate::paragraph::Align::Center => folio_model::Align::Center,
                    crate::paragraph::Align::Right => folio_model::Align::Right,
                    crate::paragraph::Align::Justify => folio_model::Align::Justify,
                });
                let (left, top) = (x + PAD_X, r as f64 * row_h + PAD_Y);
                for line in &mut laid.lines {
                    line.x += left;
                    line.top += top;
                    line.baseline += top;
                }
                laid.text_box = Some(LayoutRect {
                    x: left,
                    y: top,
                    w: inner_w,
                    h: inner_h,
                });
                out.push(laid);
                x += w;
            }
        }
        Ok(out)
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
        PreflightItem::of_frame(PreflightKind::PoemLineTooLong, &spread.id, &text.base.id)
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
        origin: None,
        text_box: None,
        text_align: None,
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
            origin: None,
            text_box: None,
            text_align: None,
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
        origin: None,
        text_box: None,
        text_align: None,
    }
}
