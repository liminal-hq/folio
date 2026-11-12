// Word (.docx): the words of `word/document.xml` as the paragraphs of a story
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

// A paragraph's `Heading 1` becomes a chapter title, other headings subheadings, quotation styles
// the epigraph style and list items a paragraph that starts with a dash; bold, italic, underlined and struck-through runs keep
// their emphasis. Tracked insertions are accepted and deletions dropped. Pictures, tables,
// footnotes and comments are left out, and the report says so. A hyperlink to an address Folio
// follows keeps its address; one to a place in the document keeps only its words.

use std::collections::HashMap;
use std::io::{Cursor, Read};

use folio_model::{join_runs, Run, StoryPara, StyleId};
use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader;

use crate::{ImportReport, Imported, StyleMap, Tally};

/// Why a Word file could not be read.
#[derive(Debug, thiserror::Error)]
pub enum DocxError {
    #[error("That is not a Word (.docx) file, or it is damaged.")]
    NotDocx,
}

fn attr(e: &BytesStart, name: &[u8]) -> Option<String> {
    e.attributes()
        .flatten()
        .find(|a| a.key.local_name().as_ref() == name)
        .and_then(|a| a.unescape_value().ok().map(|v| v.into_owned()))
}

/// The style a Word paragraph style name maps to, if it is one Folio has a use for.
fn style_for(word: &str) -> Option<&'static str> {
    let key: String = word
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect::<String>()
        .to_lowercase();
    match key.as_str() {
        "heading1" | "title" => Some("h1"),
        "heading2" | "heading3" | "heading4" | "subtitle" => Some("h2"),
        "quote" | "intensequote" | "blockquote" => Some("epigraph"),
        _ => None,
    }
}

#[derive(Default)]
struct Paragraph {
    /// The style's name in the document, and the Folio style Folio would give it unasked.
    source: Option<String>,
    style: Option<&'static str>,
    list: bool,
    runs: Vec<Run>,
}

/// Reads a `.docx` held in `bytes` into a story.
pub fn import_docx(bytes: &[u8]) -> Result<Imported, DocxError> {
    import_docx_with(bytes, &StyleMap::new())
}

/// Reads a `.docx` with the person's choices of Folio style for the document's styles.
pub fn import_docx_with(bytes: &[u8], map: &StyleMap) -> Result<Imported, DocxError> {
    let mut zip = zip::ZipArchive::new(Cursor::new(bytes)).map_err(|_| DocxError::NotDocx)?;
    let mut xml = String::new();
    zip.by_name("word/document.xml")
        .map_err(|_| DocxError::NotDocx)?
        .read_to_string(&mut xml)
        .map_err(|_| DocxError::NotDocx)?;
    let has_comments = zip.by_name("word/comments.xml").is_ok();
    // Where each link of the document goes: its relationships name the addresses.
    let mut rels = String::new();
    if let Ok(mut file) = zip.by_name("word/_rels/document.xml.rels") {
        let _ = file.read_to_string(&mut rels);
    }
    Ok(read_document(&xml, has_comments, map, &link_targets(&rels)))
}

/// The addresses a document's relationships name, by relationship id.
fn link_targets(xml: &str) -> HashMap<String, String> {
    let mut out = HashMap::new();
    let mut reader = Reader::from_str(xml);
    while let Ok(event) = reader.read_event() {
        match event {
            Event::Eof => break,
            Event::Start(e) | Event::Empty(e) if e.local_name().as_ref() == b"Relationship" => {
                if let (Some(id), Some(target)) = (attr(&e, b"Id"), attr(&e, b"Target")) {
                    if attr(&e, b"TargetMode").as_deref() == Some("External") {
                        out.insert(id, target);
                    }
                }
            }
            _ => {}
        }
    }
    out
}

fn read_document(
    xml: &str,
    has_comments: bool,
    map: &StyleMap,
    links: &HashMap<String, String>,
) -> Imported {
    let mut tally = Tally::default();
    let mut reader = Reader::from_str(xml);
    let mut report = ImportReport::default();
    let mut note = |report: &mut ImportReport, text: &str| {
        if !report.notes.iter().any(|n| n == text) {
            report.notes.push(text.to_owned());
        }
    };
    if has_comments {
        note(&mut report, "Comments were left out.");
    }
    let mut story: Vec<StoryPara> = Vec::new();
    let mut para: Option<Paragraph> = None;
    let (mut look, mut in_text, mut deleted, mut in_run_props) =
        (Look::default(), false, 0u32, false);
    let mut skipping = 0u32;
    let mut in_table = 0u32;
    // The address of the link being read, when the words are in one.
    let mut link: Option<String> = None;
    loop {
        let event = match reader.read_event() {
            Ok(Event::Eof) | Err(_) => break,
            Ok(event) => event,
        };
        // Pictures and text boxes are left out whole.
        match &event {
            Event::Start(e)
                if matches!(
                    e.local_name().as_ref(),
                    b"drawing" | b"pict" | b"footnoteReference" | b"endnoteReference"
                ) =>
            {
                if e.local_name().as_ref() == b"drawing" || e.local_name().as_ref() == b"pict" {
                    note(
                        &mut report,
                        "Pictures were left out. Add them to the book from the Insert tab.",
                    );
                } else {
                    note(&mut report, "Footnotes and endnotes were left out.");
                }
                skipping += 1;
                continue;
            }
            Event::Empty(e)
                if matches!(
                    e.local_name().as_ref(),
                    b"footnoteReference" | b"endnoteReference"
                ) =>
            {
                note(&mut report, "Footnotes and endnotes were left out.");
                continue;
            }
            Event::End(e)
                if skipping > 0
                    && matches!(
                        e.local_name().as_ref(),
                        b"drawing" | b"pict" | b"footnoteReference" | b"endnoteReference"
                    ) =>
            {
                skipping -= 1;
                continue;
            }
            _ if skipping > 0 => continue,
            _ => {}
        }
        match event {
            Event::Start(e) => match e.local_name().as_ref() {
                b"tbl" => {
                    in_table += 1;
                    note(&mut report, "Tables were left out.");
                }
                b"p" if in_table == 0 => para = Some(Paragraph::default()),
                b"pStyle" => {}
                b"rPr" => in_run_props = true,
                b"r" => {
                    look = Look::default();
                }
                b"t" => in_text = true,
                b"del" => deleted += 1,
                b"hyperlink" => {
                    // A link to a place in the document (`w:anchor`) has nothing to go to here.
                    link = attr(&e, b"id")
                        .and_then(|id| links.get(&id))
                        .and_then(|target| folio_model::clean_href(target));
                    if link.is_none() {
                        note(
                            &mut report,
                            "Links that Folio cannot follow were left out; their words were kept.",
                        );
                    }
                }
                _ => {}
            },
            Event::Empty(e) => match e.local_name().as_ref() {
                b"pStyle" => {
                    if let (Some(p), Some(v)) = (para.as_mut(), attr(&e, b"val")) {
                        p.style = style_for(&v);
                        p.source = Some(v);
                    }
                }
                b"numId" if attr(&e, b"val").is_some_and(|v| v != "0") => {
                    if let Some(p) = para.as_mut() {
                        p.list = true;
                    }
                }
                b"b" if in_run_props => {
                    look.bold = attr(&e, b"val").is_none_or(|v| v != "0" && v != "false")
                }
                b"i" if in_run_props => {
                    look.italic = attr(&e, b"val").is_none_or(|v| v != "0" && v != "false")
                }
                // Sizes are in half-points and letter spacing in twentieths of a point.
                b"sz" if in_run_props => {
                    look.size = attr(&e, b"val")
                        .and_then(|v| v.parse::<f64>().ok())
                        .map(|half| half / 2.0)
                        .filter(|pt| *pt > 0.0)
                }
                b"spacing" if in_run_props => {
                    look.spacing = attr(&e, b"val")
                        .and_then(|v| v.parse::<f64>().ok())
                        .map(|twentieths| twentieths / 20.0)
                }
                // `w:lang` names a language tag.
                b"lang" if in_run_props => {
                    look.lang = attr(&e, b"val")
                        .as_deref()
                        .and_then(folio_model::clean_lang)
                }
                // `w:u` names a line style; `none` (or `0`) is no line.
                b"u" if in_run_props => {
                    look.underline = attr(&e, b"val").is_none_or(|v| v != "none" && v != "0")
                }
                b"strike" | b"dstrike" if in_run_props => {
                    look.strike = attr(&e, b"val").is_none_or(|v| v != "0" && v != "false")
                }
                b"tab" | b"br" if in_table == 0 && deleted == 0 => {
                    if let Some(p) = para.as_mut() {
                        p.runs = join_runs(&p.runs, &[run(" ", &look)]);
                    }
                }
                _ => {}
            },
            Event::Text(t) if in_text && deleted == 0 && in_table == 0 => {
                if let (Some(p), Ok(text)) = (para.as_mut(), t.unescape()) {
                    let mut linked = look.clone();
                    linked.href = link.clone();
                    p.runs = join_runs(&p.runs, &[run(&text, &linked)]);
                }
            }
            Event::End(e) => match e.local_name().as_ref() {
                b"t" => in_text = false,
                b"rPr" => in_run_props = false,
                b"hyperlink" => link = None,
                b"del" => deleted = deleted.saturating_sub(1),
                b"tbl" => in_table = in_table.saturating_sub(1),
                b"p" => {
                    if let Some(p) = para.take() {
                        finish(p, &mut story, &mut report, &mut note, &mut tally, map);
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }
    crate::keep_unusual_looks(&mut story);
    report.paragraphs = story.len() as u32;
    report.styles = tally.into_styles();
    Imported { story, report }
}

/// The look of the run being read.
#[derive(Clone, Default)]
struct Look {
    bold: bool,
    italic: bool,
    underline: bool,
    strike: bool,
    /// The size in points, and the letter spacing in points.
    size: Option<f64>,
    spacing: Option<f64>,
    lang: Option<String>,
    href: Option<String>,
}

fn run(text: &str, look: &Look) -> Run {
    Run {
        text: text.to_owned(),
        bold: look.bold,
        italic: look.italic,
        underline: look.underline,
        strike: look.strike,
        size: look.size,
        lang: look.lang.clone(),
        href: look.href.clone(),
        tracking: look
            .spacing
            .and_then(|pt| crate::tracking_of(pt, look.size.unwrap_or(crate::REFERENCE_SIZE))),
        ..Default::default()
    }
}

fn finish(
    para: Paragraph,
    story: &mut Vec<StoryPara>,
    report: &mut ImportReport,
    note: &mut impl FnMut(&mut ImportReport, &str),
    tally: &mut Tally,
    map: &StyleMap,
) {
    if !para.runs.iter().any(|r| !r.text.trim().is_empty()) {
        return;
    }
    let words: String = para.runs.iter().map(|r| r.text.as_str()).collect();
    let name = para.source.as_deref().unwrap_or("Normal");
    let style = tally.see(name, para.style, map, words.trim());
    let mut runs = para.runs;
    if para.list {
        runs.insert(0, run("\u{2013} ", &Look::default()));
        note(
            report,
            "List items became paragraphs that start with a dash.",
        );
    }
    if matches!(style.as_deref(), Some("h1" | "h2")) {
        report.headings += 1;
    }
    story.push(StoryPara {
        style: style.map(StyleId),
        inline: runs,
        look: None,
    });
}
