// Pasted HTML: the words of a page copied from a browser, Word or Google Docs as the paragraphs of a story
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

// HTML from the clipboard is whatever the program that made it wrote: Word's conditional comments
// and `<o:p>`, Google Docs' `<b style="font-weight:normal">` round everything, and a browser's
// computed style on every span. The reader keeps what a run of a book can hold — bold, italic,
// underline, strike-through, a colour, a size, a typeface that is installed, a link — and nothing
// else, so no markup, script, style sheet, picture or unknown element survives: the output is
// `Run`s of words, which is safe by construction. A block (`p`, `div`, a heading, a list item, a
// table row) or a `<br>` ends a paragraph, and empty paragraphs are dropped, as a book spaces its
// paragraphs and does not keep blank ones.

use folio_model::{clean_href, join_runs, Colour, Run, StoryPara};

use crate::{ImportReport, Imported};

/// The most HTML that is read, in bytes. A larger paste is pasted as plain words instead.
pub const MAX_HTML_BYTES: usize = 2 * 1024 * 1024;

/// How deep elements are followed; deeper ones are read as if they were not there.
const MAX_DEPTH: usize = 256;

/// Why pasted HTML was not read.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum HtmlError {
    #[error("That is too much to paste with its formatting. It can be pasted as plain words.")]
    TooLarge,
}

/// Elements whose words are not the page's: they are read past, not into.
const HIDDEN: &[&str] = &[
    "head", "title", "script", "style", "template", "noscript", "iframe", "object", "embed", "svg",
    "math", "canvas", "audio", "video", "select", "button", "applet", "xml", "map",
];

/// Elements with nothing inside them.
const VOID: &[&str] = &[
    "br", "hr", "img", "input", "meta", "link", "wbr", "col", "area", "base", "param", "source",
    "track",
];

/// Elements that start and end a paragraph.
const BLOCK: &[&str] = &[
    "address",
    "article",
    "aside",
    "blockquote",
    "body",
    "caption",
    "center",
    "dd",
    "details",
    "dir",
    "div",
    "dl",
    "dt",
    "fieldset",
    "figcaption",
    "figure",
    "footer",
    "form",
    "h1",
    "h2",
    "h3",
    "h4",
    "h5",
    "h6",
    "header",
    "hgroup",
    "hr",
    "html",
    "li",
    "main",
    "menu",
    "nav",
    "ol",
    "p",
    "pre",
    "section",
    "summary",
    "table",
    "tbody",
    "tfoot",
    "thead",
    "tr",
    "ul",
];

/// The look of the words inside an element.
#[derive(Clone, Default)]
struct Look {
    bold: bool,
    italic: bool,
    underline: bool,
    strike: bool,
    colour: Option<String>,
    font: Option<String>,
    size: Option<f64>,
    href: Option<String>,
    hidden: bool,
    pre: bool,
}

struct Open {
    name: String,
    look: Look,
}

struct Reader<'a> {
    installed: &'a dyn Fn(&str) -> bool,
    story: Vec<StoryPara>,
    current: Vec<Run>,
    stack: Vec<Open>,
    /// The lists being read: the number the next item of an ordered list takes, none for a bullet list.
    lists: Vec<Option<u32>>,
    notes: Vec<String>,
    /// The paragraph being read is preformatted, so its spaces are its own.
    keep_space: bool,
}

impl Reader<'_> {
    fn note(&mut self, text: &str) {
        if !self.notes.iter().any(|n| n == text) {
            self.notes.push(text.to_owned());
        }
    }

    fn look(&self) -> Look {
        self.stack
            .last()
            .map(|o| o.look.clone())
            .unwrap_or_default()
    }

    fn text(&mut self, raw: &str) {
        let look = self.look();
        if look.hidden || raw.is_empty() {
            return;
        }
        if look.pre {
            let text = decode_entities(raw);
            for (i, line) in text.split('\n').enumerate() {
                if i > 0 {
                    self.flush();
                }
                self.keep_space = true;
                self.push(line.trim_end_matches('\r'), &look);
            }
            return;
        }
        // Runs of white space are one space, and a paragraph does not begin with one.
        let mut collapsed = String::with_capacity(raw.len());
        let mut after_space = self.ends_with_space() || self.current.is_empty();
        for c in raw.chars() {
            if matches!(c, ' ' | '\t' | '\n' | '\r' | '\u{c}') {
                if !after_space {
                    collapsed.push(' ');
                }
                after_space = true;
            } else {
                collapsed.push(c);
                after_space = false;
            }
        }
        self.push(&decode_entities(&collapsed), &look);
    }

    fn ends_with_space(&self) -> bool {
        self.current.last().is_some_and(|r| r.text.ends_with(' '))
    }

    fn push(&mut self, text: &str, look: &Look) {
        if text.is_empty() {
            return;
        }
        // The page marks a link itself, in its own colour and with its own rule; the blue and the
        // underline a page gives its links are not kept.
        let linked = look.href.is_some();
        let run = Run {
            text: text.to_owned(),
            bold: look.bold,
            italic: look.italic,
            underline: look.underline && !linked,
            strike: look.strike,
            colour: look.colour.clone().filter(|_| !linked).map(Colour),
            font: look.font.clone(),
            size: look.size,
            href: look.href.clone(),
            ..Default::default()
        };
        self.current = join_runs(&self.current, &[run]);
    }

    /// Ends the paragraph being read, if it has words.
    fn flush(&mut self) {
        let mut runs = std::mem::take(&mut self.current);
        let keep = std::mem::take(&mut self.keep_space);
        // The spaces that end a paragraph, and begin one, are not its words (unless it is preformatted).
        while let Some(last) = runs.last_mut().filter(|_| !keep) {
            let kept = last.text.trim_end_matches(' ').len();
            last.text.truncate(kept);
            if last.text.is_empty() {
                runs.pop();
            } else {
                break;
            }
        }
        if let Some(first) = runs.first_mut().filter(|_| !keep) {
            first.text = first.text.trim_start_matches(' ').to_owned();
        }
        runs.retain(|r| !r.text.is_empty());
        if runs.iter().any(|r| !r.text.trim().is_empty()) {
            self.story.push(StoryPara {
                style: None,
                inline: runs,
                look: None,
            });
        }
    }

    fn start(&mut self, name: &str, attrs: &[(String, String)], self_closing: bool) {
        let attr = |key: &str| {
            attrs
                .iter()
                .find(|(k, _)| k == key)
                .map(|(_, v)| v.as_str())
        };
        let hidden_here = self.look().hidden;
        match name {
            "br" | "hr" => return self.flush(),
            "img" => {
                if !hidden_here {
                    self.note("Pictures were left out. Add them to the book from the Insert tab.");
                }
                return;
            }
            _ => {}
        }
        if VOID.contains(&name) {
            return;
        }
        if BLOCK.contains(&name) {
            self.flush();
        }
        let mut look = self.look();
        match name {
            "b" | "strong" => look.bold = true,
            "i" | "em" | "cite" | "dfn" | "var" => look.italic = true,
            "u" | "ins" => look.underline = true,
            "s" | "strike" | "del" => look.strike = true,
            "a" => look.href = attr("href").and_then(|h| clean_href(&decode_entities(h))),
            "pre" => look.pre = true,
            "font" => {
                if let Some(face) = attr("face") {
                    look.font = self.font_of(face);
                }
                if let Some(colour) = attr("color").and_then(colour_of) {
                    look.colour = colour;
                }
                if let Some(size) = attr("size").and_then(font_tag_size) {
                    look.size = Some(size);
                }
            }
            _ => {}
        }
        if HIDDEN.contains(&name) || attr("hidden").is_some() {
            look.hidden = true;
        }
        if let Some(style) = attr("style") {
            self.apply_style(&mut look, style);
        }
        match name {
            "ul" | "menu" | "dir" => self.lists.push(None),
            "ol" => self.lists.push(Some(1)),
            "li" if !look.hidden => {
                let prefix = match self.lists.last_mut() {
                    Some(Some(n)) => {
                        let text = format!("{n}. ");
                        *n += 1;
                        text
                    }
                    _ => "\u{2013} ".to_owned(),
                };
                self.push(&prefix, &Look::default());
            }
            "td" | "th" if !look.hidden => {
                if !self.current.is_empty() && !self.ends_with_space() {
                    self.push(" ", &Look::default());
                }
            }
            "table" if !look.hidden => self.note("Tables came in as paragraphs, a row each."),
            _ => {}
        }
        if self_closing || self.stack.len() >= MAX_DEPTH {
            return;
        }
        self.stack.push(Open {
            name: name.to_owned(),
            look,
        });
    }

    fn end(&mut self, name: &str) {
        if let Some(at) = self.stack.iter().rposition(|o| o.name == name) {
            for open in self.stack.drain(at..) {
                if matches!(open.name.as_str(), "ul" | "ol" | "menu" | "dir") {
                    self.lists.pop();
                }
            }
        }
        if BLOCK.contains(&name) {
            self.flush();
        }
    }

    /// The first typeface of a `font-family` list that is installed, if any is.
    fn font_of(&self, families: &str) -> Option<String> {
        families
            .split(',')
            .map(|f| f.trim().trim_matches(|c| c == '"' || c == '\'').trim())
            .filter(|f| !f.is_empty() && !GENERIC.contains(&f.to_ascii_lowercase().as_str()))
            .find(|f| (self.installed)(f))
            .map(str::to_owned)
    }

    fn apply_style(&self, look: &mut Look, style: &str) {
        for declaration in style.split(';') {
            let Some((property, value)) = declaration.split_once(':') else {
                continue;
            };
            let property = property.trim().to_ascii_lowercase();
            let value = value
                .trim()
                .trim_end_matches("!important")
                .trim()
                .to_ascii_lowercase();
            match property.as_str() {
                "font-weight" => {
                    look.bold = match value.as_str() {
                        "bold" | "bolder" => true,
                        "normal" | "lighter" => false,
                        n => n.parse::<u32>().map_or(look.bold, |w| w >= 600),
                    }
                }
                "font-style" => {
                    look.italic = match value.as_str() {
                        "italic" | "oblique" => true,
                        "normal" => false,
                        _ => look.italic,
                    }
                }
                "text-decoration" | "text-decoration-line" => {
                    if value.contains("none") {
                        look.underline = false;
                        look.strike = false;
                    }
                    if value.contains("underline") {
                        look.underline = true;
                    }
                    if value.contains("line-through") {
                        look.strike = true;
                    }
                }
                "color" => {
                    if let Some(colour) = colour_of(&value) {
                        look.colour = colour;
                    }
                }
                "font-family" => look.font = self.font_of(&declaration_value(declaration)),
                "font-size" => {
                    if let Some(size) = length_in_points(&value) {
                        look.size = Some(size);
                    }
                }
                "display" if value == "none" => look.hidden = true,
                "visibility" if value == "hidden" || value == "collapse" => look.hidden = true,
                "mso-hide" if value == "all" => look.hidden = true,
                "white-space" => look.pre = value.starts_with("pre"),
                _ => {}
            }
        }
    }
}

/// The value of a declaration as written (`font-family` is case-sensitive in its names).
fn declaration_value(declaration: &str) -> String {
    declaration
        .split_once(':')
        .map(|(_, v)| v.trim().trim_end_matches("!important").trim().to_owned())
        .unwrap_or_default()
}

/// Names that are a kind of face and not a face.
const GENERIC: &[&str] = &[
    "serif",
    "sans-serif",
    "monospace",
    "cursive",
    "fantasy",
    "system-ui",
    "ui-serif",
    "ui-sans-serif",
    "ui-monospace",
    "ui-rounded",
    "math",
    "emoji",
    "fangsong",
    "inherit",
    "initial",
    "unset",
    "-apple-system",
    "blinkmacsystemfont",
];

/// A length of the `px` or `pt` kind in points; relative lengths and keywords are none.
fn length_in_points(value: &str) -> Option<f64> {
    let (number, scale) = if let Some(n) = value.strip_suffix("pt") {
        (n, 1.0)
    } else {
        (value.strip_suffix("px")?, 0.75)
    };
    let points = number.trim().parse::<f64>().ok()? * scale;
    (points.is_finite() && (1.0..=1000.0).contains(&points)).then(|| (points * 10.0).round() / 10.0)
}

/// The points of a `<font size=…>`, which counts 1 to 7 (3 is the usual size).
fn font_tag_size(value: &str) -> Option<f64> {
    let value = value.trim();
    let n: i32 = match value.strip_prefix('+') {
        Some(rest) => 3 + rest.parse::<i32>().ok()?,
        None => match value.strip_prefix('-') {
            Some(rest) => 3 - rest.parse::<i32>().ok()?,
            None => value.parse().ok()?,
        },
    };
    Some(match n.clamp(1, 7) {
        1 => 8.0,
        2 => 10.0,
        3 => 12.0,
        4 => 14.0,
        5 => 18.0,
        6 => 24.0,
        _ => 36.0,
    })
}

/// A colour written in CSS as `#rrggbb`: `Some(None)` for one that is the page's ink or paper
/// (near-black and near-white, which a browser writes on every span), `Some(Some(hex))` for a
/// colour worth keeping, and `None` for what is not read (`inherit`, `windowtext`, a bad value).
fn colour_of(value: &str) -> Option<Option<String>> {
    let value = value.trim().to_ascii_lowercase();
    let (r, g, b) = if let Some(hex) = value.strip_prefix('#') {
        let digits: Vec<u8> = hex
            .chars()
            .map(|c| c.to_digit(16).map(|d| d as u8))
            .collect::<Option<_>>()?;
        match digits.len() {
            3 => (digits[0] * 17, digits[1] * 17, digits[2] * 17),
            6 | 8 => (
                digits[0] * 16 + digits[1],
                digits[2] * 16 + digits[3],
                digits[4] * 16 + digits[5],
            ),
            _ => return None,
        }
    } else if let Some(args) = value
        .strip_prefix("rgba(")
        .or_else(|| value.strip_prefix("rgb("))
        .and_then(|v| v.strip_suffix(')'))
    {
        let parts: Vec<&str> = args
            .split(|c: char| c == ',' || c == '/' || c.is_whitespace())
            .filter(|p| !p.is_empty())
            .collect();
        if parts.len() < 3 {
            return None;
        }
        let channel = |p: &str| -> Option<u8> {
            match p.strip_suffix('%') {
                Some(pc) => Some((pc.parse::<f64>().ok()? * 2.55).round().clamp(0.0, 255.0) as u8),
                None => Some(p.parse::<f64>().ok()?.round().clamp(0.0, 255.0) as u8),
            }
        };
        // Nearly see-through colour is no colour.
        if parts
            .get(3)
            .and_then(|a| a.parse::<f64>().ok())
            .is_some_and(|a| a < 0.5)
        {
            return Some(None);
        }
        (channel(parts[0])?, channel(parts[1])?, channel(parts[2])?)
    } else {
        match value.as_str() {
            "black" => (0, 0, 0),
            "white" => (255, 255, 255),
            "red" => (255, 0, 0),
            "green" => (0, 128, 0),
            "blue" => (0, 0, 255),
            "gray" | "grey" => (128, 128, 128),
            "silver" => (192, 192, 192),
            "maroon" => (128, 0, 0),
            "purple" => (128, 0, 128),
            "navy" => (0, 0, 128),
            "teal" => (0, 128, 128),
            "olive" => (128, 128, 0),
            "orange" => (255, 165, 0),
            "yellow" => (255, 255, 0),
            "lime" => (0, 255, 0),
            "aqua" | "cyan" => (0, 255, 255),
            "fuchsia" | "magenta" => (255, 0, 255),
            "brown" => (165, 42, 42),
            "transparent" => return Some(None),
            "windowtext" | "canvastext" | "currentcolor" | "initial" | "unset" => {
                return Some(None)
            }
            _ => return None,
        }
    };
    // Near-black text is the ink of the page the words came from, and near-white is its paper.
    if r.max(g).max(b) <= 0x40 || r.min(g).min(b) >= 0xf0 {
        return Some(None);
    }
    Some(Some(format!("#{r:02x}{g:02x}{b:02x}")))
}

/// Text with its character references read: the named ones writing uses most, and any number.
fn decode_entities(text: &str) -> String {
    if !text.contains('&') {
        return text.to_owned();
    }
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(at) = rest.find('&') {
        out.push_str(&rest[..at]);
        let after = &rest[at + 1..];
        let end = after
            .char_indices()
            .take(12)
            .find(|(_, c)| *c == ';')
            .map(|(i, _)| i);
        let decoded = end.and_then(|end| {
            let name = &after[..end];
            let c = if let Some(number) = name.strip_prefix('#') {
                let code = match number.strip_prefix(['x', 'X']) {
                    Some(hex) => u32::from_str_radix(hex, 16).ok()?,
                    None => number.parse().ok()?,
                };
                char::from_u32(code).filter(|c| *c != '\0')?
            } else {
                named(name)?
            };
            Some((c, end + 1))
        });
        match decoded {
            Some((c, used)) => {
                out.push(c);
                rest = &after[used..];
            }
            None => {
                out.push('&');
                rest = after;
            }
        }
    }
    out.push_str(rest);
    out
}

fn named(name: &str) -> Option<char> {
    Some(match name {
        "amp" => '&',
        "lt" => '<',
        "gt" => '>',
        "quot" => '"',
        "apos" => '\'',
        "nbsp" => '\u{a0}',
        "ensp" | "emsp" | "thinsp" => ' ',
        "ndash" => '\u{2013}',
        "mdash" => '\u{2014}',
        "lsquo" => '\u{2018}',
        "rsquo" => '\u{2019}',
        "sbquo" => '\u{201a}',
        "ldquo" => '\u{201c}',
        "rdquo" => '\u{201d}',
        "bdquo" => '\u{201e}',
        "laquo" => '\u{ab}',
        "raquo" => '\u{bb}',
        "hellip" => '\u{2026}',
        "bull" => '\u{2022}',
        "middot" => '\u{b7}',
        "copy" => '\u{a9}',
        "reg" => '\u{ae}',
        "trade" => '\u{2122}',
        "deg" => '\u{b0}',
        "plusmn" => '\u{b1}',
        "times" => '\u{d7}',
        "divide" => '\u{f7}',
        "frac12" => '\u{bd}',
        "frac14" => '\u{bc}',
        "frac34" => '\u{be}',
        "euro" => '\u{20ac}',
        "pound" => '\u{a3}',
        "yen" => '\u{a5}',
        "cent" => '\u{a2}',
        "sect" => '\u{a7}',
        "para" => '\u{b6}',
        "dagger" => '\u{2020}',
        "shy" => '\u{ad}',
        "zwj" => '\u{200d}',
        "zwnj" => '\u{200c}',
        "eacute" => '\u{e9}',
        "egrave" => '\u{e8}',
        "agrave" => '\u{e0}',
        "aacute" => '\u{e1}',
        "ccedil" => '\u{e7}',
        "ntilde" => '\u{f1}',
        "uuml" => '\u{fc}',
        "ouml" => '\u{f6}',
        "auml" => '\u{e4}',
        _ => return None,
    })
}

/// Reads HTML from the clipboard into the paragraphs of a story. `installed` says whether a typeface
/// is on this computer or in the book, since a family named by a page is kept only if it is.
///
/// What the words of most of the paste have in common — their size and their typeface — is left
/// off, so pasted words take the look of the frame they go into, and what differs (a larger
/// heading, a word in another face) stays.
pub fn import_html(html: &str, installed: &dyn Fn(&str) -> bool) -> Result<Imported, HtmlError> {
    if html.len() > MAX_HTML_BYTES {
        return Err(HtmlError::TooLarge);
    }
    // A fragment copied from a page is between markers, and Windows puts a header before the page.
    let html = match (
        html.find("<!--StartFragment-->"),
        html.find("<!--EndFragment-->"),
    ) {
        (Some(from), Some(to)) if from + 20 <= to => &html[from + 20..to],
        _ => html,
    };
    let mut reader = Reader {
        installed,
        story: Vec::new(),
        current: Vec::new(),
        stack: Vec::new(),
        lists: Vec::new(),
        notes: Vec::new(),
        keep_space: false,
    };
    let bytes = html.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] != b'<' {
            let to = html[i..].find('<').map_or(bytes.len(), |p| i + p);
            reader.text(&html[i..to]);
            i = to;
            continue;
        }
        let rest = &html[i..];
        if rest.starts_with("<!--") {
            i = html[i + 4..]
                .find("-->")
                .map_or(bytes.len(), |p| i + 4 + p + 3);
        } else if rest.starts_with("<!") || rest.starts_with("<?") {
            i = rest.find('>').map_or(bytes.len(), |p| i + p + 1);
        } else if let Some(closing) = rest.strip_prefix("</") {
            let name: String = closing
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || matches!(c, ':' | '-' | '_' | '.'))
                .collect::<String>()
                .to_ascii_lowercase();
            i = rest.find('>').map_or(bytes.len(), |p| i + p + 1);
            if !name.is_empty() {
                reader.end(&name);
            }
        } else if bytes.get(i + 1).is_some_and(u8::is_ascii_alphabetic) {
            let (name, attrs, self_closing, next) = read_start_tag(html, i + 1);
            i = next;
            // The text of a script or a style sheet is not markup and is not read.
            if matches!(name.as_str(), "script" | "style") && !self_closing {
                let lower = html[i..].to_ascii_lowercase();
                let close = format!("</{name}");
                i = lower
                    .find(&close)
                    .and_then(|p| html[i + p..].find('>').map(|q| i + p + q + 1))
                    .unwrap_or(bytes.len());
                continue;
            }
            reader.start(&name, &attrs, self_closing);
        } else {
            reader.text("<");
            i += 1;
        }
    }
    reader.flush();

    let mut story = reader.story;
    crate::keep_unusual(
        &mut story,
        |r| r.size.map(|s| (s * 10.0).round() as i64),
        |r| r.size = None,
    );
    crate::keep_unusual(&mut story, |r| r.font.clone(), |r| r.font = None);
    let report = ImportReport {
        paragraphs: story.len() as u32,
        notes: reader.notes,
        ..ImportReport::default()
    };
    Ok(Imported { story, report })
}

type StartTag = (String, Vec<(String, String)>, bool, usize);

/// Reads a start tag whose name begins at `from`: its name, attributes, whether it ends `/>`, and
/// where the text after it begins.
fn read_start_tag(html: &str, from: usize) -> StartTag {
    let bytes = html.as_bytes();
    let mut i = from;
    while i < bytes.len()
        && (bytes[i].is_ascii_alphanumeric() || matches!(bytes[i], b':' | b'-' | b'_' | b'.'))
    {
        i += 1;
    }
    let name = html[from..i].to_ascii_lowercase();
    let mut attrs = Vec::new();
    let mut self_closing = false;
    while i < bytes.len() {
        while i < bytes.len() && (bytes[i].is_ascii_whitespace() || bytes[i] == b'/') {
            self_closing = bytes[i] == b'/';
            i += 1;
        }
        if i >= bytes.len() {
            break;
        }
        if bytes[i] == b'>' {
            i += 1;
            return (name, attrs, self_closing, i);
        }
        self_closing = false;
        let start = i;
        while i < bytes.len()
            && !bytes[i].is_ascii_whitespace()
            && !matches!(bytes[i], b'=' | b'>' | b'/')
        {
            i += 1;
        }
        let key = html[start..i].to_ascii_lowercase();
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        let mut value = String::new();
        if bytes.get(i) == Some(&b'=') {
            i += 1;
            while i < bytes.len() && bytes[i].is_ascii_whitespace() {
                i += 1;
            }
            match bytes.get(i) {
                Some(&q) if q == b'"' || q == b'\'' => {
                    let end = html[i + 1..]
                        .find(q as char)
                        .map_or(bytes.len(), |p| i + 1 + p);
                    value = html[i + 1..end.min(bytes.len())].to_owned();
                    i = (end + 1).min(bytes.len());
                }
                _ => {
                    let from = i;
                    while i < bytes.len() && !bytes[i].is_ascii_whitespace() && bytes[i] != b'>' {
                        i += 1;
                    }
                    value = html[from..i].to_owned();
                }
            }
        }
        if !key.is_empty() {
            attrs.push((key, value));
        }
    }
    (name, attrs, self_closing, bytes.len())
}
