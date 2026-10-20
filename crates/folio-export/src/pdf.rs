// The PDF exporter: each page of the page scene drawn with krilla
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::collections::HashMap;
use std::sync::Arc;

use folio_layout::scene::{focused_rect, SceneItem, SceneKind, SceneLine, SceneRect};
use folio_layout::wire::LayoutRun;
use folio_layout::{FontBook, Layouter, StyleSheet};
use folio_model::{Background, Book, Colour, Fit, Shape};
use krilla::color::rgb;
use krilla::geom::{PathBuilder, Point, Rect as KRect, Size, Transform};
use krilla::image::Image;
use krilla::metadata::Metadata;
use krilla::num::NormalizedF32;
use krilla::page::PageSettings;
use krilla::paint::{Fill, FillRule, LinearGradient, SpreadMethod, Stop, Stroke};
use krilla::surface::Surface;
use krilla::text::{Font, GlyphId, KrillaGlyph, Tag};
use krilla::{Data, SerializeSettings};

use crate::error::ExportError;

/// Where the bytes of a picture come from: the book's package, in the app.
pub trait Assets {
    fn bytes(&self, path: &str) -> Option<Vec<u8>>;
}

/// No pictures at all, for a book that has none or a test.
pub struct NoAssets;

impl Assets for NoAssets {
    fn bytes(&self, _path: &str) -> Option<Vec<u8>> {
        None
    }
}

/// What kind of PDF to make.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PdfTarget {
    /// To read on a screen or send round: one page per page, with the book's title and author.
    #[default]
    Sharing,
    /// Print-ready for Kindle Direct Publishing: single pages, nothing but the title and author in
    /// the file's information, no bookmarks or links.
    Print,
}

#[derive(Debug, Clone, Default)]
pub struct PdfOptions {
    pub target: PdfTarget,
}

/// Draws every page of `book` into a PDF.
pub fn export_pdf(
    book: &Book,
    styles: &StyleSheet,
    fonts: &FontBook,
    assets: &dyn Assets,
    options: &PdfOptions,
) -> Result<Vec<u8>, ExportError> {
    let layouter = Layouter::new(book, styles, fonts);
    let settings = SerializeSettings {
        compress_content_streams: true,
        xmp_metadata: options.target == PdfTarget::Sharing,
        ..SerializeSettings::default()
    };
    let mut document = krilla::Document::new_with(settings);
    let mut drawing = Drawing::new(fonts, assets);

    // A print PDF is the interior only: the cover is made for the cover file.
    let skip = usize::from(
        options.target == PdfTarget::Print
            && book
                .spreads
                .first()
                .is_some_and(|s| s.kind == folio_model::SpreadKind::Cover),
    );
    let mut scenes = Vec::new();
    for spread in book.spreads.iter().skip(skip) {
        if let Some(scene) = layouter.scene_spread(&spread.id)? {
            scenes.extend(scene.pages);
        }
    }
    if scenes.is_empty() {
        return Err(ExportError::NoPages);
    }
    let print = options.target == PdfTarget::Print;
    // Bleed is for a print file, and when anything bleeds every page is set up with it.
    let bleed = if print && book.bleed.0 > 0.0 && scenes.iter().any(spills) {
        book.bleed.0
    } else {
        0.0
    };
    // A printed book has an even number of pages: a blank one closes an odd count.
    let blank_at_end = print && scenes.len() % 2 == 1;
    let total = scenes.len() + usize::from(blank_at_end);
    for number in 0..total {
        let page = scenes.get(number);
        let (width, height) = page.map_or((book.trim.w.0, book.trim.h.0), |p| (p.width, p.height));
        let recto = page.map_or(number % 2 == 0, |p| p.recto);
        // The outer edge, away from the fold, is where the picture runs on into the bleed.
        let (left, right) = if recto { (0.0, bleed) } else { (bleed, 0.0) };
        let area = SceneRect {
            x: -left,
            y: -bleed,
            w: width + left + right,
            h: height + 2.0 * bleed,
        };
        let size = Size::from_wh(area.w as f32, area.h as f32)
            .ok_or_else(|| ExportError::Pdf("a page has no size".into()))?;
        let mut settings = PageSettings::new(size);
        if bleed > 0.0 {
            settings = settings
                .with_trim_box(KRect::from_xywh(
                    left as f32,
                    bleed as f32,
                    width as f32,
                    height as f32,
                ))
                .with_bleed_box(KRect::from_xywh(0.0, 0.0, area.w as f32, area.h as f32));
        }
        let mut pdf_page = document.start_page_with(settings);
        let mut surface = pdf_page.surface();
        if bleed > 0.0 {
            surface.push_transform(&Transform::from_translate(left as f32, bleed as f32));
        }
        if let Some(page) = page {
            for item in &page.items {
                drawing.item(&mut surface, item, &area);
            }
        }
        if bleed > 0.0 {
            surface.pop();
        }
        surface.finish();
        pdf_page.finish();
    }
    let mut metadata = Metadata::new().title(book.title.clone());
    let authors: Vec<String> = book.authors.iter().map(|a| a.name.clone()).collect();
    if !authors.is_empty() {
        metadata = metadata.authors(authors);
    }
    if options.target == PdfTarget::Sharing {
        metadata = metadata.language(book.language.clone());
    }
    document.set_metadata(metadata);
    document
        .finish()
        .map_err(|e| ExportError::Pdf(format!("{e:?}")))
}

struct Drawing<'a> {
    fonts: &'a FontBook,
    assets: &'a dyn Assets,
    /// The krilla fonts made so far, by file key, weight and size, since each variable instance is
    /// its own font to the PDF.
    faces: HashMap<(String, u16, u32), Option<Font>>,
    images: HashMap<String, Option<Image>>,
}

/// Whether something on the page runs past its trim on a side, so the file needs a bleed.
fn spills(page: &folio_layout::scene::PageScene) -> bool {
    page.items.iter().any(|item| match &item.kind {
        SceneKind::Background { .. } => true,
        SceneKind::Picture { rect, .. } | SceneKind::Shape { rect, .. } => {
            rect.y < -0.01
                || rect.y + rect.h > page.height + 0.01
                || rect.x < -0.01
                || rect.x + rect.w > page.width + 0.01
        }
        _ => false,
    })
}

fn colour(c: &Colour) -> rgb::Color {
    let hex = c.0.trim_start_matches('#');
    let byte = |at: usize| u8::from_str_radix(hex.get(at..at + 2).unwrap_or("00"), 16).unwrap_or(0);
    rgb::Color::new(byte(0), byte(2), byte(4))
}

/// The share of a radius a cubic curve's handles take to draw a quarter circle.
const KAPPA: f32 = 0.552_284_8;

const INK: &str = "#1c1b20";

fn fill_of(c: &Colour, opacity: f32) -> Fill {
    Fill {
        paint: colour(c).into(),
        opacity: NormalizedF32::new(opacity).unwrap_or(NormalizedF32::ONE),
        rule: FillRule::default(),
    }
}

fn krect(r: &SceneRect) -> Option<KRect> {
    KRect::from_xywh(
        r.x as f32,
        r.y as f32,
        r.w.max(0.0) as f32,
        r.h.max(0.0) as f32,
    )
}

impl<'a> Drawing<'a> {
    fn new(fonts: &'a FontBook, assets: &'a dyn Assets) -> Self {
        Self {
            fonts,
            assets,
            faces: HashMap::new(),
            images: HashMap::new(),
        }
    }

    /// Draws `item` into `area`: the page and the bleed beyond its outer edge, which is as far as
    /// anything goes. A frame across the gutter shows only its own side.
    fn item(&mut self, surface: &mut Surface, item: &SceneItem, area: &SceneRect) {
        let clip = *area;
        if let Some(rect) = krect(&clip) {
            let mut pb = PathBuilder::new();
            pb.push_rect(rect);
            if let Some(path) = pb.finish() {
                surface.push_clip_path(&path, &FillRule::default());
            }
        }
        match &item.kind {
            SceneKind::Background { bg, across } => {
                self.background(surface, bg, across.as_ref(), area)
            }
            SceneKind::Picture {
                rect,
                rotation,
                asset,
                fit,
                focus,
                ..
            } => self.picture(surface, rect, *rotation, asset.as_deref(), *fit, *focus),
            SceneKind::Shape { .. } => self.shape(surface, &item.kind),
            SceneKind::Table {
                rotation: _,
                tints,
                rules,
            } => {
                for (rect, opacity) in tints {
                    if let Some(r) = krect(rect) {
                        let mut pb = PathBuilder::new();
                        pb.push_rect(r);
                        if let Some(path) = pb.finish() {
                            surface.set_fill(Some(fill_of(
                                &Colour("#14110f".into()),
                                *opacity as f32,
                            )));
                            surface.set_stroke(None);
                            surface.draw_path(&path);
                        }
                    }
                }
                for rule in rules {
                    self.rule(surface, rule);
                }
            }
            SceneKind::Text {
                at,
                size,
                rotation,
                lines,
            } => {
                let turned = *rotation != 0.0;
                if turned {
                    surface.push_transform(&Transform::from_rotate_at(
                        *rotation as f32,
                        (at.0 + size.0 / 2.0) as f32,
                        (at.1 + size.1 / 2.0) as f32,
                    ));
                }
                for line in lines {
                    for run in &line.runs {
                        self.run(surface, at.0 + line.x, at.1 + line.baseline, run);
                    }
                }
                if turned {
                    surface.pop();
                }
            }
        }
        surface.pop();
    }

    fn rule(&mut self, surface: &mut Surface, rule: &SceneLine) {
        let mut pb = PathBuilder::new();
        pb.move_to(rule.x1 as f32, rule.y1 as f32);
        pb.line_to(rule.x2 as f32, rule.y2 as f32);
        if let Some(path) = pb.finish() {
            surface.set_fill(None);
            surface.set_stroke(Some(Stroke {
                paint: colour(&Colour("#14110f".into())).into(),
                width: rule.width as f32,
                opacity: NormalizedF32::new(0.7).unwrap_or(NormalizedF32::ONE),
                ..Stroke::default()
            }));
            surface.draw_path(&path);
        }
    }

    fn background(
        &mut self,
        surface: &mut Surface,
        bg: &Background,
        across: Option<&SceneRect>,
        area: &SceneRect,
    ) {
        let whole = *area;
        match bg {
            Background::Colour { colour: c } => self.fill_rect(surface, &whole, fill_of(c, 1.0)),
            Background::Gradient { angle, stops } => {
                let r = across.copied().unwrap_or(whole);
                let radians = angle.to_radians();
                // CSS: 0 degrees runs bottom to top, 90 left to right.
                let (dx, dy) = (radians.sin(), -radians.cos());
                let (cx, cy) = (r.x + r.w / 2.0, r.y + r.h / 2.0);
                let half = (r.w * dx.abs() + r.h * dy.abs()) / 2.0;
                let paint = LinearGradient {
                    x1: (cx - dx * half) as f32,
                    y1: (cy - dy * half) as f32,
                    x2: (cx + dx * half) as f32,
                    y2: (cy + dy * half) as f32,
                    transform: Transform::default(),
                    spread_method: SpreadMethod::Pad,
                    stops: stops
                        .iter()
                        .map(|s| Stop {
                            offset: NormalizedF32::new(s.at as f32).unwrap_or(NormalizedF32::ZERO),
                            color: colour(&s.colour).into(),
                            opacity: NormalizedF32::ONE,
                        })
                        .collect(),
                    anti_alias: false,
                };
                self.fill_rect(
                    surface,
                    &whole,
                    Fill {
                        paint: paint.into(),
                        opacity: NormalizedF32::ONE,
                        rule: FillRule::default(),
                    },
                );
            }
            // A texture is drawn as its tint, flat, until the pattern tiles are shared with the page.
            Background::Texture { tint, .. } => {
                let c = tint.clone().unwrap_or_else(|| Colour("#ebe5d6".into()));
                self.fill_rect(surface, &whole, fill_of(&c, 1.0));
            }
            Background::Picture { asset, fit, focus } => {
                let r = across.copied().unwrap_or(whole);
                let fit = match fit {
                    folio_model::BgFit::Fill => Fit::Fill,
                    folio_model::BgFit::Fit => Fit::Fit,
                };
                self.picture(surface, &r, 0.0, Some(asset), fit, *focus);
            }
        }
    }

    fn fill_rect(&mut self, surface: &mut Surface, rect: &SceneRect, fill: Fill) {
        if let Some(r) = krect(rect) {
            let mut pb = PathBuilder::new();
            pb.push_rect(r);
            if let Some(path) = pb.finish() {
                surface.set_fill(Some(fill));
                surface.set_stroke(None);
                surface.draw_path(&path);
            }
        }
    }

    fn picture(
        &mut self,
        surface: &mut Surface,
        rect: &SceneRect,
        rotation: f64,
        asset: Option<&str>,
        fit: Fit,
        focus: (f64, f64),
    ) {
        let turned = rotation != 0.0;
        if turned {
            surface.push_transform(&Transform::from_rotate_at(
                rotation as f32,
                (rect.x + rect.w / 2.0) as f32,
                (rect.y + rect.h / 2.0) as f32,
            ));
        }
        let image = asset.and_then(|path| self.image(path));
        match image {
            None => {
                // A frame still waiting for its picture, or one that cannot be found: a quiet box.
                self.fill_rect(surface, rect, fill_of(&Colour("#e0ddd4".into()), 0.6));
            }
            Some(image) => {
                let (nw, nh) = image.size();
                let at = focused_rect(*rect, (f64::from(nw), f64::from(nh)), fit, focus);
                let (x, y, w, h) = (at.x, at.y, at.w, at.h);
                if let Some(r) = krect(rect) {
                    let mut pb = PathBuilder::new();
                    pb.push_rect(r);
                    if let Some(path) = pb.finish() {
                        surface.push_clip_path(&path, &FillRule::default());
                        surface.push_transform(&Transform::from_translate(x as f32, y as f32));
                        if let Some(size) = Size::from_wh(w as f32, h as f32) {
                            surface.draw_image(image, size);
                        }
                        surface.pop();
                        surface.pop();
                    }
                }
            }
        }
        if turned {
            surface.pop();
        }
    }

    fn image(&mut self, path: &str) -> Option<Image> {
        if let Some(found) = self.images.get(path) {
            return found.clone();
        }
        let made = self.assets.bytes(path).and_then(|bytes| {
            let lower = path.to_ascii_lowercase();
            let data: Data = Arc::new(bytes).into();
            if lower.ends_with(".png") {
                Image::from_png(data, true).ok()
            } else if lower.ends_with(".jpg") || lower.ends_with(".jpeg") {
                Image::from_jpeg(data, true).ok()
            } else if lower.ends_with(".gif") {
                Image::from_gif(data, true).ok()
            } else if lower.ends_with(".webp") {
                Image::from_webp(data, true).ok()
            } else {
                None
            }
        });
        self.images.insert(path.to_string(), made.clone());
        made
    }

    fn shape(&mut self, surface: &mut Surface, kind: &SceneKind) {
        let SceneKind::Shape {
            rect,
            rotation,
            shape,
            fill,
            stroke,
            stroke_width,
            radius,
            flip,
            paths,
        } = kind
        else {
            return;
        };
        let turned = *rotation != 0.0;
        if turned {
            surface.push_transform(&Transform::from_rotate_at(
                *rotation as f32,
                (rect.x + rect.w / 2.0) as f32,
                (rect.y + rect.h / 2.0) as f32,
            ));
        }
        let fill_paint = fill.as_ref().map(|c| fill_of(c, 1.0));
        let stroke_paint = stroke.as_ref().map(|c| Stroke {
            paint: colour(c).into(),
            width: stroke_width.unwrap_or(1.0) as f32,
            ..Stroke::default()
        });
        match shape {
            Shape::Line => {
                let mut pb = PathBuilder::new();
                let (y1, y2) = if *flip {
                    (rect.y + rect.h, rect.y)
                } else {
                    (rect.y, rect.y + rect.h)
                };
                pb.move_to(rect.x as f32, y1 as f32);
                pb.line_to((rect.x + rect.w) as f32, y2 as f32);
                if let Some(path) = pb.finish() {
                    let ink = stroke
                        .clone()
                        .or_else(|| fill.clone())
                        .unwrap_or_else(|| Colour("#14110f".into()));
                    surface.set_fill(None);
                    surface.set_stroke(Some(Stroke {
                        paint: colour(&ink).into(),
                        width: stroke_width.unwrap_or(2.0) as f32,
                        ..Stroke::default()
                    }));
                    surface.draw_path(&path);
                }
            }
            Shape::Ellipse => {
                let mut pb = PathBuilder::new();
                let (cx, cy) = (
                    (rect.x + rect.w / 2.0) as f32,
                    (rect.y + rect.h / 2.0) as f32,
                );
                let (rx, ry) = ((rect.w / 2.0) as f32, (rect.h / 2.0) as f32);
                let (kx, ky) = (rx * KAPPA, ry * KAPPA);
                pb.move_to(cx + rx, cy);
                pb.cubic_to(cx + rx, cy + ky, cx + kx, cy + ry, cx, cy + ry);
                pb.cubic_to(cx - kx, cy + ry, cx - rx, cy + ky, cx - rx, cy);
                pb.cubic_to(cx - rx, cy - ky, cx - kx, cy - ry, cx, cy - ry);
                pb.cubic_to(cx + kx, cy - ry, cx + rx, cy - ky, cx + rx, cy);
                pb.close();
                self.paint_path(surface, pb, fill_paint, stroke_paint);
            }
            Shape::Rect | Shape::Round => {
                let mut pb = PathBuilder::new();
                let corner = if *shape == Shape::Round {
                    radius.unwrap_or(rect.w.min(rect.h) * 0.12)
                } else {
                    0.0
                };
                rounded_rect(&mut pb, rect, corner);
                self.paint_path(surface, pb, fill_paint, stroke_paint);
            }
            Shape::Speech | Shape::Thought | Shape::Shout => {
                for d in paths {
                    let mut pb = PathBuilder::new();
                    if let Ok(bez) = kurbo::BezPath::from_svg(d) {
                        for el in bez.elements() {
                            let at = |p: &kurbo::Point| {
                                (
                                    (rect.x + p.x * rect.w) as f32,
                                    (rect.y + p.y * rect.h) as f32,
                                )
                            };
                            match el {
                                kurbo::PathEl::MoveTo(p) => {
                                    let (x, y) = at(p);
                                    pb.move_to(x, y);
                                }
                                kurbo::PathEl::LineTo(p) => {
                                    let (x, y) = at(p);
                                    pb.line_to(x, y);
                                }
                                kurbo::PathEl::QuadTo(c, p) => {
                                    let ((cx, cy), (x, y)) = (at(c), at(p));
                                    pb.quad_to(cx, cy, x, y);
                                }
                                kurbo::PathEl::CurveTo(a, b, p) => {
                                    let ((ax, ay), (bx, by), (x, y)) = (at(a), at(b), at(p));
                                    pb.cubic_to(ax, ay, bx, by, x, y);
                                }
                                kurbo::PathEl::ClosePath => pb.close(),
                            }
                        }
                    }
                    self.paint_path(surface, pb, fill_paint.clone(), stroke_paint.clone());
                }
            }
        }
        if turned {
            surface.pop();
        }
    }

    fn paint_path(
        &mut self,
        surface: &mut Surface,
        pb: PathBuilder,
        fill: Option<Fill>,
        stroke: Option<Stroke>,
    ) {
        if let Some(path) = pb.finish() {
            surface.set_fill(fill);
            surface.set_stroke(stroke);
            surface.draw_path(&path);
        }
    }

    fn run(&mut self, surface: &mut Surface, line_x: f64, baseline: f64, run: &LayoutRun) {
        if run.glyphs.is_empty() || run.text.trim().is_empty() && !run.hyphen {
            return;
        }
        let Some(font) = self.font(&run.face_key, run.weight, run.size) else {
            return;
        };
        let size = run.size as f32;
        // Where each glyph's text is, as the cluster to the next cluster's start.
        let starts: Vec<usize> = run.glyphs.iter().map(|g| g.cluster as usize).collect();
        let glyphs: Vec<KrillaGlyph> = run
            .glyphs
            .iter()
            .enumerate()
            .map(|(i, g)| {
                let from = g.cluster as usize;
                let to = starts
                    .iter()
                    .skip(i + 1)
                    .copied()
                    .find(|s| *s != from)
                    .unwrap_or(run.text.len())
                    .max(from);
                KrillaGlyph::new(
                    GlyphId::new(u32::from(g.id)),
                    (g.advance / run.size) as f32,
                    (g.dx / run.size) as f32,
                    (-g.dy / run.size) as f32,
                    0.0,
                    from.min(run.text.len())..to.min(run.text.len()),
                    None,
                )
            })
            .collect();
        let ink = run
            .colour
            .clone()
            .unwrap_or_else(|| Colour(INK.to_string()));
        surface.set_fill(Some(fill_of(&ink, 1.0)));
        surface.set_stroke(None);
        surface.draw_glyphs(
            Point::from_xy((line_x + run.x) as f32, baseline as f32),
            &glyphs,
            font,
            &run.text,
            size,
            false,
        );
    }

    fn font(&mut self, key: &str, weight: u16, size_pt: f64) -> Option<Font> {
        // Optical size changes with the size, but whole points are enough to tell instances apart.
        let id = (key.to_string(), weight, size_pt.round() as u32);
        if let Some(found) = self.faces.get(&id) {
            return found.clone();
        }
        let made = self.fonts.font_by_key(key).and_then(|f| {
            let data: Data =
                (Arc::new(Arc::clone(f.data())) as Arc<dyn AsRef<[u8]> + Send + Sync>).into();
            let axes: Vec<(Tag, f32)> = f
                .variations_for(size_pt, weight)
                .into_iter()
                .map(|(tag, value)| (Tag::new(&tag), value))
                .collect();
            if axes.is_empty() {
                Font::new(data, f.index())
            } else {
                Font::new_variable(data, f.index(), &axes)
            }
        });
        self.faces.insert(id, made.clone());
        made
    }
}

fn rounded_rect(pb: &mut PathBuilder, r: &SceneRect, corner: f64) {
    let (x, y, w, h) = (r.x as f32, r.y as f32, r.w as f32, r.h as f32);
    let c = (corner as f32).clamp(0.0, w.min(h) / 2.0);
    if c <= 0.0 {
        if let Some(rect) = krect(r) {
            pb.push_rect(rect);
        }
        return;
    }
    // A quarter circle is a cubic with this share of the radius as its handles.
    let k = c * KAPPA;
    pb.move_to(x + c, y);
    pb.line_to(x + w - c, y);
    pb.cubic_to(x + w - c + k, y, x + w, y + c - k, x + w, y + c);
    pb.line_to(x + w, y + h - c);
    pb.cubic_to(x + w, y + h - c + k, x + w - c + k, y + h, x + w - c, y + h);
    pb.line_to(x + c, y + h);
    pb.cubic_to(x + c - k, y + h, x, y + h - c + k, x, y + h - c);
    pb.line_to(x, y + c);
    pb.cubic_to(x, y + c - k, x + c - k, y, x + c, y);
    pb.close();
}
