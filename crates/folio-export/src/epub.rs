// The EPUB exporter: a book's words and pictures as reflowable XHTML a reader sets in its own way
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! A reflowable EPUB 3 follows the book's story, not its pages: the text frames of every spread
//! are read in order, top to bottom and left to right on each page, and each frame's style is
//! mapped to what it is (a heading, a paragraph, a caption, a poem). The CSS uses relative units so
//! the reader's own text size and fonts apply. A chapter opener starts a new file.

use std::collections::BTreeMap;
use std::io::Write;

use folio_layout::{FontBook, FontRequest};
use folio_model::{Book, Frame, InlineContent, Run, StyleDef, Styles};
use unicode_segmentation::UnicodeSegmentation;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

use crate::error::ExportError;
use crate::pdf::Assets;

/// Which reader the file is made for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EpubFlavour {
    /// A standard EPUB 3 any reader opens: what Kindle Direct Publishing and Kobo take.
    #[default]
    Standard,
    /// A Kobo EPUB: each sentence wrapped in a `koboSpan`, which the reader uses for faster page
    /// turns and reading statistics.
    Kobo,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct EpubOptions {
    pub flavour: EpubFlavour,
    /// Put the fonts the book uses inside the file, when their licences allow it.
    pub embed_fonts: bool,
}

/// What a style is in the book's structure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Role {
    /// A title that opens a chapter or the book.
    Title,
    Subheading,
    Paragraph,
    Caption,
    Epigraph,
    Poem,
    Note,
    /// Only meaningful on a printed page: page numbers, running heads, the spine.
    Skip,
}

fn role_of(style: &str) -> Role {
    match style {
        "display" | "title" | "h1big" | "h1" => Role::Title,
        "numeral" | "h2" => Role::Subheading,
        "caption" => Role::Caption,
        "epigraph" => Role::Epigraph,
        "poem" => Role::Poem,
        "callout" | "footnote" => Role::Note,
        "folio" | "rh" | "spine" | "ornament" | "bubble" | "label" => Role::Skip,
        _ => Role::Paragraph,
    }
}

/// One thing in the story, in reading order.
enum Piece {
    Title(Vec<Run>),
    Block(Role, String, Vec<Vec<Run>>),
    Picture { asset: String, alt: String },
    Table(Vec<Vec<String>>, u8),
}

fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// The paragraphs of some words: one for each line break, each a list of runs.
fn paragraphs(content: &InlineContent) -> Vec<Vec<Run>> {
    let mut out: Vec<Vec<Run>> = vec![Vec::new()];
    for run in &content.inline {
        let mut parts = run.text.split('\n');
        if let Some(first) = parts.next() {
            out.last_mut().expect("one is open").push(Run {
                text: first.to_string(),
                ..run.clone()
            });
        }
        for part in parts {
            out.push(vec![Run {
                text: part.to_string(),
                ..run.clone()
            }]);
        }
    }
    out.retain(|p| p.iter().any(|r| !r.text.trim().is_empty()));
    out
}

fn plain(runs: &[Run]) -> String {
    runs.iter()
        .map(|r| r.text.as_str())
        .collect::<String>()
        .trim()
        .to_string()
}

/// The pieces of a flow's story: each paragraph in the style it names, or the first frame's, with
/// neighbouring paragraphs of one style kept together.
fn flow_pieces(flow: &folio_model::Flow, frame_style: &str) -> Vec<Piece> {
    let mut out: Vec<Piece> = Vec::new();
    for para in &flow.paragraphs {
        let style = para.style.as_ref().map_or(frame_style, |s| s.0.as_str());
        let words = InlineContent {
            inline: para.inline.clone(),
        };
        let paras = paragraphs(&words);
        if paras.is_empty() {
            continue;
        }
        match role_of(style) {
            Role::Skip => {}
            Role::Title => out.push(Piece::Title(paras.into_iter().flatten().collect())),
            role => match out.last_mut() {
                Some(Piece::Block(last_role, last_style, blocks))
                    if *last_role == role && last_style == style =>
                {
                    blocks.extend(paras);
                }
                _ => out.push(Piece::Block(role, style.to_owned(), paras)),
            },
        }
    }
    out
}

/// The story of `book` in reading order. The cover (the first spread) is not part of it.
fn story(book: &Book) -> Vec<Piece> {
    let mut out = Vec::new();
    let mut poured = std::collections::HashSet::new();
    for spread in book.spreads.iter().skip(1) {
        let page_w = book.trim.w.0;
        let mut frames: Vec<(usize, f64, f64, &Frame)> = spread
            .frames
            .iter()
            .map(|f| {
                let r = f.base().rect;
                let page = ((r.x.0 + r.w.0 / 2.0) / page_w).floor().max(0.0) as usize;
                (page, r.y.0, r.x.0, f)
            })
            .collect();
        frames.sort_by(|a, b| {
            (a.0, a.1.round() as i64, a.2.round() as i64).cmp(&(
                b.0,
                b.1.round() as i64,
                b.2.round() as i64,
            ))
        });
        for (_, _, _, frame) in frames {
            match frame {
                // A flowed frame stands for the whole story, told once where it is first met.
                Frame::Text(t)
                    if t.flow
                        .as_ref()
                        .is_some_and(|s| poured.insert(s.id.0.clone())) =>
                {
                    if let Some(flow) = t
                        .flow
                        .as_ref()
                        .and_then(|s| book.flows.iter().find(|f| f.id == s.id))
                    {
                        out.extend(flow_pieces(flow, &t.style.0));
                    }
                }
                Frame::Text(t) if t.flow.is_none() => {
                    let Some(content) = &t.content else { continue };
                    let paras = paragraphs(content);
                    if paras.is_empty() {
                        continue;
                    }
                    let role = role_of(&t.style.0);
                    match role {
                        Role::Skip => {}
                        Role::Title => {
                            out.push(Piece::Title(paras.into_iter().flatten().collect()))
                        }
                        role => out.push(Piece::Block(role, t.style.0.clone(), paras)),
                    }
                }
                Frame::Image(i) => {
                    if let Some(asset) = &i.image.asset {
                        out.push(Piece::Picture {
                            asset: asset.clone(),
                            alt: i.base.description.clone().unwrap_or_default(),
                        });
                    }
                }
                Frame::Table(t) => {
                    let rows = t
                        .rows
                        .iter()
                        .map(|row| {
                            row.iter()
                                .map(|c| {
                                    c.content
                                        .as_ref()
                                        .map(|c| plain(&c.inline))
                                        .unwrap_or_default()
                                })
                                .collect()
                        })
                        .collect();
                    out.push(Piece::Table(rows, t.head));
                }
                _ => {}
            }
        }
    }
    out
}

/// The sentences of `text`, so each can be wrapped for a Kobo reader.
fn sentences(text: &str) -> Vec<&str> {
    text.split_sentence_bounds().collect()
}

struct Writer {
    kobo: bool,
    spans: usize,
    chapter: usize,
}

impl Writer {
    fn runs(&mut self, runs: &[Run]) -> String {
        let mut out = String::new();
        for run in runs {
            let mut text = String::new();
            if self.kobo {
                for sentence in sentences(&run.text) {
                    if sentence.trim().is_empty() {
                        text.push_str(&escape(sentence));
                        continue;
                    }
                    self.spans += 1;
                    text.push_str(&format!(
                        "<span class=\"koboSpan\" id=\"kobo.{}.{}\">{}</span>",
                        self.chapter,
                        self.spans,
                        escape(sentence)
                    ));
                }
            } else {
                text = escape(&run.text);
            }
            if let Some(colour) = &run.colour {
                text = format!("<span style=\"color:{}\">{text}</span>", escape(&colour.0));
            }
            if run.italic {
                text = format!("<em>{text}</em>");
            }
            if run.bold {
                text = format!("<strong>{text}</strong>");
            }
            out.push_str(&text);
        }
        out
    }
}

fn css(book: &Book, families: &BTreeMap<String, Option<(String, bool)>>) -> String {
    let styles: &Styles = &book.styles;
    let base = styles
        .styles
        .get("body")
        .and_then(|s| s.size_pt)
        .unwrap_or(10.5);
    let family = if styles.font_family.is_empty() {
        "Literata".to_string()
    } else {
        styles.font_family.clone()
    };
    let mut out = String::new();
    for (name, file) in families
        .iter()
        .filter_map(|(n, f)| f.as_ref().map(|f| (n, f)))
    {
        out.push_str(&format!(
            "@font-face {{ font-family: \"{name}\"; src: url(\"../fonts/{}\"); font-weight: 100 900; font-style: {}; }}\n",
            file.0,
            if file.1 { "italic" } else { "normal" }
        ));
    }
    out.push_str(&format!(
        "body {{ font-family: \"{family}\", serif; line-height: 1.4; margin: 0; }}\n"
    ));
    out.push_str("p { margin: 0; }\nimg { max-width: 100%; height: auto; }\nfigure { margin: 1em 0; text-align: center; }\n");
    out.push_str("table { border-collapse: collapse; margin: 1em 0; }\ntd, th { padding: 0.2em 0.6em; border-bottom: 1px solid #999; text-align: left; }\n");
    for (id, def) in &styles.styles {
        let StyleDef {
            size_pt,
            weight,
            italic,
            align,
            first_indent,
            space_before,
            space_after,
            ..
        } = def;
        let mut rules = Vec::new();
        if let Some(size) = size_pt {
            rules.push(format!("font-size: {:.3}em", size / base));
        }
        if let Some(w) = weight {
            rules.push(format!("font-weight: {w}"));
        }
        if *italic {
            rules.push("font-style: italic".into());
        }
        if let Some(a) = align {
            rules.push(format!("text-align: {a}"));
        }
        if let Some(indent) = first_indent {
            rules.push(format!("text-indent: {indent}em"));
        }
        if let Some(before) = space_before {
            rules.push(format!("margin-top: {:.3}em", before / base));
        }
        if let Some(after) = space_after {
            rules.push(format!("margin-bottom: {:.3}em", after / base));
        }
        if let Some(def_family) = &def.family {
            rules.push(format!("font-family: \"{def_family}\", serif"));
        }
        if !rules.is_empty() {
            out.push_str(&format!(".{id} {{ {} }}\n", rules.join("; ")));
        }
    }
    out
}

fn xhtml(title: &str, language: &str, body: &str) -> String {
    format!(
        "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<!DOCTYPE html>\n<html xmlns=\"http://www.w3.org/1999/xhtml\" xmlns:epub=\"http://www.idpf.org/2007/ops\" lang=\"{lang}\" xml:lang=\"{lang}\">\n<head>\n<meta charset=\"utf-8\"/>\n<title>{title}</title>\n<link rel=\"stylesheet\" type=\"text/css\" href=\"../css/book.css\"/>\n</head>\n<body>\n{body}</body>\n</html>\n",
        lang = escape(language),
        title = escape(title),
    )
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
        "svg" => "image/svg+xml",
        _ => "application/octet-stream",
    }
}

/// A chapter of the story: its title and its body as XHTML.
#[derive(Debug, Clone)]
pub struct ReflowChapter {
    pub title: String,
    pub body: String,
}

/// The book's story as chapters, and the pictures they use.
#[derive(Debug, Clone)]
pub struct Reflow {
    pub chapters: Vec<ReflowChapter>,
    /// Each picture as its path in the package and its name in the chapters (`../images/<name>`).
    pub images: Vec<(String, String)>,
}

/// The story of `book` as chapters of XHTML. The EPUB and the reader preview both draw from this,
/// so what a person previews is what ships.
pub fn reflow(
    book: &Book,
    assets: &dyn Assets,
    flavour: EpubFlavour,
) -> Result<Reflow, ExportError> {
    let pieces = story(book);
    if pieces.is_empty() {
        return Err(ExportError::NoPages);
    }
    let kobo = flavour == EpubFlavour::Kobo;
    let mut writer = Writer {
        kobo,
        spans: 0,
        chapter: 1,
    };
    // Chapters: each title starts a file, and what comes before the first title is the first.
    let mut chapters: Vec<ReflowChapter> = vec![ReflowChapter {
        title: book.title.clone(),
        body: String::new(),
    }];
    let mut images: Vec<(String, String)> = Vec::new(); // package path, name in the file
    let mut image_names: BTreeMap<String, String> = BTreeMap::new();
    let mut name_image = |asset: &str, images: &mut Vec<(String, String)>| -> String {
        if let Some(name) = image_names.get(asset) {
            return name.clone();
        }
        let ext = asset.rsplit('.').next().unwrap_or("bin");
        let name = format!("image-{:03}.{ext}", images.len() + 1);
        image_names.insert(asset.to_string(), name.clone());
        images.push((asset.to_string(), name.clone()));
        name
    };
    for piece in &pieces {
        match piece {
            Piece::Title(runs) => {
                let text = plain(runs);
                if chapters.len() > 1 || !chapters[0].body.is_empty() {
                    writer.chapter += 1;
                    chapters.push(ReflowChapter {
                        title: text.clone(),
                        body: String::new(),
                    });
                } else {
                    chapters[0].title = text.clone();
                }
                let current = chapters.last_mut().expect("a chapter is open");
                let markup = writer.runs(runs);
                current.body.push_str(&format!("<h1>{markup}</h1>\n"));
            }
            Piece::Block(role, style, paras) => {
                let current = chapters.last_mut().expect("a chapter is open");
                for para in paras {
                    let markup = writer.runs(para);
                    let line = match role {
                        Role::Subheading => format!("<h2 class=\"{style}\">{markup}</h2>\n"),
                        Role::Caption => format!("<p class=\"{style}\">{markup}</p>\n"),
                        Role::Epigraph => {
                            format!("<blockquote class=\"{style}\"><p>{markup}</p></blockquote>\n")
                        }
                        Role::Poem => format!("<p class=\"{style}\">{markup}</p>\n"),
                        Role::Note => {
                            format!("<aside class=\"{style}\"><p>{markup}</p></aside>\n")
                        }
                        _ => format!("<p class=\"{style}\">{markup}</p>\n"),
                    };
                    current.body.push_str(&line);
                }
            }
            Piece::Picture { asset, alt } => {
                if assets.bytes(asset).is_some() {
                    let name = name_image(asset, &mut images);
                    chapters
                        .last_mut()
                        .expect("a chapter is open")
                        .body
                        .push_str(&format!(
                            "<figure><img src=\"../images/{name}\" alt=\"{}\"/></figure>\n",
                            escape(alt)
                        ));
                }
            }
            Piece::Table(rows, head) => {
                let mut html = String::from("<table>\n");
                for (r, row) in rows.iter().enumerate() {
                    html.push_str("<tr>");
                    for cell in row {
                        let tag = if r < usize::from(*head) { "th" } else { "td" };
                        html.push_str(&format!("<{tag}>{}</{tag}>", escape(cell)));
                    }
                    html.push_str("</tr>\n");
                }
                html.push_str("</table>\n");
                chapters
                    .last_mut()
                    .expect("a chapter is open")
                    .body
                    .push_str(&html);
            }
        }
    }

    Ok(Reflow { chapters, images })
}

/// The book as an EPUB 3 file.
pub fn export_epub(
    book: &Book,
    fonts: &FontBook,
    assets: &dyn Assets,
    options: &EpubOptions,
) -> Result<Vec<u8>, ExportError> {
    let Reflow { chapters, images } = reflow(book, assets, options.flavour)?;

    // Fonts the book names that may travel with it.
    let mut families: BTreeMap<String, Option<(String, bool)>> = BTreeMap::new();
    let mut font_files: Vec<(String, std::sync::Arc<[u8]>)> = Vec::new();
    if options.embed_fonts {
        for family in folio_layout::families::families_used(book) {
            let found = fonts.resolve(&FontRequest::new(&family, 400, false)).ok();
            let entry = found
                .filter(|r| !r.family_substituted && r.font.embedding_rights().may_keep_with_book())
                .map(|r| {
                    let ext = if r.font.data().starts_with(b"OTTO") {
                        "otf"
                    } else {
                        "ttf"
                    };
                    let name = format!("{}.{ext}", r.font.face_key());
                    if !font_files.iter().any(|(n, _)| n == &name) {
                        font_files.push((name.clone(), std::sync::Arc::clone(r.font.data())));
                    }
                    (name, r.font.italic())
                });
            families.insert(family, entry);
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
    put(
        &mut zip,
        "OEBPS/css/book.css",
        css(book, &families).as_bytes(),
        deflated,
    )?;
    for (index, chapter) in chapters.iter().enumerate() {
        let body = format!(
            "<section epub:type=\"chapter\">\n{}</section>\n",
            chapter.body
        );
        put(
            &mut zip,
            &format!("OEBPS/text/chapter-{:03}.xhtml", index + 1),
            xhtml(&chapter.title, &book.language, &body).as_bytes(),
            deflated,
        )?;
    }
    let mut manifest_images = String::new();
    for (path, name) in &images {
        if let Some(bytes) = assets.bytes(path) {
            put(&mut zip, &format!("OEBPS/images/{name}"), &bytes, deflated)?;
            manifest_images.push_str(&format!(
                "<item id=\"img-{name}\" href=\"images/{name}\" media-type=\"{}\"/>\n",
                media_type(name)
            ));
        }
    }
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
    let nav_items: String = chapters
        .iter()
        .enumerate()
        .map(|(i, c)| {
            format!(
                "<li><a href=\"text/chapter-{:03}.xhtml\">{}</a></li>\n",
                i + 1,
                escape(&c.title)
            )
        })
        .collect();
    let nav = format!(
        "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<!DOCTYPE html>\n<html xmlns=\"http://www.w3.org/1999/xhtml\" xmlns:epub=\"http://www.idpf.org/2007/ops\" lang=\"{lang}\" xml:lang=\"{lang}\">\n<head><meta charset=\"utf-8\"/><title>{title}</title></head>\n<body>\n<nav epub:type=\"toc\" id=\"toc\"><h1>{contents}</h1>\n<ol>\n{nav_items}</ol></nav>\n</body>\n</html>\n",
        lang = escape(&book.language),
        title = escape(&book.title),
        contents = "Contents",
    );
    put(&mut zip, "OEBPS/nav.xhtml", nav.as_bytes(), deflated)?;

    let identifier = book
        .metadata
        .isbn
        .ebook
        .clone()
        .filter(|s| !s.is_empty())
        .map(|i| format!("urn:isbn:{}", i.replace(['-', ' '], "")))
        .unwrap_or_else(|| format!("urn:folio:{}", book.id.0));
    let manifest_chapters: String = (1..=chapters.len())
        .map(|i| format!("<item id=\"chapter-{i:03}\" href=\"text/chapter-{i:03}.xhtml\" media-type=\"application/xhtml+xml\"/>\n"))
        .collect();
    let spine: String = (1..=chapters.len())
        .map(|i| format!("<itemref idref=\"chapter-{i:03}\"/>\n"))
        .collect();
    let authors: String = book
        .authors
        .iter()
        .map(|a| format!("<dc:creator>{}</dc:creator>\n", escape(&a.name)))
        .collect();
    let description = if book.metadata.description.is_empty() {
        String::new()
    } else {
        format!(
            "<dc:description>{}</dc:description>\n",
            escape(&book.metadata.description)
        )
    };
    let opf = format!(
        "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<package xmlns=\"http://www.idpf.org/2007/opf\" version=\"3.0\" unique-identifier=\"book-id\" xml:lang=\"{lang}\">\n<metadata xmlns:dc=\"http://purl.org/dc/elements/1.1/\">\n<dc:identifier id=\"book-id\">{identifier}</dc:identifier>\n<dc:title>{title}</dc:title>\n{authors}<dc:language>{lang}</dc:language>\n{description}<meta property=\"dcterms:modified\">{modified}</meta>\n<meta property=\"schema:accessMode\">textual</meta>\n<meta property=\"schema:accessibilityFeature\">readingOrder</meta>\n<meta property=\"schema:accessibilitySummary\">The book is reflowable text in reading order.</meta>\n</metadata>\n<manifest>\n<item id=\"nav\" href=\"nav.xhtml\" media-type=\"application/xhtml+xml\" properties=\"nav\"/>\n<item id=\"css\" href=\"css/book.css\" media-type=\"text/css\"/>\n{manifest_chapters}{manifest_images}{manifest_fonts}</manifest>\n<spine>\n{spine}</spine>\n</package>\n",
        lang = escape(&book.language),
        identifier = escape(&identifier),
        title = escape(&book.title),
        modified = crate::time::utc_now(),
    );
    put(&mut zip, "OEBPS/content.opf", opf.as_bytes(), deflated)?;
    let done = zip.finish().map_err(|e| ExportError::Epub(e.to_string()))?;
    Ok(done.into_inner())
}

/// What the reader preview shows: the same chapters and CSS the EPUB ships, and the pictures.
#[derive(Debug, Clone)]
pub struct ReaderPreview {
    pub title: String,
    pub language: String,
    pub css: String,
    pub chapters: Vec<ReflowChapter>,
    /// Each picture by its name in the chapters, with its media type and bytes.
    pub images: Vec<(String, &'static str, Vec<u8>)>,
}

/// The reader preview of `book`: its story as the EPUB tells it, in the EPUB's own CSS. The fonts
/// are the page's to supply, since the preview is shown inside it.
pub fn reader_preview(book: &Book, assets: &dyn Assets) -> Result<ReaderPreview, ExportError> {
    let Reflow { chapters, images } = reflow(book, assets, EpubFlavour::Standard)?;
    let images = images
        .into_iter()
        .filter_map(|(path, name)| {
            assets
                .bytes(&path)
                .map(|bytes| (name.clone(), media_type(&name), bytes))
        })
        .collect();
    Ok(ReaderPreview {
        title: book.title.clone(),
        language: book.language.clone(),
        css: css(book, &BTreeMap::new()),
        chapters,
        images,
    })
}
