// Drawing a page of the scene as pixels, for the pictures a store or a screen wants
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use folio_layout::scene::{focused_rect, PageScene, SceneItem, SceneKind, SceneRect};
use folio_layout::wire::LayoutRun;
use folio_layout::FontBook;
use folio_model::{Background, Colour, Fit, Shape};
use tiny_skia::{
    Color, FillRule, GradientStop, LinearGradient, Mask, Paint, PathBuilder, Pixmap, PixmapPaint,
    Point, SpreadMode, Transform,
};

use crate::error::ExportError;
use crate::pdf::Assets;

/// A JPEG of `page` at `width` × `height` pixels. The page is scaled to fit and centred, and the
/// room left over is filled with the page's background colour, so a cover of another shape still
/// fills the picture a store asks for.
pub fn page_to_jpeg(
    page: &PageScene,
    fonts: &FontBook,
    assets: &dyn Assets,
    width: u32,
    height: u32,
) -> Result<Vec<u8>, ExportError> {
    let mut pixmap = render_page(page, fonts, assets, width, height)?;
    border_if_light(&mut pixmap);
    // JPEG has no transparency, and the page is opaque, so the colour channels are what is wanted.
    let mut rgb = Vec::with_capacity((width * height * 3) as usize);
    for pixel in pixmap.pixels() {
        let c = pixel.demultiply();
        rgb.extend_from_slice(&[c.red(), c.green(), c.blue()]);
    }
    let mut out = Vec::new();
    image::codecs::jpeg::JpegEncoder::new_with_quality(&mut out, 90)
        .encode(&rgb, width, height, image::ExtendedColorType::Rgb8)
        .map_err(|e| ExportError::Pdf(e.to_string()))?;
    Ok(out)
}

/// Stores show a cover on a white page, so a light one gets a thin mid-grey border (3 to 4 pixels
/// at the size they ask for) that keeps its edge visible.
fn border_if_light(pixmap: &mut Pixmap) {
    let (w, h) = (pixmap.width(), pixmap.height());
    let luma = |x: u32, y: u32| {
        let c = pixmap.pixel(x, y).map_or(0.0, |p| {
            let c = p.demultiply();
            0.2126 * f32::from(c.red())
                + 0.7152 * f32::from(c.green())
                + 0.0722 * f32::from(c.blue())
        });
        c / 255.0
    };
    let edge = (luma(2, 2) + luma(w - 3, 2) + luma(2, h - 3) + luma(w - 3, h - 3)) / 4.0;
    if edge < 0.85 {
        return;
    }
    let width = (w / 400).clamp(3, 4);
    let mut paint = Paint::default();
    paint.set_color(Color::from_rgba8(0x88, 0x88, 0x88, 255));
    for rect in [
        (0, 0, w, width),
        (0, h - width, w, width),
        (0, 0, width, h),
        (w - width, 0, width, h),
    ] {
        if let Some(r) =
            tiny_skia::Rect::from_xywh(rect.0 as f32, rect.1 as f32, rect.2 as f32, rect.3 as f32)
        {
            pixmap.fill_rect(r, &paint, Transform::identity(), None);
        }
    }
}

/// `page` drawn into a pixmap of `width` × `height`.
pub fn render_page(
    page: &PageScene,
    fonts: &FontBook,
    assets: &dyn Assets,
    width: u32,
    height: u32,
) -> Result<Pixmap, ExportError> {
    let mut pixmap = Pixmap::new(width, height)
        .ok_or_else(|| ExportError::Pdf("the picture has no size".into()))?;
    let scale = (f64::from(width) / page.width).min(f64::from(height) / page.height);
    let (shown_w, shown_h) = (page.width * scale, page.height * scale);
    let (left, top) = (
        (f64::from(width) - shown_w) / 2.0,
        (f64::from(height) - shown_h) / 2.0,
    );

    // The page's own colour first, over everything, so the room round it is the same colour.
    let ground = page
        .items
        .iter()
        .find_map(|item| match &item.kind {
            SceneKind::Background {
                bg: Background::Colour { colour },
                ..
            } => Some(colour.clone()),
            _ => None,
        })
        .unwrap_or_else(|| Colour("#ffffff".into()));
    pixmap.fill(colour(&ground));

    let to_page =
        Transform::from_translate(left as f32, top as f32).pre_scale(scale as f32, scale as f32);
    let mut draw = Raster {
        pixmap: &mut pixmap,
        to_page,
        fonts,
        assets,
        page,
        scale,
        left,
        top,
    };
    for item in &page.items {
        draw.item(item);
    }
    Ok(pixmap)
}

fn colour(c: &Colour) -> Color {
    let (r, g, b) = folio_colour::parse_hex(&c.0).unwrap_or((0, 0, 0));
    Color::from_rgba8(r, g, b, 255)
}

struct Raster<'a> {
    pixmap: &'a mut Pixmap,
    to_page: Transform,
    fonts: &'a FontBook,
    assets: &'a dyn Assets,
    page: &'a PageScene,
    scale: f64,
    left: f64,
    top: f64,
}

fn rect_path(r: &SceneRect) -> Option<tiny_skia::Path> {
    let rect = tiny_skia::Rect::from_xywh(
        r.x as f32,
        r.y as f32,
        r.w.max(0.0) as f32,
        r.h.max(0.0) as f32,
    )?;
    Some(PathBuilder::from_rect(rect))
}

impl Raster<'_> {
    fn paint(&self, c: &Colour, opacity: f32) -> Paint<'static> {
        let mut paint = Paint::default();
        let mut color = colour(c);
        color.set_alpha(opacity);
        paint.set_color(color);
        paint.anti_alias = true;
        paint
    }

    /// The transform of a frame turned about its middle, on the page.
    fn turned(&self, rect: &SceneRect, degrees: f64) -> Transform {
        self.to_page.pre_rotate_at(
            degrees as f32,
            (rect.x + rect.w / 2.0) as f32,
            (rect.y + rect.h / 2.0) as f32,
        )
    }

    fn item(&mut self, item: &SceneItem) {
        match &item.kind {
            SceneKind::Background { bg, .. } => self.background(bg),
            SceneKind::Picture {
                rect,
                rotation,
                asset,
                fit,
                focus,
                ..
            } => self.picture(rect, *rotation, asset.as_deref(), *fit, *focus),
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
            } => self.shape(
                rect,
                *rotation,
                *shape,
                fill,
                stroke,
                *stroke_width,
                *radius,
                *flip,
                paths,
            ),
            SceneKind::Table { tints, rules, .. } => {
                for (rect, opacity) in tints {
                    if let Some(path) = rect_path(rect) {
                        let paint = self.paint(&Colour("#14110f".into()), *opacity as f32);
                        self.pixmap
                            .fill_path(&path, &paint, FillRule::Winding, self.to_page, None);
                    }
                }
                for rule in rules {
                    let mut pb = PathBuilder::new();
                    pb.move_to(rule.x1 as f32, rule.y1 as f32);
                    pb.line_to(rule.x2 as f32, rule.y2 as f32);
                    if let Some(path) = pb.finish() {
                        let stroke = tiny_skia::Stroke {
                            width: rule.width as f32,
                            ..tiny_skia::Stroke::default()
                        };
                        let paint = self.paint(&Colour("#14110f".into()), 0.7);
                        self.pixmap
                            .stroke_path(&path, &paint, &stroke, self.to_page, None);
                    }
                }
            }
            SceneKind::Text {
                at,
                size,
                rotation,
                lines,
            } => {
                let frame = SceneRect {
                    x: at.0,
                    y: at.1,
                    w: size.0,
                    h: size.1,
                };
                let transform = self.turned(&frame, *rotation);
                for line in lines {
                    for run in &line.runs {
                        self.run(transform, at.0 + line.x, at.1 + line.baseline, run);
                    }
                }
            }
        }
    }

    fn background(&mut self, bg: &Background) {
        let whole = SceneRect {
            x: -self.left / self.scale,
            y: -self.top / self.scale,
            w: f64::from(self.pixmap.width()) / self.scale,
            h: f64::from(self.pixmap.height()) / self.scale,
        };
        match bg {
            Background::Colour { colour: c } => self.fill(&whole, c),
            Background::Texture { id, tint } => {
                let tile = crate::texture::tile_png(id, tint.as_ref().map(|c| c.0.as_str()))
                    .and_then(|(png, size)| {
                        let img = image::load_from_memory(&png).ok()?.to_rgba8();
                        let (w, h) = (img.width(), img.height());
                        let mut bytes = img.into_raw();
                        for px in bytes.as_chunks_mut::<4>().0 {
                            let a = u16::from(px[3]);
                            for c in &mut px[..3] {
                                *c = ((u16::from(*c) * a + 127) / 255) as u8;
                            }
                        }
                        let pixmap = Pixmap::from_vec(bytes, tiny_skia::IntSize::from_wh(w, h)?)?;
                        Some((pixmap, size, w))
                    });
                match (tile, rect_path(&whole)) {
                    (Some((pixmap, size, px)), Some(path)) => {
                        let scale = (size / f64::from(px)) as f32;
                        let paint = Paint {
                            shader: tiny_skia::Pattern::new(
                                pixmap.as_ref(),
                                SpreadMode::Repeat,
                                tiny_skia::FilterQuality::Bilinear,
                                1.0,
                                Transform::from_scale(scale, scale),
                            ),
                            anti_alias: false,
                            ..Paint::default()
                        };
                        self.pixmap
                            .fill_path(&path, &paint, FillRule::Winding, self.to_page, None);
                    }
                    _ => {
                        let c = tint.clone().unwrap_or_else(|| Colour("#ebe5d6".into()));
                        self.fill(&whole, &c);
                    }
                }
            }
            Background::Gradient { angle, stops } => {
                let page = SceneRect {
                    x: 0.0,
                    y: 0.0,
                    w: self.page.width,
                    h: self.page.height,
                };
                let radians = angle.to_radians();
                let (dx, dy) = (radians.sin(), -radians.cos());
                let (cx, cy) = (page.w / 2.0, page.h / 2.0);
                let half = (page.w * dx.abs() + page.h * dy.abs()) / 2.0;
                let points: Vec<GradientStop> = stops
                    .iter()
                    .map(|s| GradientStop::new(s.at as f32, colour(&s.colour)))
                    .collect();
                if let (Some(shader), Some(path)) = (
                    LinearGradient::new(
                        Point::from_xy((cx - dx * half) as f32, (cy - dy * half) as f32),
                        Point::from_xy((cx + dx * half) as f32, (cy + dy * half) as f32),
                        points,
                        SpreadMode::Pad,
                        Transform::identity(),
                    ),
                    rect_path(&whole),
                ) {
                    let paint = Paint {
                        shader,
                        ..Paint::default()
                    };
                    self.pixmap
                        .fill_path(&path, &paint, FillRule::Winding, self.to_page, None);
                }
            }
            Background::Picture { asset, fit, focus } => {
                let fit = match fit {
                    folio_model::BgFit::Fill => Fit::Fill,
                    folio_model::BgFit::Fit => Fit::Fit,
                };
                let page = SceneRect {
                    x: 0.0,
                    y: 0.0,
                    w: self.page.width,
                    h: self.page.height,
                };
                self.picture(&page, 0.0, Some(asset), fit, *focus);
            }
        }
    }

    fn fill(&mut self, rect: &SceneRect, c: &Colour) {
        if let Some(path) = rect_path(rect) {
            let paint = self.paint(c, 1.0);
            self.pixmap
                .fill_path(&path, &paint, FillRule::Winding, self.to_page, None);
        }
    }

    fn picture(
        &mut self,
        rect: &SceneRect,
        rotation: f64,
        asset: Option<&str>,
        fit: Fit,
        focus: (f64, f64),
    ) {
        let transform = self.turned(rect, rotation);
        let decoded = asset
            .and_then(|path| self.assets.bytes(path))
            .and_then(|bytes| image::load_from_memory(&bytes).ok())
            .map(|img| img.to_rgba8());
        let Some(image) = decoded else {
            let paint = self.paint(&Colour("#e0ddd4".into()), 0.6);
            if let Some(path) = rect_path(rect) {
                self.pixmap
                    .fill_path(&path, &paint, FillRule::Winding, transform, None);
            }
            return;
        };
        let (iw, ih) = (image.width(), image.height());
        let mut bytes = image.into_raw();
        // tiny-skia holds premultiplied colour.
        for px in bytes.as_chunks_mut::<4>().0 {
            let a = u16::from(px[3]);
            for c in &mut px[..3] {
                *c = ((u16::from(*c) * a + 127) / 255) as u8;
            }
        }
        let Some(source) =
            tiny_skia::IntSize::from_wh(iw, ih).and_then(|size| Pixmap::from_vec(bytes, size))
        else {
            return;
        };
        let at = focused_rect(*rect, (f64::from(iw), f64::from(ih)), fit, focus);
        // The picture is cut to its frame.
        let mut mask = Mask::new(self.pixmap.width(), self.pixmap.height()).expect("a mask");
        if let Some(path) = rect_path(rect) {
            mask.fill_path(&path, FillRule::Winding, true, transform);
        }
        let place = transform
            .pre_translate(at.x as f32, at.y as f32)
            .pre_scale((at.w / f64::from(iw)) as f32, (at.h / f64::from(ih)) as f32);
        let paint = PixmapPaint {
            quality: tiny_skia::FilterQuality::Bicubic,
            ..PixmapPaint::default()
        };
        self.pixmap
            .draw_pixmap(0, 0, source.as_ref(), &paint, place, Some(&mask));
    }

    #[allow(clippy::too_many_arguments)]
    fn shape(
        &mut self,
        rect: &SceneRect,
        rotation: f64,
        shape: Shape,
        fill: &Option<Colour>,
        stroke: &Option<Colour>,
        stroke_width: Option<f64>,
        radius: Option<f64>,
        flip: bool,
        paths: &[String],
    ) {
        let transform = self.turned(rect, rotation);
        let mut outline: Vec<(tiny_skia::Path, Transform)> = Vec::new();
        let mut pb = PathBuilder::new();
        match shape {
            Shape::Line => {
                let (y1, y2) = if flip {
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
                    let line = tiny_skia::Stroke {
                        width: stroke_width.unwrap_or(2.0) as f32,
                        ..tiny_skia::Stroke::default()
                    };
                    self.pixmap
                        .stroke_path(&path, &self.paint(&ink, 1.0), &line, transform, None);
                }
                return;
            }
            Shape::Ellipse => {
                if let Some(r) = tiny_skia::Rect::from_xywh(
                    rect.x as f32,
                    rect.y as f32,
                    rect.w as f32,
                    rect.h as f32,
                ) {
                    pb.push_oval(r);
                }
            }
            Shape::Rect | Shape::Round => {
                let corner = if shape == Shape::Round {
                    radius.unwrap_or(rect.w.min(rect.h) * 0.12) as f32
                } else {
                    0.0
                };
                let (x, y, w, h) = (rect.x as f32, rect.y as f32, rect.w as f32, rect.h as f32);
                let c = corner.clamp(0.0, w.min(h) / 2.0);
                if c <= 0.0 {
                    if let Some(r) = tiny_skia::Rect::from_xywh(x, y, w, h) {
                        pb.push_rect(r);
                    }
                } else {
                    let k = c * 0.552_284_8;
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
            }
            Shape::Speech | Shape::Thought | Shape::Shout => {
                for d in paths {
                    let mut p = PathBuilder::new();
                    if let Ok(bez) = kurbo::BezPath::from_svg(d) {
                        for el in bez.elements() {
                            let at = |q: &kurbo::Point| {
                                (
                                    (rect.x + q.x * rect.w) as f32,
                                    (rect.y + q.y * rect.h) as f32,
                                )
                            };
                            match el {
                                kurbo::PathEl::MoveTo(q) => {
                                    let (x, y) = at(q);
                                    p.move_to(x, y);
                                }
                                kurbo::PathEl::LineTo(q) => {
                                    let (x, y) = at(q);
                                    p.line_to(x, y);
                                }
                                kurbo::PathEl::QuadTo(c, q) => {
                                    let ((cx, cy), (x, y)) = (at(c), at(q));
                                    p.quad_to(cx, cy, x, y);
                                }
                                kurbo::PathEl::CurveTo(a, b, q) => {
                                    let ((ax, ay), (bx, by), (x, y)) = (at(a), at(b), at(q));
                                    p.cubic_to(ax, ay, bx, by, x, y);
                                }
                                kurbo::PathEl::ClosePath => p.close(),
                            }
                        }
                    }
                    if let Some(path) = p.finish() {
                        outline.push((path, transform));
                    }
                }
            }
        }
        if let Some(path) = pb.finish() {
            outline.push((path, transform));
        }
        for (path, transform) in outline {
            if let Some(c) = fill {
                self.pixmap.fill_path(
                    &path,
                    &self.paint(c, 1.0),
                    FillRule::Winding,
                    transform,
                    None,
                );
            }
            if let Some(c) = stroke {
                let line = tiny_skia::Stroke {
                    width: stroke_width.unwrap_or(1.0) as f32,
                    ..tiny_skia::Stroke::default()
                };
                self.pixmap
                    .stroke_path(&path, &self.paint(c, 1.0), &line, transform, None);
            }
        }
    }

    /// The glyphs of a run, drawn from the font's outlines at the size and weight they were shaped at.
    fn run(&mut self, transform: Transform, line_x: f64, baseline: f64, run: &LayoutRun) {
        let Some(font) = self.fonts.font_by_key(&run.face_key) else {
            return;
        };
        let Ok(mut face) = ttf_parser::Face::parse(font.data(), font.index()) else {
            return;
        };
        for (tag, value) in font.variations_for(run.size, run.weight) {
            face.set_variation(ttf_parser::Tag::from_bytes(&tag), value);
        }
        let scale = (run.size / f64::from(face.units_per_em())) as f32;
        let ink = run
            .colour
            .clone()
            .unwrap_or_else(|| Colour("#1c1b20".into()));
        let paint = self.paint(&ink, 1.0);
        let mut pen = line_x + run.x;
        for glyph in &run.glyphs {
            let mut outline = Outline(PathBuilder::new());
            if face
                .outline_glyph(ttf_parser::GlyphId(glyph.id), &mut outline)
                .is_some()
            {
                if let Some(path) = outline.0.finish() {
                    // Font units run up, the page runs down; the glyph sits on the baseline.
                    let place = transform
                        .pre_translate((pen + glyph.dx) as f32, (baseline - glyph.dy) as f32)
                        .pre_scale(scale, -scale);
                    self.pixmap
                        .fill_path(&path, &paint, FillRule::Winding, place, None);
                }
            }
            pen += glyph.advance;
        }
    }
}

struct Outline(PathBuilder);

impl ttf_parser::OutlineBuilder for Outline {
    fn move_to(&mut self, x: f32, y: f32) {
        self.0.move_to(x, y);
    }
    fn line_to(&mut self, x: f32, y: f32) {
        self.0.line_to(x, y);
    }
    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        self.0.quad_to(x1, y1, x, y);
    }
    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        self.0.cubic_to(x1, y1, x2, y2, x, y);
    }
    fn close(&mut self) {
        self.0.close();
    }
}
