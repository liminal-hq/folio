// Word (.docx): the words of `word/document.xml` as the paragraphs of a story
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

// A paragraph's `Heading 1` becomes a chapter title, other headings subheadings, quotation styles
// the epigraph style and list items a paragraph that starts with a dash; bold and italic runs keep
// their emphasis. Tracked insertions are accepted and deletions dropped. Pictures, tables,
// footnotes and comments are left out, and the report says so.

use std::io::{Cursor, Read};

use folio_model::{join_runs, Run, StoryPara, StyleId};
use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader;

use crate::{ImportReport, Imported};

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
    style: Option<&'static str>,
    list: bool,
    runs: Vec<Run>,
}

/// Reads a `.docx` held in `bytes` into a story.
pub fn import_docx(bytes: &[u8]) -> Result<Imported, DocxError> {
    let mut zip = zip::ZipArchive::new(Cursor::new(bytes)).map_err(|_| DocxError::NotDocx)?;
    let mut xml = String::new();
    zip.by_name("word/document.xml")
        .map_err(|_| DocxError::NotDocx)?
        .read_to_string(&mut xml)
        .map_err(|_| DocxError::NotDocx)?;
    let has_comments = zip.by_name("word/comments.xml").is_ok();
    Ok(read_document(&xml, has_comments))
}

fn read_document(xml: &str, has_comments: bool) -> Imported {
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
    let (mut bold, mut italic, mut in_text, mut deleted, mut in_run_props) =
        (false, false, false, 0u32, false);
    let mut skipping = 0u32;
    let mut in_table = 0u32;
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
                    bold = false;
                    italic = false;
                }
                b"t" => in_text = true,
                b"del" => deleted += 1,
                _ => {}
            },
            Event::Empty(e) => match e.local_name().as_ref() {
                b"pStyle" => {
                    if let (Some(p), Some(v)) = (para.as_mut(), attr(&e, b"val")) {
                        p.style = style_for(&v);
                    }
                }
                b"numId" if attr(&e, b"val").is_some_and(|v| v != "0") => {
                    if let Some(p) = para.as_mut() {
                        p.list = true;
                    }
                }
                b"b" if in_run_props => {
                    bold = attr(&e, b"val").is_none_or(|v| v != "0" && v != "false")
                }
                b"i" if in_run_props => {
                    italic = attr(&e, b"val").is_none_or(|v| v != "0" && v != "false")
                }
                b"tab" | b"br" if in_table == 0 && deleted == 0 => {
                    if let Some(p) = para.as_mut() {
                        p.runs = join_runs(&p.runs, &[run(" ", bold, italic)]);
                    }
                }
                _ => {}
            },
            Event::Text(t) if in_text && deleted == 0 && in_table == 0 => {
                if let (Some(p), Ok(text)) = (para.as_mut(), t.unescape()) {
                    p.runs = join_runs(&p.runs, &[run(&text, bold, italic)]);
                }
            }
            Event::End(e) => match e.local_name().as_ref() {
                b"t" => in_text = false,
                b"rPr" => in_run_props = false,
                b"del" => deleted = deleted.saturating_sub(1),
                b"tbl" => in_table = in_table.saturating_sub(1),
                b"p" => {
                    if let Some(p) = para.take() {
                        finish(p, &mut story, &mut report, &mut note);
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }
    report.paragraphs = story.len() as u32;
    Imported { story, report }
}

fn run(text: &str, bold: bool, italic: bool) -> Run {
    Run {
        text: text.to_owned(),
        style: None,
        bold,
        italic,
        font: None,
        colour: None,
    }
}

fn finish(
    para: Paragraph,
    story: &mut Vec<StoryPara>,
    report: &mut ImportReport,
    note: &mut impl FnMut(&mut ImportReport, &str),
) {
    if !para.runs.iter().any(|r| !r.text.trim().is_empty()) {
        return;
    }
    let mut runs = para.runs;
    if para.list {
        runs.insert(0, run("\u{2013} ", false, false));
        note(
            report,
            "List items became paragraphs that start with a dash.",
        );
    }
    if para.style == Some("h1") || para.style == Some("h2") {
        report.headings += 1;
    }
    story.push(StoryPara {
        style: para.style.map(|s| StyleId(s.to_owned())),
        inline: runs,
    });
}
