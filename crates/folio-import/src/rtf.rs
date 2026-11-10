// Rich Text Format (.rtf): the words of a document as the paragraphs of a story
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

// RTF names no heading or quotation styles that can be told apart from the text, so every
// paragraph comes in as body text with its bold, italic, underlined and struck-through runs; the report says so. Font, colour
// and style tables, pictures, notes and other destinations are skipped.

use folio_model::{join_runs, Run, StoryPara};

use crate::{keep_unusual_looks, ImportReport, Imported};

/// Why an RTF file could not be read.
#[derive(Debug, thiserror::Error)]
pub enum RtfError {
    #[error("That is not a Rich Text (.rtf) file, or it is damaged.")]
    NotRtf,
}

/// Destinations whose contents are not the document's words.
const SKIPPED: &[&str] = &[
    "fonttbl",
    "colortbl",
    "stylesheet",
    "info",
    "pict",
    "footnote",
    "header",
    "footer",
    "headerl",
    "headerr",
    "footerl",
    "footerr",
    "listtable",
    "listoverridetable",
    "generator",
    "object",
    "annotation",
    "private",
];

#[derive(Clone, Default)]
struct State {
    bold: bool,
    italic: bool,
    underline: bool,
    strike: bool,
    /// The size in points, and the letter spacing in points.
    size: Option<f64>,
    spacing: Option<f64>,
    lang: Option<String>,
    /// Where the words link to, inside the result of a hyperlink field.
    href: Option<String>,
    /// Inside the instruction of a field, whose text is read for a hyperlink's address.
    field: bool,
    skip: bool,
    /// Characters a `\uN` escape still has to step over (`\ucN`).
    skip_chars: u32,
}

/// The language tag of a Windows language identifier (`\\lang1036`), for the ones Folio can check.
fn language_of(id: i32) -> Option<String> {
    let tag = match id {
        1033 => "en-US",
        2057 => "en-GB",
        4105 => "en-CA",
        3081 => "en-AU",
        1036 => "fr-FR",
        3084 => "fr-CA",
        2060 => "fr-BE",
        1031 => "de-DE",
        1034 | 3082 => "es-ES",
        1046 => "pt-BR",
        2070 => "pt-PT",
        1043 => "nl-NL",
        1040 => "it-IT",
        1053 => "sv-SE",
        _ => return None,
    };
    Some(tag.to_owned())
}

/// Reads an `.rtf` held in `bytes` into a story.
pub fn import_rtf(bytes: &[u8]) -> Result<Imported, RtfError> {
    if !bytes.starts_with(b"{\\rtf") {
        return Err(RtfError::NotRtf);
    }
    let mut report = ImportReport::default();
    let mut story: Vec<StoryPara> = Vec::new();
    let mut runs: Vec<Run> = Vec::new();
    let mut stack: Vec<State> = Vec::new();
    let mut state = State::default();
    let mut uc = 1u32;
    let mut i = 0;
    let mut pending = String::new();
    // The instruction of the field being read, and the address of the hyperlink it made.
    let mut field_text = String::new();
    let mut field_href: Option<String> = None;
    let note = |report: &mut ImportReport, text: &str| {
        if !report.notes.iter().any(|n| n == text) {
            report.notes.push(text.to_owned());
        }
    };
    let flush_text = |pending: &mut String, runs: &mut Vec<Run>, state: &State| {
        if !pending.is_empty() {
            let run = Run {
                text: std::mem::take(pending),
                bold: state.bold,
                italic: state.italic,
                underline: state.underline,
                strike: state.strike,
                size: state.size,
                lang: state.lang.clone(),
                href: state.href.clone(),
                tracking: state.spacing.and_then(|pt| {
                    crate::tracking_of(pt, state.size.unwrap_or(crate::REFERENCE_SIZE))
                }),
                ..Default::default()
            };
            *runs = join_runs(runs, &[run]);
        }
    };
    let end_paragraph = |runs: &mut Vec<Run>, story: &mut Vec<StoryPara>| {
        if runs.iter().any(|r| !r.text.trim().is_empty()) {
            story.push(StoryPara {
                style: None,
                inline: std::mem::take(runs),
                look: None,
            });
        } else {
            runs.clear();
        }
    };
    while i < bytes.len() {
        let b = bytes[i];
        match b {
            b'{' => {
                flush_text(&mut pending, &mut runs, &state);
                stack.push(state.clone());
                i += 1;
            }
            b'}' => {
                flush_text(&mut pending, &mut runs, &state);
                let closing = state.field;
                state = stack.pop().unwrap_or_default();
                // The end of a field's instruction: a hyperlink's address is in it.
                if closing && !state.field {
                    field_href = hyperlink_of(&field_text);
                    if field_href.is_none() && field_text.contains("HYPERLINK") {
                        note(
                            &mut report,
                            "Links that Folio cannot follow were left out; their words were kept.",
                        );
                    }
                    field_text.clear();
                }
                i += 1;
            }
            b'\r' | b'\n' => i += 1,
            b'\\' => {
                i += 1;
                let Some(&next) = bytes.get(i) else { break };
                match next {
                    b'\\' | b'{' | b'}' => {
                        if !state.skip {
                            pending.push(next as char);
                        }
                        i += 1;
                    }
                    b'\'' => {
                        let hex = bytes
                            .get(i + 1..i + 3)
                            .and_then(|h| std::str::from_utf8(h).ok());
                        if let Some(value) = hex.and_then(|h| u8::from_str_radix(h, 16).ok()) {
                            if state.skip_chars > 0 {
                                state.skip_chars -= 1;
                            } else if !state.skip {
                                // Windows-1252, the code page Word writes by default.
                                pending.push(char::from(value));
                            }
                        }
                        i += 3;
                    }
                    b'*' => {
                        // An ignorable destination: everything up to the group's end is skipped.
                        state.skip = true;
                        i += 1;
                    }
                    b'~' => {
                        if !state.skip {
                            pending.push('\u{a0}');
                        }
                        i += 1;
                    }
                    b'-' | b'_' => i += 1,
                    c if c.is_ascii_alphabetic() => {
                        let start = i;
                        while bytes.get(i).is_some_and(u8::is_ascii_alphabetic) {
                            i += 1;
                        }
                        let word = std::str::from_utf8(&bytes[start..i]).unwrap_or("");
                        let negative = bytes.get(i) == Some(&b'-');
                        let digits_from = i + usize::from(negative);
                        let mut end = digits_from;
                        while bytes.get(end).is_some_and(u8::is_ascii_digit) {
                            end += 1;
                        }
                        let param: Option<i32> = std::str::from_utf8(&bytes[digits_from..end])
                            .ok()
                            .and_then(|d| d.parse().ok())
                            .map(|n: i32| if negative { -n } else { n });
                        i = end;
                        // One space after a control word belongs to it.
                        if bytes.get(i) == Some(&b' ') {
                            i += 1;
                        }
                        if word == "fldinst" {
                            state.field = true;
                            state.skip = true;
                            field_text.clear();
                            continue;
                        }
                        // The words a field shows: a hyperlink's, linked.
                        if word == "fldrslt" {
                            state.href = field_href.take();
                            continue;
                        }
                        if SKIPPED.contains(&word) {
                            if word == "pict" {
                                note(&mut report, "Pictures were left out. Add them to the book from the Insert tab.");
                            } else if word == "footnote" {
                                note(&mut report, "Footnotes and endnotes were left out.");
                            }
                            state.skip = true;
                            continue;
                        }
                        if state.skip {
                            continue;
                        }
                        match word {
                            "par" | "sect" | "page" => {
                                flush_text(&mut pending, &mut runs, &state);
                                end_paragraph(&mut runs, &mut story);
                            }
                            "line" | "tab" => pending.push(' '),
                            "b" => {
                                flush_text(&mut pending, &mut runs, &state);
                                state.bold = param != Some(0);
                            }
                            "i" => {
                                flush_text(&mut pending, &mut runs, &state);
                                state.italic = param != Some(0);
                            }
                            "ul" | "uld" | "uldash" | "uldashd" | "uldashdd" | "uldb" | "ulth"
                            | "ulwave" | "ulw" => {
                                flush_text(&mut pending, &mut runs, &state);
                                state.underline = param != Some(0);
                            }
                            "ulnone" => {
                                flush_text(&mut pending, &mut runs, &state);
                                state.underline = false;
                            }
                            "strike" | "striked" => {
                                flush_text(&mut pending, &mut runs, &state);
                                state.strike = param != Some(0);
                            }
                            // Half-points, and (`\expndtw`) twentieths of a point.
                            "fs" => {
                                flush_text(&mut pending, &mut runs, &state);
                                state.size = param.filter(|n| *n > 0).map(|n| f64::from(n) / 2.0);
                            }
                            "expndtw" => {
                                flush_text(&mut pending, &mut runs, &state);
                                state.spacing =
                                    param.filter(|n| *n != 0).map(|n| f64::from(n) / 20.0);
                            }
                            "lang" => {
                                flush_text(&mut pending, &mut runs, &state);
                                state.lang = param.and_then(language_of);
                            }
                            "plain" => {
                                flush_text(&mut pending, &mut runs, &state);
                                state = State {
                                    skip: state.skip,
                                    skip_chars: state.skip_chars,
                                    ..State::default()
                                };
                            }
                            "uc" => uc = param.unwrap_or(1).max(0) as u32,
                            "u" => {
                                if let Some(n) = param {
                                    let code = if n < 0 { (n + 65536) as u32 } else { n as u32 };
                                    if let Some(c) = char::from_u32(code) {
                                        pending.push(c);
                                    }
                                    state.skip_chars = uc;
                                }
                            }
                            "emdash" => pending.push('\u{2014}'),
                            "endash" => pending.push('\u{2013}'),
                            "lquote" => pending.push('\u{2018}'),
                            "rquote" => pending.push('\u{2019}'),
                            "ldblquote" => pending.push('\u{201c}'),
                            "rdblquote" => pending.push('\u{201d}'),
                            "bullet" => pending.push('\u{2022}'),
                            "trowd" | "cell" => note(&mut report, "Tables were left out."),
                            _ => {}
                        }
                    }
                    _ => i += 1,
                }
            }
            _ => {
                if state.field {
                    field_text.push(char::from(b));
                    i += 1;
                } else if state.skip {
                    i += 1;
                } else if state.skip_chars > 0 {
                    state.skip_chars -= 1;
                    i += 1;
                } else {
                    // The bytes of a character that is not ASCII are kept whole.
                    let rest = &bytes[i..];
                    let len = std::str::from_utf8(rest)
                        .map(|s| s.chars().next().map_or(1, char::len_utf8))
                        .unwrap_or(1);
                    pending.push_str(&String::from_utf8_lossy(&bytes[i..i + len]));
                    i += len;
                }
            }
        }
    }
    flush_text(&mut pending, &mut runs, &state);
    end_paragraph(&mut runs, &mut story);
    note(
        &mut report,
        "Headings and quotations in a Rich Text file are not told apart; every paragraph came in as body text. Use Word (.docx) or OpenDocument (.odt) to keep them.",
    );
    keep_unusual_looks(&mut story);
    report.paragraphs = story.len() as u32;
    Ok(Imported { story, report })
}

/// The address of the hyperlink a field's instruction (` HYPERLINK "https://example.org" `) names.
/// A link to a place in the document (`\\l`) has none.
fn hyperlink_of(instruction: &str) -> Option<String> {
    let rest = instruction.trim().strip_prefix("HYPERLINK")?;
    let rest = rest.trim_start();
    if rest.starts_with("\\l") {
        return None;
    }
    let start = rest.find('"')? + 1;
    let end = start + rest[start..].find('"')?;
    folio_model::clean_href(&rest[start..end])
}
