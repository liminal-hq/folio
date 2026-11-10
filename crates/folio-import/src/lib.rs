// Importers: words with structure, brought in as the paragraphs of a story
//
// Pure Rust with no Tauri. `import_markdown` reads Markdown (and so plain text, whose blank lines
// are paragraphs) into `StoryPara`s and a report of what came across and what did not.
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

mod docx;
mod mapping;
mod odt;
mod rtf;

pub use docx::{import_docx, import_docx_with, DocxError};
pub use mapping::{SourceStyle, StyleMap, Tally, MAPPABLE};
pub use odt::{import_odt, import_odt_with, OdtError};
pub use rtf::{import_rtf, RtfError};

use folio_model::{join_runs, Run, StoryPara, StyleId};
use folio_model::{smarten_with, QuoteStyle, SmartRules};
use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag, TagEnd};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// What an import brought in, and what it had to leave out or approximate.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub struct ImportReport {
    /// Paragraphs in the story, headings and scene breaks included.
    pub paragraphs: u32,
    /// How many of them are headings.
    pub headings: u32,
    /// How many are scene breaks (`***`).
    pub scene_breaks: u32,
    /// What was left out or approximated, each said once in plain words.
    pub notes: Vec<String>,
    /// The paragraph styles the document used, and the Folio style each was given, when the
    /// document names them (Word and OpenDocument do).
    #[serde(default)]
    pub styles: Vec<SourceStyle>,
}

/// The story read from a document, and the report on it.
#[derive(Debug, Clone, PartialEq)]
pub struct Imported {
    pub story: Vec<StoryPara>,
    pub report: ImportReport,
}

/// The style a first-level heading, other headings, quotations and scene breaks are given.
const CHAPTER: &str = "h1";
const SUBHEADING: &str = "h2";
const QUOTE: &str = "epigraph";
const BREAK: &str = "ornament";

/// The text of a scene break as it is set.
const SCENE_BREAK: &str = "* * *";

struct Reader {
    story: Vec<StoryPara>,
    current: Vec<Run>,
    style: Option<&'static str>,
    bold: u32,
    italic: u32,
    quote: u32,
    in_item: bool,
    report: ImportReport,
    /// How quotes are set when Markdown's straight ones are made curly; `None` leaves them be.
    smart: Option<QuoteStyle>,
}

/// The words of `runs`, run together.
fn plain_text(runs: &[Run]) -> String {
    runs.iter().map(|r| r.text.as_str()).collect()
}

impl Reader {
    fn note(&mut self, text: &str) {
        if !self.report.notes.iter().any(|n| n == text) {
            self.report.notes.push(text.to_owned());
        }
    }

    fn push(&mut self, text: &str) {
        if text.is_empty() {
            return;
        }
        let run = Run {
            text: text.to_owned(),
            style: None,
            bold: self.bold > 0,
            italic: self.italic > 0,
            font: None,
            colour: None,
        };
        self.current = join_runs(&self.current, &[run]);
    }

    /// Adds words that were written as text, set with smart punctuation by what comes before them
    /// in the paragraph. Words in `code` spans are pushed as they are.
    fn push_text(&mut self, text: &str) {
        let Some(style) = self.smart else {
            return self.push(text);
        };
        // A rule may reach back into the words before (a second hyphen, a third dot).
        let before = plain_text(&self.current);
        let set = smarten_with(&before, text, style, SmartRules::ALL);
        trim_end(&mut self.current, set.rewind);
        self.push(&set.text);
    }

    /// Ends the paragraph being read, if it has words.
    fn flush(&mut self) {
        let has_words = self.current.iter().any(|r| !r.text.trim().is_empty());
        if has_words {
            let style = self.style.or((self.quote > 0).then_some(QUOTE));
            self.story.push(StoryPara {
                style: style.map(|s| StyleId(s.to_owned())),
                inline: std::mem::take(&mut self.current),
            });
        } else {
            self.current.clear();
        }
        self.style = None;
    }
}

/// Takes `count` characters off the end of `runs`, dropping runs left empty.
fn trim_end(runs: &mut Vec<Run>, mut count: usize) {
    while count > 0 {
        let Some(last) = runs.last_mut() else { return };
        let have = last.text.chars().count();
        if have <= count {
            count -= have;
            runs.pop();
        } else {
            last.text = last.text.chars().take(have - count).collect();
            count = 0;
        }
    }
}

/// Sets the quotes, dashes and ellipses of every paragraph of `story` the way a book sets them,
/// in the quote style given. The runs keep their looks; a rule may reach back into the run before.
pub fn smarten_story(story: &mut [StoryPara], style: QuoteStyle) {
    for para in story {
        let mut done: Vec<Run> = Vec::new();
        for run in std::mem::take(&mut para.inline) {
            let before = plain_text(&done);
            let set = smarten_with(&before, &run.text, style, SmartRules::ALL);
            trim_end(&mut done, set.rewind);
            let mut next = run;
            next.text = set.text;
            if !next.text.is_empty() {
                done.push(next);
            }
        }
        para.inline = done;
    }
}

/// Reads Markdown (or plain text) into a story: `#` is a chapter title, other headings are
/// subheadings, blank lines separate paragraphs, `***` is a scene break, `*` and `**` are italic
/// and bold, quotations take the epigraph style and list items become paragraphs. Pictures, tables,
/// code and HTML are left out, and the report says so.
pub fn import_markdown(text: &str) -> Imported {
    import_markdown_with(text, Some(QuoteStyle::Curly))
}

/// As `import_markdown`, with the quotes of text set in `smart` style (`None` leaves them as
/// written). Dashes and ellipses follow the same rules as typing (`folio_model::smarten`).
pub fn import_markdown_with(text: &str, smart: Option<QuoteStyle>) -> Imported {
    let mut reader = Reader {
        story: Vec::new(),
        current: Vec::new(),
        style: None,
        bold: 0,
        italic: 0,
        quote: 0,
        in_item: false,
        report: ImportReport::default(),
        smart,
    };
    let mut skipping = 0u32;
    // The parser's own smart punctuation is off: it makes `--` an en dash, and Folio's rules apply.
    let options =
        Options::ENABLE_STRIKETHROUGH | Options::ENABLE_TABLES | Options::ENABLE_FOOTNOTES;
    for event in Parser::new_ext(text, options) {
        // Inside a block that is left out, only its end is looked for.
        if skipping > 0 {
            match event {
                Event::Start(_) => skipping += 1,
                Event::End(_) => skipping -= 1,
                _ => {}
            }
            continue;
        }
        match event {
            Event::Start(Tag::Paragraph) => {}
            Event::End(TagEnd::Paragraph) if !reader.in_item => reader.flush(),
            Event::Start(Tag::Heading { level, .. }) => {
                reader.flush();
                reader.style = Some(if level == HeadingLevel::H1 {
                    CHAPTER
                } else {
                    SUBHEADING
                });
                reader.report.headings += 1;
            }
            Event::End(TagEnd::Heading(_)) => reader.flush(),
            Event::Start(Tag::Strong) => reader.bold += 1,
            Event::End(TagEnd::Strong) => reader.bold = reader.bold.saturating_sub(1),
            Event::Start(Tag::Emphasis) => reader.italic += 1,
            Event::End(TagEnd::Emphasis) => reader.italic = reader.italic.saturating_sub(1),
            Event::Start(Tag::BlockQuote(_)) => {
                reader.flush();
                reader.quote += 1;
            }
            Event::End(TagEnd::BlockQuote(_)) => {
                reader.flush();
                reader.quote = reader.quote.saturating_sub(1);
            }
            Event::Start(Tag::Item) => {
                reader.flush();
                reader.in_item = true;
                reader.push("\u{2013} ");
                reader.note("List items became paragraphs that start with a dash.");
            }
            Event::End(TagEnd::Item) => {
                reader.in_item = false;
                reader.flush();
            }
            Event::Start(Tag::Link { .. }) => {
                reader.note("Links were left out; their words were kept.");
            }
            Event::Start(Tag::Strikethrough) => {
                reader.note("Struck-through words were kept without the line through them.");
            }
            Event::Text(t) => reader.push_text(&t),
            Event::Code(t) => reader.push(&t),
            Event::SoftBreak | Event::HardBreak => reader.push(" "),
            Event::Rule => {
                reader.flush();
                reader.story.push(StoryPara {
                    style: Some(StyleId(BREAK.to_owned())),
                    inline: vec![Run {
                        text: SCENE_BREAK.to_owned(),
                        style: None,
                        bold: false,
                        italic: false,
                        font: None,
                        colour: None,
                    }],
                });
                reader.report.scene_breaks += 1;
            }
            Event::Start(Tag::CodeBlock(_)) => {
                reader.note("Code blocks were left out.");
                skipping = 1;
            }
            Event::Start(Tag::Image { .. }) => {
                reader.note("Pictures were left out. Add them to the book from the Insert tab.");
                skipping = 1;
            }
            Event::Start(Tag::Table(_)) => {
                reader.note("Tables were left out.");
                skipping = 1;
            }
            Event::Start(Tag::HtmlBlock) | Event::Start(Tag::FootnoteDefinition(_)) => {
                reader.note("HTML and footnotes were left out.");
                skipping = 1;
            }
            Event::Html(_) | Event::InlineHtml(_) => reader.note("HTML was left out."),
            Event::FootnoteReference(_) => reader.note("HTML and footnotes were left out."),
            _ => {}
        }
    }
    reader.flush();
    reader.report.paragraphs = reader.story.len() as u32;
    Imported {
        story: reader.story,
        report: reader.report,
    }
}
