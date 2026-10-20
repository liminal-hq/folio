// Rich Text Format (.rtf): the words of a document as the paragraphs of a story
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

// RTF names no heading or quotation styles that can be told apart from the text, so every
// paragraph comes in as body text with its bold and italic runs; the report says so. Font, colour
// and style tables, pictures, notes and other destinations are skipped.

use folio_model::{join_runs, Run, StoryPara};

use crate::{ImportReport, Imported};

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

#[derive(Clone, Copy, Default)]
struct State {
    bold: bool,
    italic: bool,
    skip: bool,
    /// Characters a `\uN` escape still has to step over (`\ucN`).
    skip_chars: u32,
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
    let mut note = |report: &mut ImportReport, text: &str| {
        if !report.notes.iter().any(|n| n == text) {
            report.notes.push(text.to_owned());
        }
    };
    let flush_text = |pending: &mut String, runs: &mut Vec<Run>, state: &State| {
        if !pending.is_empty() {
            let run = Run {
                text: std::mem::take(pending),
                style: None,
                bold: state.bold,
                italic: state.italic,
                font: None,
                colour: None,
            };
            *runs = join_runs(runs, &[run]);
        }
    };
    let end_paragraph = |runs: &mut Vec<Run>, story: &mut Vec<StoryPara>| {
        if runs.iter().any(|r| !r.text.trim().is_empty()) {
            story.push(StoryPara {
                style: None,
                inline: std::mem::take(runs),
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
                stack.push(state);
                i += 1;
            }
            b'}' => {
                flush_text(&mut pending, &mut runs, &state);
                state = stack.pop().unwrap_or_default();
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
                if state.skip {
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
    report.paragraphs = story.len() as u32;
    Ok(Imported { story, report })
}
