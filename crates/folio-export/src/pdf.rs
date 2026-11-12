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
use krilla::action::{Action, LinkAction};
use krilla::annotation::{Annotation, LinkAnnotation, Target};
use krilla::color::{cmyk, rgb};
use krilla::destination::XyzDestination;
use krilla::geom::{PathBuilder, Point, Quadrilateral, Rect as KRect, Size, Transform};
use krilla::image::Image;
use krilla::num::NormalizedF32;
use krilla::page::PageSettings;
use krilla::paint::{Fill, FillRule, LinearGradient, SpreadMethod, Stop, Stroke};
use krilla::surface::Surface;
use krilla::text::{Font, GlyphId, KrillaGlyph, Tag};
use krilla::{Data, SerializeSettings};

use folio_colour::Separator;

use crate::error::ExportError;
use crate::outline::{CoverSide, Navigation, Origin};
use crate::pages::{CoverPages, PageChoice};
use crate::picture::{ImagePolicy, PictureCache};
use crate::shop::ShopOptions;

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
    /// To read on a screen or send round: one page per page, with the book's title and author, and
    /// the book's links live: an address opens, a link to a spread goes to its first page.
    #[default]
    Sharing,
    /// Print-ready for Kindle Direct Publishing: single pages, nothing but the title and author in
    /// the file's information, no bookmarks or links.
    Print,
    /// For a print shop: every page, with the bleed all round, the trim marked, optional crop marks
    /// and the file made PDF/X-4 with an output intent.
    PrintShop,
    /// The paperback cover sheet for KDP: one page with the bleed all round, in ink, with nothing
    /// but the title and author in the file.
    Cover,
}

#[derive(Debug, Clone, Default)]
pub struct PdfOptions {
    pub target: PdfTarget,
    /// Write colours as ink rather than sRGB, through this CMYK profile when there is one and by a
    /// plain separation when there is not.
    pub ink: Option<InkOptions>,
    /// Crop marks and room for them, for a print shop.
    pub crop_marks: bool,
    /// Bookmarks, page labels and what the file says about itself.
    pub navigation: Navigation,
    /// Which pages the file has and how they sit on its sheets.
    pub pages: PageChoice,
    /// What only a print shop's file has: its bleed, the room for crop marks and press ink.
    pub shop: ShopOptions,
    /// The program to name in the file's information, for the targets that carry it.
    pub application: Option<crate::outline::Application>,
    /// Blend pictures with see-through parts onto what is behind them, so the file holds none;
    /// `None` is what the target usually does (everything but a Sharing PDF).
    pub flatten: Option<bool>,
    /// Stops the file when a newer request has made it not worth finishing.
    pub cancel: crate::cancel::Cancel,
}

/// Print colour for a file.
#[derive(Debug, Clone, Default)]
pub struct InkOptions {
    /// The ICC bytes of the press's CMYK profile, when the author or Folio has one.
    pub profile: Option<Vec<u8>>,
}

/// Draws every page of `book` into a PDF, with the pictures treated as the target usually has them.
pub fn export_pdf(
    book: &Book,
    styles: &StyleSheet,
    fonts: &FontBook,
    assets: &dyn Assets,
    options: &PdfOptions,
) -> Result<Vec<u8>, ExportError> {
    export_pdf_with(
        book,
        styles,
        fonts,
        assets,
        options,
        &ImagePolicy::for_target(options.target),
        &PictureCache::default(),
    )
}

/// Draws every page of `book` into a PDF, its pictures prepared by `pictures` and kept in `cache`.
pub fn export_pdf_with(
    book: &Book,
    styles: &StyleSheet,
    fonts: &FontBook,
    assets: &dyn Assets,
    options: &PdfOptions,
    pictures: &ImagePolicy,
    cache: &PictureCache,
) -> Result<Vec<u8>, ExportError> {
    export_pdf_noted(book, styles, fonts, assets, options, pictures, cache).map(|(bytes, _)| bytes)
}

/// Draws every page of `book` into a PDF as `export_pdf_with` does, and says what it left as it was
/// where the file was asked to be flattened: the notes, one line each, for the person to read.
pub fn export_pdf_noted(
    book: &Book,
    styles: &StyleSheet,
    fonts: &FontBook,
    assets: &dyn Assets,
    options: &PdfOptions,
    pictures: &ImagePolicy,
    cache: &PictureCache,
) -> Result<(Vec<u8>, Vec<String>), ExportError> {
    let choice = options.pages.for_target(options.target);
    // A print PDF is the interior only: the cover is made for the cover file.
    let skip = usize::from(
        (options.target == PdfTarget::Print || choice.cover == CoverPages::None)
            && book
                .spreads
                .first()
                .is_some_and(|s| s.kind == folio_model::SpreadKind::Cover),
    );
    let has_cover = book
        .spreads
        .first()
        .is_some_and(|s| s.kind == folio_model::SpreadKind::Cover);
    // The back cover, as a page of its own at the end of the file.
    let back = (has_cover && choice.cover == CoverPages::FrontAndBackEnds)
        .then(|| crate::cover::back_cover_view(book));
    // Only the cover file wants the whole cover: everything else wants the front, as a page.
    let front;
    let book = if options.target == PdfTarget::Cover || choice.cover == CoverPages::FrontAndBack {
        book
    } else {
        front = folio_model::front_cover_view(book);
        &front
    };
    let layouter = Layouter::new(book, styles, fonts);
    let settings = SerializeSettings {
        compress_content_streams: true,
        xmp_metadata: options.target == PdfTarget::Sharing,
        ..SerializeSettings::default()
    };
    let mut document = krilla::Document::new_with(settings);
    // A print shop's file is in ink only when it has a press profile to name the output condition.
    let ink = match options.target {
        PdfTarget::PrintShop => options
            .shop
            .ink
            .as_ref()
            .filter(|ink| ink.profile.is_some()),
        _ => options.ink.as_ref(),
    };
    let separator = match ink {
        None => None,
        Some(ink) => Some(match &ink.profile {
            Some(bytes) => {
                let profile = folio_colour::Profile::from_icc(bytes)
                    .map_err(|e| ExportError::Pdf(e.to_string()))?;
                Separator::through(&profile, folio_colour::Intent::Perceptual)
                    .map_err(|e| ExportError::Pdf(e.to_string()))?
            }
            None => Separator::plain(),
        }),
    };
    let mut drawing = Drawing::new(fonts, assets, separator, book.styles.swatches.clone());
    drawing.policy = *pictures;
    drawing.cache = cache.clone();
    drawing.cancel = options.cancel.clone();
    drawing.flatten = options
        .flatten
        .unwrap_or(options.target != PdfTarget::Sharing);
    drawing.ink_key = match ink.and_then(|ink| ink.profile.as_ref()) {
        Some(bytes) => {
            use std::hash::{Hash, Hasher};
            let mut h = std::collections::hash_map::DefaultHasher::new();
            bytes.hash(&mut h);
            h.finish() | 2
        }
        None => 1,
    };

    let mut scenes = Vec::new();
    // The page each spread of the book begins on (the spreads left out have none), for the links
    // that go to a spread.
    let mut first_page: Vec<Option<usize>> = Vec::new();
    let mut origins: Vec<Origin> = Vec::new();
    // Whether the first page of the file is the front cover alone, a page of the trim size.
    let front_alone = has_cover && skip == 0 && choice.cover != CoverPages::FrontAndBack;
    let labels = folio_model::page_labels(&book.spreads);
    for (at, spread) in book.spreads.iter().enumerate() {
        if at < skip {
            first_page.push(None);
            continue;
        }
        first_page.push(Some(scenes.len()));
        if let Some(scene) = layouter.scene_spread(&spread.id)? {
            for page in &scene.pages {
                let label = labels.get(at).and_then(|l| l.get(page.index));
                origins.push(Origin {
                    spread: spread.id.0.clone(),
                    index: page.index,
                    label: label.map(|l| l.label.clone()).unwrap_or_default(),
                    section: label.map(|l| l.section.clone()).unwrap_or_default(),
                    cover: (at == 0 && front_alone).then_some(CoverSide::Front),
                });
            }
            scenes.extend(scene.pages);
        }
    }
    let navigation = options.navigation.for_target(options.target);
    // The range counts the pages of the interior; the cover pages are added outside it.
    let lead = usize::from(has_cover && skip == 0);
    let interior = crate::pages::kept(scenes.len() - lead.min(scenes.len()), choice.range);
    let kept = lead + interior.start..lead + interior.end;
    let new_index: Vec<Option<usize>> = (0..scenes.len())
        .map(|p| {
            if p < lead {
                Some(p)
            } else {
                kept.contains(&p).then(|| p - kept.start + lead)
            }
        })
        .collect();
    let mut keep: Vec<usize> = (0..lead.min(scenes.len())).chain(kept).collect();
    keep.dedup();
    if keep.len() != scenes.len() {
        origins = keep.iter().map(|&i| origins[i].clone()).collect();
        scenes = keep.iter().map(|&i| scenes[i].clone()).collect();
    }
    if let Some(back) = &back {
        if let Some(scene) = Layouter::new(back, styles, fonts)
            .scene_spread(&back.spreads[0].id)?
            .and_then(|s| s.pages.into_iter().next())
        {
            origins.push(Origin {
                spread: format!("{}#back", back.spreads[0].id.0),
                index: 0,
                label: String::new(),
                section: String::new(),
                cover: Some(CoverSide::Back),
            });
            scenes.push(scene);
        }
    }
    // A link to a spread that falls outside the kept pages goes nowhere.
    for page in first_page.iter_mut() {
        *page = page.and_then(|p| new_index.get(p).copied().flatten());
    }
    let sheets = crate::pages::sheets(&origins, choice.sheets);
    // A link goes to the sheet that holds the page its spread begins on.
    for page in first_page.iter_mut() {
        *page = page.and_then(|p| sheets.iter().position(|s| s.contains(&p)));
    }
    // A sheet is labelled and bookmarked as the first page on it.
    let sheet_origins: Vec<Origin> = sheets.iter().map(|s| origins[s[0]].clone()).collect();
    if scenes.is_empty() {
        return Err(ExportError::NoPages);
    }
    let print = options.target == PdfTarget::Print;
    let shop = options.target == PdfTarget::PrintShop;
    let sheet = options.target == PdfTarget::Cover;
    // Bleed is for a print file. A print shop wants it all round whenever the book has one; KDP
    // wants it only when something bleeds, and then on the outer edge of each page.
    let bleed = if shop {
        options.shop.bleed_in_points(book.bleed.0)
    } else if (print && scenes.iter().any(spills)) || sheet {
        book.bleed.0.max(0.0)
    } else {
        0.0
    };
    // Room outside the bleed for crop marks.
    drawing.demand = crate::picture::demand(&scenes, assets, bleed);
    let slug = if shop && options.crop_marks {
        options.shop.slug_in_points()
    } else {
        0.0
    };
    // A printed book has an even number of pages: a blank one closes an odd count.
    let blank_at_end = print && scenes.len() % 2 == 1;
    let total = sheets.len() + usize::from(blank_at_end);
    for number in 0..total {
        if options.cancel.is_set() {
            return Err(ExportError::Cancelled);
        }
        let members: Vec<&folio_layout::scene::PageScene> = sheets
            .get(number)
            .map(|s| s.iter().filter_map(|&i| scenes.get(i)).collect())
            .unwrap_or_default();
        let page = members.first().copied();
        let (width, height) = match page {
            None => (book.trim.w.0, book.trim.h.0),
            Some(_) => (
                members.iter().map(|p| p.width).sum(),
                members.iter().map(|p| p.height).fold(0.0, f64::max),
            ),
        };
        let recto = page.map_or(number % 2 == 0, |p| p.recto);
        // KDP's bleed is on the outer edge, away from the fold; a print shop's is on every side.
        let (left, right) = match (shop || sheet, recto) {
            (true, _) => (bleed, bleed),
            (false, true) => (0.0, bleed),
            (false, false) => (bleed, 0.0),
        };
        let area = SceneRect {
            x: -left,
            y: -bleed,
            w: width + left + right,
            h: height + 2.0 * bleed,
        };
        let (media_w, media_h) = (area.w + 2.0 * slug, area.h + 2.0 * slug);
        let size = Size::from_wh(media_w as f32, media_h as f32)
            .ok_or_else(|| ExportError::Pdf("a page has no size".into()))?;
        let mut settings = PageSettings::new(size);
        if navigation.page_labels {
            if let Some(label) = sheet_origins
                .get(number)
                .and_then(|o| crate::outline::page_label(o, &navigation.names))
            {
                settings = settings.with_page_label(label);
            }
        }
        let (origin_x, origin_y) = (left + slug, bleed + slug);
        if bleed > 0.0 || shop || sheet {
            settings = settings
                .with_trim_box(KRect::from_xywh(
                    origin_x as f32,
                    origin_y as f32,
                    width as f32,
                    height as f32,
                ))
                .with_bleed_box(KRect::from_xywh(
                    slug as f32,
                    slug as f32,
                    area.w as f32,
                    area.h as f32,
                ));
        }
        let mut pdf_page = document.start_page_with(settings);
        let mut surface = pdf_page.surface();
        let moved = origin_x != 0.0 || origin_y != 0.0;
        if moved {
            surface.push_transform(&Transform::from_translate(origin_x as f32, origin_y as f32));
        }
        // The pages of a sheet sit side by side, each drawn in its own coordinates.
        let mut across = 0.0;
        for page in &members {
            let shifted = across != 0.0;
            if shifted {
                surface.push_transform(&Transform::from_translate(across as f32, 0.0));
            }
            let own = SceneRect {
                w: page.width,
                ..area
            };
            for (at, item) in page.items.iter().enumerate() {
                drawing.plan = drawing
                    .flatten
                    .then(|| {
                        crate::flatten::plan(
                            page,
                            at,
                            &drawing.swatches,
                            drawing.separator.is_some(),
                        )
                    })
                    .flatten();
                drawing.page = number + 1;
                drawing.item(
                    &mut surface,
                    item,
                    if shifted || members.len() > 1 {
                        &own
                    } else {
                        &area
                    },
                );
            }
            if shifted {
                surface.pop();
            }
            across += page.width;
        }
        if slug > 0.0 {
            drawing.crop_marks(&mut surface, width, height, bleed);
        }
        if moved {
            surface.pop();
        }
        surface.finish();
        // Links are for a file that is read on a screen; a file for print carries none.
        if options.target == PdfTarget::Sharing && navigation.links {
            let mut across = 0.0;
            for page in &members {
                for link in link_annotations(
                    page,
                    &first_page,
                    sheets.len(),
                    (origin_x + across, origin_y),
                ) {
                    pdf_page.add_annotation(link);
                }
                across += page.width;
            }
        }
        pdf_page.finish();
    }
    let authors: Vec<String> = book.authors.iter().map(|a| a.name.clone()).collect();
    let metadata = crate::outline::metadata(
        &navigation,
        options.target,
        &book.title,
        &authors,
        &book.language,
        options.application.as_ref(),
    );
    if let Some(outline) =
        crate::outline::outline(&navigation.bookmarks, &sheet_origins, &navigation.names)
    {
        document.set_outline(outline);
    }
    document.set_metadata(metadata);
    let bytes = document
        .finish()
        .map_err(|e| ExportError::Pdf(format!("{e:?}")))?;
    let notes = drawing.kept.notes();
    if shop {
        // The colours are sRGB, so that is the condition the file declares; a press profile
        // changes it when the book is made for ink.
        let srgb = folio_colour::Profile::srgb();
        let press = ink.and_then(|ink| ink.profile.as_deref());
        let name = press
            .and_then(|bytes| folio_colour::Profile::from_icc(bytes).ok())
            .and_then(|profile| profile.description());
        let made = crate::pdfx::make_pdf_x4(
            &bytes,
            &book.title,
            options.application.as_ref(),
            &match press {
                Some(profile) => crate::pdfx::OutputCondition {
                    identifier: name.as_deref().unwrap_or("Press profile"),
                    profile,
                    components: 4,
                },
                None => crate::pdfx::OutputCondition {
                    identifier: "sRGB IEC61966-2.1",
                    profile: srgb.icc(),
                    components: 3,
                },
            },
        )?;
        return Ok((made, notes));
    }
    Ok((bytes, notes))
}

/// Where on the page a piece of a link is, as a rectangle `(x, y, w, h)` before the frame's turn.
type LinkPiece = (String, (f64, f64, f64, f64));

/// The link annotations of a page: one for each stretch of a line whose words link to the same
/// place, over the line's box. `shift` is how far the page's content is moved on the sheet, and
/// `pages` how many pages the file has, since a link to a page that is not there is left out.
fn link_annotations(
    page: &folio_layout::scene::PageScene,
    first_page: &[Option<usize>],
    pages: usize,
    shift: (f64, f64),
) -> Vec<Annotation> {
    let mut out = Vec::new();
    for item in &page.items {
        let SceneKind::Text {
            at,
            size,
            rotation,
            lines,
        } = &item.kind
        else {
            continue;
        };
        // A frame wholly off the page (the other cover's, on a cover cut from the sheet) has no
        // link to show.
        if at.0 >= page.width || at.0 + size.0 <= 0.0 {
            continue;
        }
        let mut pieces: Vec<LinkPiece> = Vec::new();
        for line in lines {
            let mut open: Option<(String, f64, f64)> = None;
            let close = |open: &mut Option<(String, f64, f64)>, pieces: &mut Vec<LinkPiece>| {
                if let Some((href, from, to)) = open.take() {
                    pieces.push((
                        href,
                        (
                            at.0 + line.x + from,
                            at.1 + line.top,
                            to - from,
                            line.height,
                        ),
                    ));
                }
            };
            for run in &line.runs {
                let Some(href) = run.href.as_deref().filter(|_| !run.text.is_empty()) else {
                    close(&mut open, &mut pieces);
                    continue;
                };
                match &mut open {
                    // The next word of the same link, with only the space between.
                    Some((was, _, to)) if was == href => *to = run.x + run.width,
                    _ => {
                        close(&mut open, &mut pieces);
                        open = Some((href.to_string(), run.x, run.x + run.width));
                    }
                }
            }
            close(&mut open, &mut pieces);
        }
        for (href, (x, y, w, h)) in pieces {
            let target = match folio_model::spread_of_href(&href) {
                Some(n) => {
                    let Some(page) = first_page
                        .get(n - 1)
                        .copied()
                        .flatten()
                        .filter(|p| *p < pages)
                    else {
                        continue;
                    };
                    Target::Destination(XyzDestination::new(page, Point::from_xy(0.0, 0.0)).into())
                }
                None if href.starts_with('#') => continue,
                None => Target::Action(Action::Link(LinkAction::new(href))),
            };
            // The corners, turned with the frame as its words are, and moved with the page.
            let (cx, cy) = (at.0 + size.0 / 2.0, at.1 + size.1 / 2.0);
            let (sin, cos) = rotation.to_radians().sin_cos();
            let corner = |px: f64, py: f64| {
                let (dx, dy) = (px - cx, py - cy);
                Point::from_xy(
                    (cx + dx * cos - dy * sin + shift.0) as f32,
                    (cy + dx * sin + dy * cos + shift.1) as f32,
                )
            };
            out.push(
                LinkAnnotation::new_with_quad_points(
                    vec![Quadrilateral([
                        corner(x, y + h),
                        corner(x + w, y + h),
                        corner(x + w, y),
                        corner(x, y),
                    ])],
                    target,
                )
                .into(),
            );
        }
    }
    out
}

struct Drawing<'a> {
    /// Set for a file whose colours are ink: each one is separated, the page's ink as black alone.
    separator: Option<Separator>,
    /// The book's swatches, so a colour that refers to one is drawn as it and in its exact ink.
    swatches: Vec<folio_model::BookSwatch>,
    fonts: &'a FontBook,
    assets: &'a dyn Assets,
    /// The krilla fonts made so far, by file key, weight and size, since each variable instance is
    /// its own font to the PDF.
    faces: HashMap<(String, u16, u32), Option<Font>>,
    images: HashMap<(String, Option<crate::flatten::Rgb>), Option<Image>>,
    /// Whether the file is flattened: a picture with see-through parts is blended onto what is behind
    /// it when that is plain enough to know.
    flatten: bool,
    /// What to do with the picture the item being drawn holds, when the file is flattened.
    plan: Option<crate::flatten::Plan>,
    /// The sheet being drawn, counted from 1, for the notes.
    page: usize,
    /// Pictures left see-through in a flattened file, by why and where.
    kept: crate::flatten::Kept,
    /// Whether each picture really has see-through parts, once looked at.
    see_through: HashMap<String, bool>,
    /// The widest each picture is drawn, which is what it is sized for.
    demand: crate::picture::Demand,
    policy: ImagePolicy,
    cache: PictureCache,
    cancel: crate::cancel::Cancel,
    /// Tells one separation from another, so the cache keeps each apart.
    ink_key: u64,
    /// Texture tiles made so far, by texture and tint, with each tile's side in points.
    textures: HashMap<String, Option<(Image, f64)>>,
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

/// The share of a radius a cubic curve's handles take to draw a quarter circle.
const KAPPA: f32 = 0.552_284_8;

const INK: &str = "#1c1b20";

fn krect(r: &SceneRect) -> Option<KRect> {
    KRect::from_xywh(
        r.x as f32,
        r.y as f32,
        r.w.max(0.0) as f32,
        r.h.max(0.0) as f32,
    )
}

impl<'a> Drawing<'a> {
    fn new(
        fonts: &'a FontBook,
        assets: &'a dyn Assets,
        separator: Option<Separator>,
        swatches: Vec<folio_model::BookSwatch>,
    ) -> Self {
        Self {
            separator,
            swatches,
            fonts,
            assets,
            faces: HashMap::new(),
            images: HashMap::new(),
            flatten: false,
            plan: None,
            page: 0,
            kept: Default::default(),
            see_through: HashMap::new(),
            demand: HashMap::new(),
            policy: ImagePolicy::standard(),
            cache: PictureCache::default(),
            cancel: Default::default(),
            ink_key: 0,
            textures: HashMap::new(),
        }
    }

    /// Draws `item` into `area`: the page and the bleed beyond its outer edge, which is as far as
    /// anything goes. A frame across the gutter shows only its own side.
    /// A colour as the file holds it: sRGB, or ink when the file is for print.
    fn color(&self, c: &Colour) -> krilla::color::Color {
        let paint = folio_model::Swatches(&self.swatches).resolve(c);
        let rgb = (paint.rgb[0], paint.rgb[1], paint.rgb[2]);
        match &self.separator {
            None => rgb::Color::new(rgb.0, rgb.1, rgb.2).into(),
            // A swatch with ink prints as the ink it names, with no conversion.
            Some(_) if paint.ink.is_some() => {
                let ink = paint.ink.expect("checked above");
                let plate = |percent: u8| (u16::from(percent) * 255 / 100) as u8;
                cmyk::Color::new(plate(ink.c), plate(ink.m), plate(ink.y), plate(ink.k)).into()
            }
            Some(separator) => {
                let ink = separator.cmyk(rgb);
                let plate = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
                cmyk::Color::new(plate(ink.c), plate(ink.m), plate(ink.y), plate(ink.k)).into()
            }
        }
    }

    fn fill_of(&self, c: &Colour, opacity: f32) -> Fill {
        Fill {
            paint: self.color(c).into(),
            opacity: NormalizedF32::new(opacity).unwrap_or(NormalizedF32::ONE),
            rule: FillRule::default(),
        }
    }

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
                            surface.set_fill(Some(
                                self.fill_of(&Colour("#14110f".into()), *opacity as f32),
                            ));
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

    /// Crop marks at the four corners of the trim, starting a little outside the bleed, in the
    /// registration colour (all the ink there is).
    fn crop_marks(&mut self, surface: &mut Surface, width: f64, height: f64, bleed: f64) {
        let (gap, length) = (bleed + 3.0, 18.0);
        let mark = |surface: &mut Surface, x1: f64, y1: f64, x2: f64, y2: f64| {
            let mut pb = PathBuilder::new();
            pb.move_to(x1 as f32, y1 as f32);
            pb.line_to(x2 as f32, y2 as f32);
            if let Some(path) = pb.finish() {
                surface.set_fill(None);
                surface.set_stroke(Some(Stroke {
                    paint: self.registration().into(),
                    width: 0.25,
                    ..Stroke::default()
                }));
                surface.draw_path(&path);
            }
        };
        for (x, dx) in [(0.0, -1.0), (width, 1.0)] {
            for (y, dy) in [(0.0, -1.0), (height, 1.0)] {
                mark(surface, x + dx * gap, y, x + dx * (gap + length), y);
                mark(surface, x, y + dy * gap, x, y + dy * (gap + length));
            }
        }
    }

    /// The colour of a mark every plate prints: all the ink in a file of ink, black in sRGB.
    fn registration(&self) -> krilla::color::Color {
        if self.separator.is_some() {
            cmyk::Color::new(255, 255, 255, 255).into()
        } else {
            rgb::Color::new(0, 0, 0).into()
        }
    }

    fn rule(&mut self, surface: &mut Surface, rule: &SceneLine) {
        let mut pb = PathBuilder::new();
        pb.move_to(rule.x1 as f32, rule.y1 as f32);
        pb.line_to(rule.x2 as f32, rule.y2 as f32);
        if let Some(path) = pb.finish() {
            surface.set_fill(None);
            surface.set_stroke(Some(Stroke {
                paint: self.color(&Colour("#14110f".into())).into(),
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
            Background::Colour { colour: c } => {
                self.fill_rect(surface, &whole, self.fill_of(c, 1.0))
            }
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
                            color: self.color(&s.colour),
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
            // A texture is its tile drawn at 300 ppi and laid edge to edge from the page's corner.
            Background::Texture { id, tint } => {
                if !self.tiles(surface, id, tint.as_ref(), &whole) {
                    let c = tint.clone().unwrap_or_else(|| Colour("#ebe5d6".into()));
                    self.fill_rect(surface, &whole, self.fill_of(&c, 1.0));
                }
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

    /// Lays tiles of a texture over `area`; `false` when Folio cannot draw that texture.
    fn tiles(
        &mut self,
        surface: &mut Surface,
        id: &str,
        tint: Option<&Colour>,
        area: &SceneRect,
    ) -> bool {
        // A tint that is a swatch is drawn as the colour the swatch shows as.
        let tint_hex = tint.map(|c| {
            let [r, g, b] = folio_model::Swatches(&self.swatches).resolve(c).rgb;
            format!("#{r:02x}{g:02x}{b:02x}")
        });
        let key = format!("{id}|{}", tint_hex.as_deref().unwrap_or(""));
        let entry = self.textures.entry(key).or_insert_with(|| {
            let (png, size) = crate::texture::tile_png(id, tint_hex.as_deref())?;
            Some((
                Image::from_png(
                    Data::from(Arc::new(png) as Arc<dyn AsRef<[u8]> + Send + Sync>),
                    false,
                )
                .ok()?,
                size,
            ))
        });
        let Some((image, size)) = entry.clone() else {
            return false;
        };
        let Some(tile) = Size::from_wh(size as f32, size as f32) else {
            return false;
        };
        let (x0, y0) = (
            (area.x / size).floor() * size,
            (area.y / size).floor() * size,
        );
        let mut y = y0;
        while y < area.y + area.h {
            let mut x = x0;
            while x < area.x + area.w {
                surface.push_transform(&Transform::from_translate(x as f32, y as f32));
                surface.draw_image(image.clone(), tile);
                surface.pop();
                x += size;
            }
            y += size;
        }
        true
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
        let backdrop = match self.plan {
            Some(crate::flatten::Plan::Flatten(rgb)) => Some(rgb),
            Some(crate::flatten::Plan::Live(why)) => {
                if asset.is_some_and(|path| self.is_see_through(path)) {
                    self.kept.add(why, self.page);
                }
                None
            }
            None => None,
        };
        let image = asset.and_then(|path| self.image(path, backdrop));
        match image {
            None => {
                // A frame still waiting for its picture, or one that cannot be found: a quiet box.
                self.fill_rect(surface, rect, self.fill_of(&Colour("#e0ddd4".into()), 0.6));
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

    /// Whether the picture at `path` has any see-through part.
    fn is_see_through(&mut self, path: &str) -> bool {
        if let Some(found) = self.see_through.get(path) {
            return *found;
        }
        let found = self
            .assets
            .bytes(path)
            .is_some_and(|bytes| crate::picture::is_see_through(&bytes));
        self.see_through.insert(path.to_string(), found);
        found
    }

    fn image(&mut self, path: &str, backdrop: Option<crate::flatten::Rgb>) -> Option<Image> {
        let id = (path.to_string(), backdrop);
        if let Some(found) = self.images.get(&id) {
            return found.clone();
        }
        if self.cancel.is_set() {
            return None;
        }
        let width = self.demand.get(path).copied().unwrap_or(0.0);
        let made = self
            .cache
            .get(
                path,
                self.assets,
                width,
                &self.policy,
                self.separator
                    .as_ref()
                    .map(|separator| crate::picture::Ink {
                        separator,
                        key: self.ink_key,
                    })
                    .as_ref(),
                backdrop,
            )
            .and_then(|prepared| prepared.image());
        self.images.insert(id, made.clone());
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
        let fill_paint = fill.as_ref().map(|c| self.fill_of(c, 1.0));
        let stroke_paint = stroke.as_ref().map(|c| Stroke {
            paint: self.color(c).into(),
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
                        paint: self.color(&ink).into(),
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
        surface.set_fill(Some(self.fill_of(&ink, 1.0)));
        surface.set_stroke(None);
        surface.draw_glyphs(
            Point::from_xy((line_x + run.x) as f32, baseline as f32),
            &glyphs,
            font,
            &run.text,
            size,
            false,
        );
        // Lines under and through the words are drawn as thin rectangles in the words' colour.
        for rule in [&run.underline, &run.strike].into_iter().flatten() {
            let (x, y) = (
                line_x + run.x,
                baseline + rule.offset - rule.thickness / 2.0,
            );
            if let Some(rect) =
                KRect::from_xywh(x as f32, y as f32, rule.width as f32, rule.thickness as f32)
            {
                let mut pb = PathBuilder::new();
                pb.push_rect(rect);
                self.paint_path(surface, pb, Some(self.fill_of(&ink, 1.0)), None);
            }
        }
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
