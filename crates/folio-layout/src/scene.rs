// The page scene: each page of a spread as a plain list of things to draw, for the exporters
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! The canvas draws a spread from the layout and the model in TypeScript. An exporter cannot: it
//! has no webview, and it needs the very glyphs layout chose. The scene is what both would draw,
//! page by page, bottom to top, in points from each page's own top-left trim corner. A frame that
//! crosses the gutter is on both pages, clipped to each.

use folio_model::{
    Background, Colour, Fit, Frame, FrameId, MasterId, PageId, Rect, Shape, SpreadId, SpreadKind,
    TableFrame, TableLook,
};

use crate::error::LayoutError;
use crate::preflight::page_rects;
use crate::spread::Layouter;
use crate::wire::{FrameLayout, LayoutLine};

/// A rectangle in points, from a page's top-left trim corner.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SceneRect {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

impl SceneRect {
    fn of(rect: &Rect) -> Self {
        Self {
            x: rect.x.0,
            y: rect.y.0,
            w: rect.w.0,
            h: rect.h.0,
        }
    }

    fn shifted(self, dx: f64) -> Self {
        Self {
            x: self.x - dx,
            ..self
        }
    }

    fn meets(&self, other: &SceneRect) -> bool {
        self.x < other.x + other.w
            && other.x < self.x + self.w
            && self.y < other.y + other.h
            && other.y < self.y + self.h
    }

    fn within(&self, other: &SceneRect) -> bool {
        self.x >= other.x - EPS
            && self.y >= other.y - EPS
            && self.x + self.w <= other.x + other.w + EPS
            && self.y + self.h <= other.y + other.h + EPS
    }
}

const EPS: f64 = 1e-6;

/// A straight rule, in points.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SceneLine {
    pub x1: f64,
    pub y1: f64,
    pub x2: f64,
    pub y2: f64,
    pub width: f64,
}

/// What a scene item is.
#[derive(Clone, Debug, PartialEq)]
pub enum SceneKind {
    /// What fills the page behind everything, out to the bleed on its outer edges. When it spans
    /// both pages, `across` is its rectangle in this page's coordinates, so each page draws its
    /// half of one picture.
    Background {
        bg: Background,
        across: Option<SceneRect>,
    },
    /// A picture frame: the box, how the picture fills it and where. `asset` is `None` for a
    /// frame still waiting for its picture.
    Picture {
        rect: SceneRect,
        rotation: f64,
        asset: Option<String>,
        fit: Fit,
        focus: (f64, f64),
        crop: Option<SceneRect>,
    },
    /// A shape. `paths` is SVG path data in the unit square for the kinds that are one (the
    /// bubbles); the others are drawn from the box.
    Shape {
        rect: SceneRect,
        rotation: f64,
        shape: Shape,
        fill: Option<Colour>,
        stroke: Option<Colour>,
        stroke_width: Option<f64>,
        radius: Option<f64>,
        flip: bool,
        paths: Vec<String>,
    },
    /// A table's tint and rules; its words are separate text items.
    Table {
        rotation: f64,
        /// Rows tinted at the given opacity.
        tints: Vec<(SceneRect, f64)>,
        rules: Vec<SceneLine>,
    },
    /// Laid-out words: lines placed from the frame's top-left, each run with its glyphs.
    Text {
        /// Where the frame's top-left is.
        at: (f64, f64),
        /// The frame's width and height, which a turn is about the middle of.
        size: (f64, f64),
        rotation: f64,
        lines: Vec<LayoutLine>,
    },
}

/// One thing to draw, and where it came from.
#[derive(Clone, Debug, PartialEq)]
pub struct SceneItem {
    pub frame: FrameId,
    /// What to cut the item to: it crosses the gutter, so each page draws only its own side.
    pub clip: Option<SceneRect>,
    pub kind: SceneKind,
}

/// One page: its trim size and what is on it, bottom to top.
#[derive(Clone, Debug, PartialEq)]
pub struct PageScene {
    pub page: PageId,
    /// Which page of the spread, from 0.
    pub index: usize,
    pub width: f64,
    pub height: f64,
    pub items: Vec<SceneItem>,
}

/// A spread as the pages to draw.
#[derive(Clone, Debug, PartialEq)]
pub struct SpreadScene {
    pub spread: SpreadId,
    pub pages: Vec<PageScene>,
}

/// The rectangle to draw a picture of `natural` pixels in, inside `rect`: the twin of `focusedRect`
/// in `focusedRect.ts`. Fill covers the box and crops the rest, sliding so the `focus` point (0 to
/// 1 across and down the picture) is as near the middle of the box as the picture allows. Fit shows
/// the whole picture, centred, and Stretch pulls it to the box. A manual crop is drawn as a fill.
pub fn focused_rect(
    rect: SceneRect,
    natural: (f64, f64),
    fit: Fit,
    focus: (f64, f64),
) -> SceneRect {
    if fit == Fit::Stretch || natural.0 <= 0.0 || natural.1 <= 0.0 {
        return rect;
    }
    let (sx, sy) = (rect.w / natural.0, rect.h / natural.1);
    let scale = if fit == Fit::Fit {
        sx.min(sy)
    } else {
        sx.max(sy)
    };
    let (w, h) = (natural.0 * scale, natural.1 * scale);
    if fit == Fit::Fit {
        return SceneRect {
            x: rect.x + (rect.w - w) / 2.0,
            y: rect.y + (rect.h - h) / 2.0,
            w,
            h,
        };
    }
    let (fx, fy) = (focus.0.clamp(0.0, 1.0), focus.1.clamp(0.0, 1.0));
    let slide =
        |room: f64, size: f64, f: f64| (room / 2.0 - f * size).clamp((room - size).min(0.0), 0.0);
    SceneRect {
        x: rect.x + slide(rect.w, w, fx),
        y: rect.y + slide(rect.h, h, fy),
        w,
        h,
    }
}

/// What a bubble is drawn with, as paths in the unit square: the twin of `SHAPE_PATHS` in
/// `shapePaths.ts`, tested against one fixture.
pub fn shape_paths(shape: Shape) -> Vec<String> {
    match shape {
        Shape::Speech => vec![
            "M0.12 0H0.88Q1 0 1 0.12V0.58Q1 0.7 0.88 0.7H0.42L0.2 1L0.26 0.7H0.12Q0 0.7 0 0.58V0.12Q0 0 0.12 0Z"
                .into(),
        ],
        Shape::Thought => vec![
            cloud(),
            "M0.26 0.84a0.06 0.05 0 1 0 0.12 0a0.06 0.05 0 1 0 -0.12 0Z".into(),
            "M0.1 0.95a0.035 0.03 0 1 0 0.07 0a0.035 0.03 0 1 0 -0.07 0Z".into(),
        ],
        Shape::Shout => vec![burst()],
        _ => Vec::new(),
    }
}

/// A number as the TypeScript twin writes it: four decimals, trailing zeros dropped.
fn num(n: f64) -> String {
    let text = format!("{n:.4}");
    let text = text.trim_end_matches('0').trim_end_matches('.');
    if text == "-0" || text.is_empty() {
        "0".into()
    } else {
        text.into()
    }
}

fn cloud() -> String {
    let (bumps, cx, cy, rx, ry) = (11, 0.5_f64, 0.4_f64, 0.46_f64, 0.34_f64);
    let point = |i: usize| {
        let angle = (i as f64 / bumps as f64) * std::f64::consts::TAU - std::f64::consts::FRAC_PI_2;
        (cx + rx * angle.cos(), cy + ry * angle.sin())
    };
    let (x0, y0) = point(0);
    let mut d = format!("M{} {}", num(x0), num(y0));
    for i in 1..=bumps {
        let (x, y) = point(i % bumps);
        d.push_str(&format!("A0.11 0.11 0 0 1 {} {}", num(x), num(y)));
    }
    d.push('Z');
    d
}

fn burst() -> String {
    let spikes = 12;
    let mut d = String::new();
    for i in 0..spikes * 2 {
        let outer = i % 2 == 0;
        let angle =
            (i as f64 / (spikes * 2) as f64) * std::f64::consts::TAU - std::f64::consts::FRAC_PI_2;
        let r = if outer { 0.5 } else { 0.36 };
        d.push_str(&format!(
            "{}{} {}",
            if i == 0 { 'M' } else { 'L' },
            num(0.5 + r * angle.cos()),
            num(0.5 + r * angle.sin())
        ));
    }
    d.push('Z');
    d
}

/// The columns' left edges and then the table's right edge, in points on the spread.
fn column_edges(table: &TableFrame) -> Vec<f64> {
    let mut edges = vec![table.base.rect.x.0];
    for share in &table.cols {
        let last = *edges.last().expect("one edge to begin with");
        edges.push(last + table.base.rect.w.0 * share);
    }
    edges
}

/// A table's tint and rules in spread coordinates, as `TableArt` draws them.
fn table_art(table: &TableFrame) -> (Vec<(SceneRect, f64)>, Vec<SceneLine>) {
    let r = &table.base.rect;
    let (x, y, w, h) = (r.x.0, r.y.0, r.w.0, r.h.0);
    let rows = table.rows.len().max(1);
    let row_h = h / rows as f64;
    let xs = column_edges(table);
    let line = |x1, y1, x2, y2, width| SceneLine {
        x1,
        y1,
        x2,
        y2,
        width,
    };
    let mut rules = Vec::new();
    let mut tints = Vec::new();
    match table.look {
        TableLook::Grid => {
            for r in 0..=rows {
                let at = y + r as f64 * row_h;
                rules.push(line(x, at, x + w, at, 0.75));
            }
            for edge in &xs {
                rules.push(line(*edge, y, *edge, y + h, 0.75));
            }
        }
        TableLook::Lined => {
            for r in 1..=rows {
                let at = y + r as f64 * row_h;
                rules.push(line(x, at, x + w, at, 0.75));
            }
        }
        TableLook::Recipe => {
            if table.head > 0 {
                let at = y + f64::from(table.head) * row_h;
                rules.push(line(x, at, x + w, at, 1.25));
            }
            for r in 0..rows {
                let head = usize::from(table.head);
                if r >= head && (r - head) % 2 == 1 {
                    tints.push((
                        SceneRect {
                            x,
                            y: y + r as f64 * row_h,
                            w,
                            h: row_h,
                        },
                        0.07,
                    ));
                }
            }
        }
    }
    (tints, rules)
}

/// What a page's pieces are cut to: the page and the bleed beyond its outer edges. The edge at the
/// gutter is the fold, so nothing runs past it.
fn clip_of(kind: SpreadKind, index: usize, count: usize, bleed: f64, w: f64, h: f64) -> SceneRect {
    let (left_out, right_out) = match kind {
        SpreadKind::Opener => (false, true),
        SpreadKind::Closer => (true, false),
        SpreadKind::Cover | SpreadKind::Single => (true, true),
        SpreadKind::Spread => (index == 0, index + 1 == count),
    };
    let left = if left_out { bleed } else { 0.0 };
    let right = if right_out { bleed } else { 0.0 };
    SceneRect {
        x: -left,
        y: -bleed,
        w: w + left + right,
        h: h + 2.0 * bleed,
    }
}

impl Layouter<'_> {
    /// The pages of spread `id` as things to draw, or `None` when there is no such spread.
    pub fn scene_spread(&self, id: &SpreadId) -> Result<Option<SpreadScene>, LayoutError> {
        let Some(spread) = self.book.spreads.iter().find(|s| &s.id == id) else {
            return Ok(None);
        };
        let layout = self.layout_spread(id)?.expect("the spread exists");
        let pages = page_rects(self.book, spread);
        let (w, h, bleed) = (self.book.trim.w.0, self.book.trim.h.0, self.book.bleed.0);
        let backgrounds = folio_model::resolve(&self.book.master_pages, spread);
        let mut out: Vec<PageScene> = pages
            .iter()
            .enumerate()
            .map(|(index, _)| PageScene {
                page: spread.pages[index].id.clone(),
                index,
                width: w,
                height: h,
                items: Vec::new(),
            })
            .collect();

        // What is drawn, bottom to top, as (frame, rect it covers on the spread, kind).
        let mut drawn: Vec<Drawn> = Vec::new();

        // The master's furniture sits below the page's own frames.
        for (index, page) in spread.pages.iter().enumerate() {
            let Some(master) = page.master.as_ref().and_then(|m| self.master(m)) else {
                continue;
            };
            let hidden = spread
                .master_overrides
                .iter()
                .find(|o| o.page_id == page.id)
                .map(|o| o.hide.as_slice())
                .unwrap_or(&[]);
            let left = pages[index].rect.x.0;
            for frame in &master.frames {
                if hidden.contains(&frame.base().id) {
                    continue;
                }
                let mut copy = frame.clone();
                let base = match &mut copy {
                    Frame::Text(t) => &mut t.base,
                    Frame::Image(i) => &mut i.base,
                    Frame::Shape(s) => &mut s.base,
                    Frame::Table(t) => &mut t.base,
                    _ => continue,
                };
                base.rect.x.0 += left;
                base.id = FrameId(format!("{}@{}", base.id.0, page.id.0));
                if let Some(item) = self.draw_frame(&copy) {
                    drawn.push(item);
                }
            }
        }
        for frame in &spread.frames {
            if let Some(item) = self.draw_frame(frame) {
                drawn.push(item);
            }
        }
        // Words go over their frame's art, in the frame's own turn.
        let mut with_text: Vec<(FrameId, Rect, SceneKind)> = Vec::new();
        for d in drawn {
            let rotation = d.rotation;
            if let Some(kind) = d.kind {
                with_text.push((d.frame.clone(), d.rect, kind));
            }
            with_text.extend(text_of(&layout.frames, &d.frame, &d.rect, rotation));
        }

        for (index, page) in pages.iter().enumerate() {
            let slot = page.rect.x.0;
            let clip = clip_of(spread.kind, index, pages.len(), bleed, w, h);
            let scene = &mut out[index];
            if let Some(Some(resolved)) = backgrounds.get(index) {
                let across = (resolved.source == folio_model::BgSource::Span).then(|| {
                    // One picture across both pages: its rectangle in this page's coordinates.
                    SceneRect {
                        x: -(slot - pages[0].rect.x.0),
                        y: 0.0,
                        w: pages.len() as f64 * w,
                        h,
                    }
                });
                scene.items.push(SceneItem {
                    frame: FrameId(format!("bg@{}", spread.pages[index].id.0)),
                    clip: Some(clip),
                    kind: SceneKind::Background {
                        bg: resolved.bg.clone(),
                        across,
                    },
                });
            }
            for (frame, rect, kind) in &with_text {
                let turned = rect.turned_bounds(rotation_of(kind));
                let bounds = SceneRect::of(&turned).shifted(slot);
                if !bounds.meets(&clip) {
                    continue;
                }
                scene.items.push(SceneItem {
                    frame: frame.clone(),
                    clip: (!bounds.within(&clip)).then_some(clip),
                    kind: shift(kind.clone(), slot),
                });
            }
        }
        Ok(Some(SpreadScene {
            spread: spread.id.clone(),
            pages: out,
        }))
    }

    fn master(&self, id: &MasterId) -> Option<&folio_model::Master> {
        self.book.master_pages.iter().find(|m| &m.id == id)
    }

    /// A frame's own art (not its words), in spread coordinates; a text frame has none and only
    /// stands for its words.
    fn draw_frame(&self, frame: &Frame) -> Option<Drawn> {
        let base = frame.base();
        let kind = match frame {
            Frame::Image(image) => Some(SceneKind::Picture {
                rect: SceneRect::of(&image.base.rect),
                rotation: image.base.rotation,
                asset: image.image.asset.clone(),
                fit: image.image.fit,
                focus: image.image.focus,
                crop: image.image.crop.as_ref().map(SceneRect::of),
            }),
            Frame::Shape(shape) => Some(SceneKind::Shape {
                rect: SceneRect::of(&shape.base.rect),
                rotation: shape.base.rotation,
                shape: shape.shape,
                fill: shape.fill.clone(),
                stroke: shape.stroke.clone(),
                stroke_width: shape.stroke_width.map(|w| w.0),
                radius: shape.radius.map(|r| r.0),
                flip: shape.flip,
                paths: shape_paths(shape.shape),
            }),
            Frame::Table(table) => {
                let (tints, rules) = table_art(table);
                Some(SceneKind::Table {
                    rotation: table.base.rotation,
                    tints,
                    rules,
                })
            }
            Frame::Text(_) => None,
            _ => return None,
        };
        Some(Drawn {
            frame: base.id.clone(),
            rect: base.rect,
            rotation: base.rotation,
            kind,
        })
    }
}

/// A frame to draw: its box, its turn, and its art if it has any.
struct Drawn {
    frame: FrameId,
    rect: Rect,
    rotation: f64,
    kind: Option<SceneKind>,
}

fn rotation_of(kind: &SceneKind) -> f64 {
    match kind {
        SceneKind::Picture { rotation, .. }
        | SceneKind::Shape { rotation, .. }
        | SceneKind::Table { rotation, .. }
        | SceneKind::Text { rotation, .. } => *rotation,
        SceneKind::Background { .. } => 0.0,
    }
}

/// Moves a kind from spread coordinates to the page whose left edge is at `slot` on the spread.
fn shift(kind: SceneKind, slot: f64) -> SceneKind {
    match kind {
        SceneKind::Picture {
            rect,
            rotation,
            asset,
            fit,
            focus,
            crop,
        } => SceneKind::Picture {
            rect: rect.shifted(slot),
            rotation,
            asset,
            fit,
            focus,
            crop,
        },
        SceneKind::Shape {
            rect,
            rotation,
            shape,
            fill,
            stroke,
            stroke_width,
            radius,
            flip,
            paths,
        } => SceneKind::Shape {
            rect: rect.shifted(slot),
            rotation,
            shape,
            fill,
            stroke,
            stroke_width,
            radius,
            flip,
            paths,
        },
        SceneKind::Table {
            rotation,
            tints,
            rules,
        } => SceneKind::Table {
            rotation,
            tints: tints
                .into_iter()
                .map(|(r, o)| (r.shifted(slot), o))
                .collect(),
            rules: rules
                .into_iter()
                .map(|l| SceneLine {
                    x1: l.x1 - slot,
                    x2: l.x2 - slot,
                    ..l
                })
                .collect(),
        },
        SceneKind::Text {
            at,
            size,
            rotation,
            lines,
        } => SceneKind::Text {
            at: (at.0 - slot, at.1),
            size,
            rotation,
            lines,
        },
        background => background,
    }
}

/// The words of frame `frame`: the lines layout placed for it, and for a table the lines of each
/// of its cells, as text items positioned from the frame's top-left.
fn text_of(
    laid: &[FrameLayout],
    frame: &FrameId,
    rect: &Rect,
    rotation: f64,
) -> Vec<(FrameId, Rect, SceneKind)> {
    let owns = |id: &str| id == frame.0 || id.split('#').next() == Some(frame.0.as_str());
    laid.iter()
        .filter(|f| owns(&f.frame.0) && !f.lines.is_empty())
        .map(|f| {
            let at = f.origin.map(|o| (o.x, o.y)).unwrap_or((rect.x.0, rect.y.0));
            (
                f.frame.clone(),
                *rect,
                SceneKind::Text {
                    at,
                    size: (rect.w.0, rect.h.0),
                    rotation,
                    lines: f.lines.clone(),
                },
            )
        })
        .collect()
}
