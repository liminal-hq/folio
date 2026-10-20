// OpenDocument text (.odt): the words of `content.xml` as the paragraphs of a story
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

// Headings (`text:h`) become chapter titles and subheadings by their outline level, quotation
// paragraph styles the epigraph style and list items a paragraph that starts with a dash. Spans
// are bold or italic as their automatic style says. Tables, pictures, notes, comments and the
// list of tracked changes are left out, and the report says so.

use std::collections::BTreeMap;
use std::io::{Cursor, Read};

use folio_model::{join_runs, Run, StoryPara, StyleId};
use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader;

use crate::{ImportReport, Imported, StyleMap, Tally};

/// Why an OpenDocument file could not be read.
#[derive(Debug, thiserror::Error)]
pub enum OdtError {
    #[error("That is not an OpenDocument text (.odt) file, or it is damaged.")]
    NotOdt,
}

fn attr(e: &BytesStart, name: &[u8]) -> Option<String> {
    e.attributes()
        .flatten()
        .find(|a| a.key.local_name().as_ref() == name)
        .and_then(|a| a.unescape_value().ok().map(|v| v.into_owned()))
}

#[derive(Clone, Copy, Default)]
struct Look {
    bold: bool,
    italic: bool,
}

fn run(text: &str, look: Look) -> Run {
    Run {
        text: text.to_owned(),
        style: None,
        bold: look.bold,
        italic: look.italic,
        font: None,
        colour: None,
    }
}

/// Whether a paragraph style's name is one of the quotation styles.
fn is_quote(name: &str) -> bool {
    let key = name.to_lowercase();
    key.contains("quot")
}

/// Reads an `.odt` held in `bytes` into a story.
pub fn import_odt(bytes: &[u8]) -> Result<Imported, OdtError> {
    import_odt_with(bytes, &StyleMap::new())
}

/// Reads an `.odt` with the person's choices of Folio style for the document's styles.
pub fn import_odt_with(bytes: &[u8], map: &StyleMap) -> Result<Imported, OdtError> {
    let mut zip = zip::ZipArchive::new(Cursor::new(bytes)).map_err(|_| OdtError::NotOdt)?;
    let mut xml = String::new();
    zip.by_name("content.xml")
        .map_err(|_| OdtError::NotOdt)?
        .read_to_string(&mut xml)
        .map_err(|_| OdtError::NotOdt)?;
    Ok(read_content(&xml, map))
}

fn note(report: &mut ImportReport, text: &str) {
    if !report.notes.iter().any(|n| n == text) {
        report.notes.push(text.to_owned());
    }
}

fn read_content(xml: &str, map: &StyleMap) -> Imported {
    let mut tally = Tally::default();
    let mut source = String::new();
    let mut reader = Reader::from_str(xml);
    let mut report = ImportReport::default();
    // Automatic text styles that are bold or italic, by name.
    let mut looks: BTreeMap<String, Look> = BTreeMap::new();
    let mut defining: Option<String> = None;
    let mut story: Vec<StoryPara> = Vec::new();
    let mut runs: Vec<Run> = Vec::new();
    let mut style: Option<&'static str> = None;
    let mut list = 0u32;
    let mut in_para = false;
    let mut spans: Vec<Look> = Vec::new();
    let mut skipping: Vec<Vec<u8>> = Vec::new();
    loop {
        let event = match reader.read_event() {
            Ok(Event::Eof) | Err(_) => break,
            Ok(e) => e,
        };
        if let Some(open) = skipping.last() {
            match &event {
                Event::Start(e) if e.name().as_ref() == open.as_slice() => {
                    skipping.push(open.clone());
                }
                Event::End(e) if e.name().as_ref() == open.as_slice() => {
                    skipping.pop();
                }
                _ => {}
            }
            continue;
        }
        match event {
            Event::Start(e) => {
                let name = e.local_name();
                match name.as_ref() {
                    b"style" => defining = attr(&e, b"name"),
                    b"text-properties" => {}
                    b"table" | b"frame" | b"note" | b"annotation" | b"tracked-changes" => {
                        note(
                            &mut report,
                            match name.as_ref() {
                                b"table" => "Tables were left out.",
                                b"frame" => {
                                    "Pictures were left out. Add them to the book from the Insert tab."
                                }
                                b"note" => "Footnotes and endnotes were left out.",
                                b"annotation" => "Comments were left out.",
                                _ => "Tracked changes were not applied; the words are as they now stand.",
                            },
                        );
                        skipping.push(e.name().as_ref().to_vec());
                    }
                    b"list" => list += 1,
                    b"h" => {
                        in_para = true;
                        runs.clear();
                        let level = attr(&e, b"outline-level").unwrap_or_else(|| "1".into());
                        style = Some(if level == "1" { "h1" } else { "h2" });
                        source = format!("Heading {level}");
                    }
                    b"p" => {
                        in_para = true;
                        runs.clear();
                        let named = attr(&e, b"style-name");
                        style = named.as_deref().filter(|n| is_quote(n)).map(|_| "epigraph");
                        source = named.unwrap_or_else(|| "Standard".into());
                    }
                    b"span" if in_para => {
                        let look = attr(&e, b"style-name")
                            .and_then(|n| looks.get(&n).copied())
                            .unwrap_or_default();
                        let outer = spans.last().copied().unwrap_or_default();
                        spans.push(Look {
                            bold: outer.bold || look.bold,
                            italic: outer.italic || look.italic,
                        });
                    }
                    _ => {}
                }
            }
            Event::Empty(e) => match e.local_name().as_ref() {
                b"text-properties" => {
                    if let Some(name) = &defining {
                        let weight = attr(&e, b"font-weight");
                        let slant = attr(&e, b"font-style");
                        looks.insert(
                            name.clone(),
                            Look {
                                bold: weight.as_deref() == Some("bold"),
                                italic: slant.as_deref() == Some("italic"),
                            },
                        );
                    }
                }
                b"s" | b"tab" | b"line-break" if in_para => {
                    let look = spans.last().copied().unwrap_or_default();
                    runs = join_runs(&runs, &[run(" ", look)]);
                }
                b"frame" => note(
                    &mut report,
                    "Pictures were left out. Add them to the book from the Insert tab.",
                ),
                _ => {}
            },
            Event::Text(t) if in_para => {
                if let Ok(text) = t.unescape() {
                    let look = spans.last().copied().unwrap_or_default();
                    runs = join_runs(&runs, &[run(&text, look)]);
                }
            }
            Event::End(e) => match e.local_name().as_ref() {
                b"style" => defining = None,
                b"list" => list = list.saturating_sub(1),
                b"span" => {
                    spans.pop();
                }
                b"h" | b"p" if in_para => {
                    in_para = false;
                    if runs.iter().any(|r| !r.text.trim().is_empty()) {
                        let mut inline = std::mem::take(&mut runs);
                        let words: String = inline.iter().map(|r| r.text.as_str()).collect();
                        let applied = tally.see(&source, style, map, words.trim());
                        if matches!(applied.as_deref(), Some("h1" | "h2")) {
                            report.headings += 1;
                        }
                        if list > 0 && applied.is_none() {
                            inline.insert(0, run("\u{2013} ", Look::default()));
                            note(
                                &mut report,
                                "List items became paragraphs that start with a dash.",
                            );
                        }
                        story.push(StoryPara {
                            style: applied.map(StyleId),
                            inline,
                        });
                    }
                    runs.clear();
                    style = None;
                }
                _ => {}
            },
            _ => {}
        }
    }
    report.paragraphs = story.len() as u32;
    report.styles = tally.into_styles();
    Imported { story, report }
}
