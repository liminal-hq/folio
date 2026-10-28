// The fixed-layout EPUB: every page as it is laid out, for picture books
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! A picture book cannot reflow: the words sit where the pictures want them. A fixed-layout EPUB
//! is one XHTML page for each page of the book, a viewport the size of the page, and everything on
//! it placed absolutely: the pictures as images, the shapes as SVG and the words as live text, one
//! piece at a time at the position layout gave it, so the lines break where Folio broke them and a
//! screen reader still reads them. The reader scales the page to its screen.

use std::collections::BTreeMap;
use std::io::Write;

use folio_layout::scene::{focused_rect, PageScene, SceneKind, SceneRect};
use folio_layout::{FontBook, FontRequest, Layouter, StyleSheet};
use folio_model::{Background, Book, Colour, Fit, Shape};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

use crate::error::ExportError;
use crate::pdf::Assets;
use crate::raster::page_to_jpeg;

/// The size of the cover picture a store asks for, which is also the cover page's size.
const COVER_W: u32 = 1600;
const COVER_H: u32 = 2560;

fn esc(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn px(n: f64) -> String {
    let text = format!("{n:.2}");
    text.trim_end_matches('0').trim_end_matches('.').to_string()
}

fn media_type(path: &str) -> &'static str {
    match path
        .rsplit('.')
        .next()
        .unwrap_or("")
        .to_ascii_lowercase()
        .as_str()
    {
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "gif" => "image/gif",
        "webp" => "image/webp",
        _ => "application/octet-stream",
    }
}

struct Page {
    body: String,
}

fn css_colour(c: &Colour) -> &str {
    &c.0
}

/// One page as XHTML: the background on the page itself, then each item over it in order.
fn page_html(
    scene: &PageScene,
    names: &mut BTreeMap<String, String>,
    images: &mut Vec<String>,
    assets: &dyn Assets,
    natural: &dyn Fn(&str) -> Option<(f64, f64)>,
) -> Page {
    let mut image_name = |asset: &str| -> Option<String> {
        assets.bytes(asset)?;
        if !names.contains_key(asset) {
            let ext = asset.rsplit('.').next().unwrap_or("bin");
            names.insert(
                asset.to_string(),
                format!("page-image-{:03}.{ext}", names.len() + 1),
            );
            images.push(asset.to_string());
        }
        names.get(asset).cloned()
    };
    let mut ground = String::from("background:#ffffff;");
    let mut inner = String::new();
    for item in &scene.items {
        match &item.kind {
            SceneKind::Background { bg, .. } => match bg {
                Background::Colour { colour } => {
                    ground = format!("background:{};", css_colour(colour))
                }
                Background::Texture { tint, .. } => {
                    let c = tint.clone().unwrap_or_else(|| Colour("#ebe5d6".into()));
                    ground = format!("background:{};", css_colour(&c));
                }
                Background::Gradient { angle, stops } => {
                    let stops: Vec<String> = stops
                        .iter()
                        .map(|s| format!("{} {}%", css_colour(&s.colour), px(s.at * 100.0)))
                        .collect();
                    ground = format!(
                        "background:linear-gradient({}deg,{});",
                        px(*angle),
                        stops.join(",")
                    );
                }
                Background::Picture { asset, fit, focus } => {
                    if let Some(name) = image_name(asset) {
                        let fit = match fit {
                            folio_model::BgFit::Fill => Fit::Fill,
                            folio_model::BgFit::Fit => Fit::Fit,
                        };
                        let whole = SceneRect {
                            x: 0.0,
                            y: 0.0,
                            w: scene.width,
                            h: scene.height,
                        };
                        inner.push_str(&picture(
                            &whole,
                            0.0,
                            &name,
                            "",
                            fit,
                            *focus,
                            natural(asset),
                        ));
                    }
                }
            },
            SceneKind::Picture {
                rect,
                rotation,
                asset,
                fit,
                focus,
                ..
            } => {
                if let Some(name) = asset
                    .as_deref()
                    .and_then(|a| image_name(a).map(|n| (a.to_string(), n)))
                {
                    inner.push_str(&picture(
                        rect,
                        *rotation,
                        &name.1,
                        "",
                        *fit,
                        *focus,
                        natural(&name.0),
                    ));
                }
            }
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
            } => {
                inner.push_str(&shape_svg(
                    rect,
                    *rotation,
                    *shape,
                    fill,
                    stroke,
                    *stroke_width,
                    *radius,
                    *flip,
                    paths,
                ));
            }
            SceneKind::Table { tints, rules, .. } => {
                let mut svg = format!(
                    "<svg xmlns=\"http://www.w3.org/2000/svg\" style=\"position:absolute;left:0;top:0\" width=\"{}\" height=\"{}\" viewBox=\"0 0 {} {}\" aria-hidden=\"true\">",
                    px(scene.width), px(scene.height), px(scene.width), px(scene.height)
                );
                for (r, opacity) in tints {
                    svg.push_str(&format!(
                        "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"#14110f\" fill-opacity=\"{}\"/>",
                        px(r.x), px(r.y), px(r.w), px(r.h), px(*opacity)
                    ));
                }
                for l in rules {
                    svg.push_str(&format!(
                        "<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"#14110f\" stroke-opacity=\"0.7\" stroke-width=\"{}\"/>",
                        px(l.x1), px(l.y1), px(l.x2), px(l.y2), px(l.width)
                    ));
                }
                svg.push_str("</svg>\n");
                inner.push_str(&svg);
            }
            SceneKind::Text {
                at,
                size,
                rotation,
                lines,
            } => {
                let turn = if *rotation != 0.0 {
                    format!("transform:rotate({}deg);", px(*rotation))
                } else {
                    String::new()
                };
                inner.push_str(&format!(
                    "<div style=\"position:absolute;left:{}px;top:{}px;width:{}px;height:{}px;{turn}\">\n",
                    px(at.0), px(at.1), px(size.0), px(size.1)
                ));
                for line in lines {
                    let words = words_of(&line.runs);
                    if words.is_empty() {
                        continue;
                    }
                    // A line is one positioned box with its words in it as running text, spaces
                    // and all. A reader that sets a word wider than the book was laid out
                    // moves the rest of the line along, where words each placed at a point would
                    // run into each other (some readers also round every letter to whole
                    // pixels), and a line that does not wrap cannot spill onto the next one.
                    inner.push_str(&format!(
                        "<div class=\"ln\" style=\"position:absolute;left:{}px;top:{}px;height:{}px;line-height:{}px\">",
                        px(line.x),
                        px(line.top),
                        px(line.height),
                        px(line.height)
                    ));
                    let mut open: Option<&Word> = None;
                    let mut text = String::new();
                    let flush = |inner: &mut String, word: Option<&Word>, text: &mut String| {
                        if let Some(word) = word {
                            inner.push_str(&format!(
                                "<span style=\"{}\">{}</span>",
                                word_style(word),
                                esc(text)
                            ));
                        }
                        text.clear();
                    };
                    for (i, word) in words.iter().enumerate() {
                        if open.is_some_and(|o| !o.same_style(word)) {
                            flush(&mut inner, open, &mut text);
                        }
                        open = Some(word);
                        text.push_str(&word.text);
                        if words
                            .get(i + 1)
                            .is_some_and(|next| next.x > word.end + 0.01)
                        {
                            text.push(' ');
                        }
                    }
                    flush(&mut inner, open, &mut text);
                    inner.push_str("</div>\n");
                }
                inner.push_str("</div>\n");
            }
        }
    }
    let body = format!(
        "<div class=\"page\" style=\"{ground}width:{}px;height:{}px\">\n{inner}</div>\n",
        px(scene.width),
        px(scene.height)
    );
    Page { body }
}

/// A word as it is set on a line: the pieces layout cut it into (for hyphenation) put back
/// together, in one font.
struct Word {
    text: String,
    x: f64,
    end: f64,
    family: String,
    size: f64,
    weight: u16,
    italic: bool,
    tracking: f64,
    colour: Option<Colour>,
}

impl Word {
    /// Whether `other` is set in the same font, size, weight, style, tracking and colour.
    fn same_style(&self, other: &Word) -> bool {
        self.family == other.family
            && self.size == other.size
            && self.weight == other.weight
            && self.italic == other.italic
            && self.tracking == other.tracking
            && self.colour == other.colour
    }
}

/// The CSS for words set as `word` is.
fn word_style(word: &Word) -> String {
    format!(
        "white-space:pre;font-family:'{}';font-size:{}px;font-weight:{};font-style:{};letter-spacing:{}em;color:{}",
        esc(&word.family),
        px(word.size),
        word.weight,
        if word.italic { "italic" } else { "normal" },
        px(word.tracking),
        word.colour.as_ref().map_or("#1c1b20", |c| css_colour(c)),
    )
}

/// The words of a line. Pieces that touch and are set alike are one word; a hyphen drawn at the
/// end of a line is part of the word it ends.
fn words_of(runs: &[folio_layout::wire::LayoutRun]) -> Vec<Word> {
    let mut words: Vec<Word> = Vec::new();
    for run in runs {
        if run.text.is_empty() && !run.hyphen {
            continue;
        }
        let text = if run.hyphen {
            format!("{}-", run.text)
        } else {
            run.text.clone()
        };
        if let Some(last) = words.last_mut() {
            let alike = last.family == run.family
                && last.size == run.size
                && last.weight == run.weight
                && last.italic == run.italic
                && last.tracking == run.tracking
                && last.colour == run.colour;
            if alike && (run.x - last.end).abs() < 0.01 {
                last.text.push_str(&text);
                last.end = run.x + run.width;
                continue;
            }
        }
        words.push(Word {
            text,
            x: run.x,
            end: run.x + run.width,
            family: run.family.clone(),
            size: run.size,
            weight: run.weight,
            italic: run.italic,
            tracking: run.tracking,
            colour: run.colour.clone(),
        });
    }
    words
}

fn picture(
    rect: &SceneRect,
    rotation: f64,
    name: &str,
    alt: &str,
    fit: Fit,
    focus: (f64, f64),
    natural: Option<(f64, f64)>,
) -> String {
    let turn = if rotation != 0.0 {
        format!("transform:rotate({}deg);", px(rotation))
    } else {
        String::new()
    };
    let placed = natural.map(|n| focused_rect(*rect, n, fit, focus));
    let (x, y, w, h) = placed.map_or((0.0, 0.0, rect.w, rect.h), |p| {
        (p.x - rect.x, p.y - rect.y, p.w, p.h)
    });
    format!(
        "<div style=\"position:absolute;left:{}px;top:{}px;width:{}px;height:{}px;overflow:hidden;{turn}\"><img src=\"../images/{name}\" alt=\"{}\" style=\"position:absolute;left:{}px;top:{}px;width:{}px;height:{}px\"/></div>\n",
        px(rect.x), px(rect.y), px(rect.w), px(rect.h), esc(alt), px(x), px(y), px(w), px(h)
    )
}

#[allow(clippy::too_many_arguments)]
fn shape_svg(
    rect: &SceneRect,
    rotation: f64,
    shape: Shape,
    fill: &Option<Colour>,
    stroke: &Option<Colour>,
    stroke_width: Option<f64>,
    radius: Option<f64>,
    flip: bool,
    paths: &[String],
) -> String {
    let turn = if rotation != 0.0 {
        format!("transform:rotate({}deg);", px(rotation))
    } else {
        String::new()
    };
    let paint = format!(
        "fill=\"{}\" stroke=\"{}\" stroke-width=\"{}\"",
        fill.as_ref().map_or("none", |c| css_colour(c)),
        stroke.as_ref().map_or("none", |c| css_colour(c)),
        px(stroke_width.unwrap_or(1.0))
    );
    let inner = match shape {
        Shape::Line => {
            let (y1, y2) = if flip { (rect.h, 0.0) } else { (0.0, rect.h) };
            let ink = stroke.as_ref().or(fill.as_ref()).map_or("#14110f", |c| css_colour(c));
            format!("<line x1=\"0\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"{ink}\" stroke-width=\"{}\"/>", px(y1), px(rect.w), px(y2), px(stroke_width.unwrap_or(2.0)))
        }
        Shape::Ellipse => format!("<ellipse cx=\"{}\" cy=\"{}\" rx=\"{}\" ry=\"{}\" {paint}/>", px(rect.w / 2.0), px(rect.h / 2.0), px(rect.w / 2.0), px(rect.h / 2.0)),
        Shape::Rect | Shape::Round => {
            let r = if shape == Shape::Round { radius.unwrap_or(rect.w.min(rect.h) * 0.12) } else { 0.0 };
            format!("<rect width=\"{}\" height=\"{}\" rx=\"{}\" {paint}/>", px(rect.w), px(rect.h), px(r))
        }
        Shape::Speech | Shape::Thought | Shape::Shout => paths
            .iter()
            .map(|d| format!("<path d=\"{d}\" transform=\"scale({} {})\" {paint} vector-effect=\"non-scaling-stroke\"/>", px(rect.w), px(rect.h)))
            .collect(),
    };
    format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" style=\"position:absolute;left:{}px;top:{}px;overflow:visible;{turn}\" width=\"{}\" height=\"{}\" viewBox=\"0 0 {} {}\" aria-hidden=\"true\">{inner}</svg>\n",
        px(rect.x), px(rect.y), px(rect.w.max(1.0)), px(rect.h.max(1.0)), px(rect.w.max(1.0)), px(rect.h.max(1.0))
    )
}

/// A fixed-layout EPUB 3 of every page of `book`.
pub fn export_fixed_epub(
    book: &Book,
    styles: &StyleSheet,
    fonts: &FontBook,
    assets: &dyn Assets,
) -> Result<Vec<u8>, ExportError> {
    let front = folio_model::front_cover_view(book);
    let book = &front;
    // The cover is the front alone: the printed sheet's back and spine are not a page of this book.
    let front = folio_model::front_cover_view(book);
    let book = &front;
    let layouter = Layouter::new(book, styles, fonts);
    let mut scenes: Vec<PageScene> = Vec::new();
    for spread in &book.spreads {
        if let Some(scene) = layouter.scene_spread(&spread.id)? {
            scenes.extend(scene.pages);
        }
    }
    if scenes.is_empty() {
        return Err(ExportError::NoPages);
    }
    // Natural pixel sizes decide where a picture sits in its frame.
    let natural = |path: &str| -> Option<(f64, f64)> {
        let bytes = assets.bytes(path)?;
        let size = imagesize(&bytes)?;
        Some((f64::from(size.0), f64::from(size.1)))
    };
    let mut names: BTreeMap<String, String> = BTreeMap::new();
    let mut images: Vec<String> = Vec::new();
    let mut pages: Vec<Page> = Vec::new();
    for scene in &scenes {
        pages.push(page_html(scene, &mut names, &mut images, assets, &natural));
    }
    let (w, h) = (book.trim.w.0, book.trim.h.0);

    // The fonts the pages ask for, one for each weight and style set, which may travel with the
    // book. Each is declared at the weight and style it is, with the kind of file it is, because
    // some readers refuse a font declared across a range of weights and fall back to their own.
    let mut faces: Vec<(String, u16, bool)> = scenes
        .iter()
        .flat_map(|s| s.items.iter())
        .filter_map(|i| match &i.kind {
            SceneKind::Text { lines, .. } => Some(
                lines
                    .iter()
                    .flat_map(|l| {
                        l.runs
                            .iter()
                            .map(|r| (r.family.clone(), r.weight, r.italic))
                    })
                    .collect::<Vec<_>>(),
            ),
            _ => None,
        })
        .flatten()
        .collect();
    faces.sort();
    faces.dedup();
    let mut font_files: Vec<(String, std::sync::Arc<[u8]>)> = Vec::new();
    let mut css = String::from(
        "html,body{margin:0;padding:0}\n.page{position:relative;overflow:hidden}\n.ln{white-space:pre}\n",
    );
    for (family, weight, italic) in &faces {
        if let Ok(found) = fonts.resolve(&FontRequest::new(family, *weight, *italic)) {
            if found.font.embedding_rights().may_keep_with_book() {
                let opentype = found.font.data().starts_with(b"OTTO");
                let ext = if opentype { "otf" } else { "ttf" };
                let file = format!("{}.{ext}", found.font.face_key());
                if !font_files.iter().any(|(n, _)| n == &file) {
                    font_files.push((file.clone(), std::sync::Arc::clone(found.font.data())));
                }
                css.push_str(&format!(
                    "@font-face{{font-family:'{}';src:url('../fonts/{file}') format('{}');font-weight:{weight};font-style:{}}}\n",
                    esc(family),
                    if opentype { "opentype" } else { "truetype" },
                    if *italic { "italic" } else { "normal" },
                ));
            }
        }
    }

    let stored = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
    let deflated = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
    let mut zip = ZipWriter::new(std::io::Cursor::new(Vec::new()));
    let put = |zip: &mut ZipWriter<std::io::Cursor<Vec<u8>>>,
               name: &str,
               bytes: &[u8],
               opts: SimpleFileOptions|
     -> Result<(), ExportError> {
        zip.start_file(name, opts)
            .map_err(|e| ExportError::Epub(e.to_string()))?;
        zip.write_all(bytes)
            .map_err(|e| ExportError::Epub(e.to_string()))
    };
    put(&mut zip, "mimetype", b"application/epub+zip", stored)?;
    put(
        &mut zip,
        "META-INF/container.xml",
        b"<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<container version=\"1.0\" xmlns=\"urn:oasis:names:tc:opendocument:xmlns:container\">\n<rootfiles><rootfile full-path=\"OEBPS/content.opf\" media-type=\"application/oebps-package+xml\"/></rootfiles>\n</container>\n",
        deflated,
    )?;
    put(&mut zip, "OEBPS/css/book.css", css.as_bytes(), deflated)?;
    // The first document of the spine is the cover picture. It is a reflowable page in the
    // fixed-layout book (see the spine), so the reader scales the picture to the screen: a Kobo
    // does not scale a fixed page by its own viewport but with the other pages', which cropped
    // it. It is what a reader that makes a thumbnail from the first document, and a store that
    // looks for the cover, show. The cover page the author laid out is the picture in it.
    let cover_page = format!(
        "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<!DOCTYPE html>\n<html xmlns=\"http://www.w3.org/1999/xhtml\" xmlns:epub=\"http://www.idpf.org/2007/ops\" lang=\"{lang}\" xml:lang=\"{lang}\">\n<head>\n<meta charset=\"utf-8\"/>\n<title>{title} — cover</title>\n<link rel=\"stylesheet\" type=\"text/css\" href=\"../css/book.css\"/>\n</head>\n<body epub:type=\"cover\">\n<div style=\"text-align:center\"><img src=\"../images/cover.jpg\" alt=\"Cover of {title}\" style=\"max-width:100%;max-height:100vh;height:auto\"/></div>\n</body>\n</html>\n",
        lang = esc(&book.language),
        title = esc(&book.title),
    );
    put(
        &mut zip,
        "OEBPS/text/cover.xhtml",
        cover_page.as_bytes(),
        deflated,
    )?;
    for (index, page) in pages.iter().enumerate().skip(1) {
        let doc = format!(
            "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<!DOCTYPE html>\n<html xmlns=\"http://www.w3.org/1999/xhtml\" xmlns:epub=\"http://www.idpf.org/2007/ops\" lang=\"{lang}\" xml:lang=\"{lang}\">\n<head>\n<meta charset=\"utf-8\"/>\n<meta name=\"viewport\" content=\"width={w}, height={h}\"/>\n<title>{title} — page {n}</title>\n<link rel=\"stylesheet\" type=\"text/css\" href=\"../css/book.css\"/>\n</head>\n<body>\n{body}</body>\n</html>\n",
            lang = esc(&book.language),
            title = esc(&book.title),
            n = index + 1,
            w = px(w),
            h = px(h),
            body = page.body
        );
        put(
            &mut zip,
            &format!("OEBPS/text/page-{:03}.xhtml", index + 1),
            doc.as_bytes(),
            deflated,
        )?;
    }
    let mut manifest_images = String::new();
    for path in &images {
        if let (Some(bytes), Some(name)) = (assets.bytes(path), names.get(path)) {
            put(&mut zip, &format!("OEBPS/images/{name}"), &bytes, deflated)?;
            manifest_images.push_str(&format!(
                "<item id=\"img-{name}\" href=\"images/{name}\" media-type=\"{}\"/>\n",
                media_type(name)
            ));
        }
    }
    // The cover, as the picture a store would show.
    let cover = scenes.first().ok_or(ExportError::NoPages)?;
    let cover_jpeg = page_to_jpeg(cover, fonts, assets, COVER_W, COVER_H)?;
    put(&mut zip, "OEBPS/images/cover.jpg", &cover_jpeg, deflated)?;
    manifest_images.push_str("<item id=\"cover-image\" href=\"images/cover.jpg\" media-type=\"image/jpeg\" properties=\"cover-image\"/>\n");
    let mut manifest_fonts = String::new();
    for (name, data) in &font_files {
        put(&mut zip, &format!("OEBPS/fonts/{name}"), data, deflated)?;
        manifest_fonts.push_str(&format!(
            "<item id=\"font-{name}\" href=\"fonts/{name}\" media-type=\"{}\"/>\n",
            if name.ends_with(".otf") {
                "font/otf"
            } else {
                "font/ttf"
            }
        ));
    }
    let mut manifest_pages = String::from(
        "<item id=\"cover\" href=\"text/cover.xhtml\" media-type=\"application/xhtml+xml\"/>\n",
    );
    manifest_pages.extend((2..=pages.len())
        .map(|i| format!("<item id=\"page-{i:03}\" href=\"text/page-{i:03}.xhtml\" media-type=\"application/xhtml+xml\"/>\n")));
    let spine: String = scenes
        .iter()
        .enumerate()
        .map(|(i, s)| {
            format!(
                "<itemref idref=\"{}\" properties=\"{}\"/>\n",
                if i == 0 {
                    "cover".to_string()
                } else {
                    format!("page-{:03}", i + 1)
                },
                if i == 0 {
                    "rendition:layout-reflowable"
                } else if s.recto {
                    "page-spread-right"
                } else {
                    "page-spread-left"
                }
            )
        })
        .collect();
    let nav = format!(
        "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<!DOCTYPE html>\n<html xmlns=\"http://www.w3.org/1999/xhtml\" xmlns:epub=\"http://www.idpf.org/2007/ops\" lang=\"{lang}\" xml:lang=\"{lang}\">\n<head><meta charset=\"utf-8\"/><title>{title}</title></head>\n<body>\n<nav epub:type=\"toc\" id=\"toc\"><h1>Contents</h1>\n<ol><li><a href=\"text/cover.xhtml\">{title}</a></li></ol></nav>\n<nav epub:type=\"landmarks\" hidden=\"hidden\"><h2>Landmarks</h2>\n<ol><li><a epub:type=\"cover\" href=\"text/cover.xhtml\">Cover</a></li></ol></nav>\n</body>\n</html>\n",
        lang = esc(&book.language),
        title = esc(&book.title)
    );
    put(&mut zip, "OEBPS/nav.xhtml", nav.as_bytes(), deflated)?;
    let authors: String = book
        .authors
        .iter()
        .map(|a| format!("<dc:creator>{}</dc:creator>\n", esc(&a.name)))
        .collect();
    let opf = format!(
        "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<package xmlns=\"http://www.idpf.org/2007/opf\" version=\"3.0\" unique-identifier=\"book-id\" xml:lang=\"{lang}\">\n<metadata xmlns:dc=\"http://purl.org/dc/elements/1.1/\">\n<dc:identifier id=\"book-id\">urn:folio:{id}</dc:identifier>\n<dc:title>{title}</dc:title>\n{authors}<dc:language>{lang}</dc:language>\n<meta name=\"cover\" content=\"cover-image\"/>\n<meta property=\"dcterms:modified\">{modified}</meta>\n<meta property=\"rendition:layout\">pre-paginated</meta>\n<meta property=\"rendition:orientation\">auto</meta>\n<meta property=\"rendition:spread\">landscape</meta>\n<meta property=\"schema:accessMode\">textual</meta>\n<meta property=\"schema:accessMode\">visual</meta>\n<meta property=\"schema:accessibilityFeature\">readingOrder</meta>\n<meta property=\"schema:accessibilitySummary\">A fixed-layout picture book: the words are live text over the pictures.</meta>\n</metadata>\n<manifest>\n<item id=\"nav\" href=\"nav.xhtml\" media-type=\"application/xhtml+xml\" properties=\"nav\"/>\n<item id=\"css\" href=\"css/book.css\" media-type=\"text/css\"/>\n{manifest_pages}{manifest_images}{manifest_fonts}</manifest>\n<spine>\n{spine}</spine>\n<guide>\n<reference type=\"cover\" title=\"Cover\" href=\"text/cover.xhtml\"/>\n</guide>\n</package>\n",
        lang = esc(&book.language),
        id = esc(&book.id.0),
        title = esc(&book.title),
        modified = crate::time::utc_now(),
    );
    put(&mut zip, "OEBPS/content.opf", opf.as_bytes(), deflated)?;
    let done = zip.finish().map_err(|e| ExportError::Epub(e.to_string()))?;
    Ok(done.into_inner())
}

/// The width and height of a picture file, from its header.
fn imagesize(bytes: &[u8]) -> Option<(u32, u32)> {
    image::ImageReader::new(std::io::Cursor::new(bytes))
        .with_guessed_format()
        .ok()?
        .into_dimensions()
        .ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use folio_layout::wire::LayoutRun;

    fn run(text: &str, x: f64, width: f64) -> LayoutRun {
        LayoutRun {
            text: text.to_string(),
            x,
            width,
            family: "Serif".to_string(),
            size: 12.0,
            weight: 400,
            italic: false,
            tracking: 0.0,
            colour: None,
            hyphen: false,
            miss: Vec::new(),
            face_key: String::new(),
            glyphs: Vec::new(),
        }
    }

    #[test]
    fn the_pieces_of_an_unbroken_word_are_one_word() {
        let runs = [
            run("grand", 0.0, 30.0),
            run("par", 30.0, 18.0),
            run("ents'", 48.0, 27.0),
        ];
        let words = words_of(&runs);
        assert_eq!(words.len(), 1);
        assert_eq!(words[0].text, "grandparents'");
        assert_eq!(words[0].end, 75.0);
    }

    #[test]
    fn a_gap_between_words_keeps_them_two_words() {
        let runs = [run("Both", 0.0, 24.0), run("side.", 30.0, 28.0)];
        let words = words_of(&runs);
        assert_eq!(
            words.iter().map(|w| w.text.as_str()).collect::<Vec<_>>(),
            ["Both", "side."]
        );
        assert!(words[1].x > words[0].end);
    }

    #[test]
    fn words_set_differently_are_not_joined_and_a_line_end_hyphen_belongs_to_its_word() {
        let mut bold = run("one", 24.0, 20.0);
        bold.weight = 700;
        let mut hyphen = run("coun", 50.0, 24.0);
        hyphen.hyphen = true;
        let words = words_of(&[run("a", 0.0, 24.0), bold, hyphen]);
        assert_eq!(words.len(), 3);
        assert_eq!(words[2].text, "coun-");
    }
}
