// Spread templates: ready-made layouts a person adds after a spread, sized to their book
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! A template is a list of frames for a two-page spread, worked out from the book's page width and
//! height, its margin and its bleed, so one definition fits every trim. The frames are drawn in
//! spread space in inches (the left page from 0, the right page from one page width on), the way
//! the sample spreads are, and turned into model frames with fresh ids. What needs features Folio
//! does not have yet (drop caps, bulleted and numbered lists, rounded pictures, picture borders) is
//! left out of the catalogue until it does. Templates are Rust data (D-18, D-48).

use folio_model::{
    BookKind, Fit, Frame, FrameBase, FrameId, ImageFrame, ImageRef, InlineContent, Pt, Rect, Run,
    Shape, ShapeFrame, StyleId, TableCell, TableFrame, TableLook, TextFrame,
};
use serde::Serialize;
use ts_rs::TS;

/// What the gallery shows of a template before it is built.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
#[serde(rename_all = "camelCase")]
pub struct TemplateInfo {
    /// A stable name, such as `pb-bleed`.
    pub id: String,
    /// The group the gallery files it under, such as `Picture book`.
    pub group: String,
    pub name: String,
}

/// The size of the book a template is built for, in inches.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Metrics {
    /// One page's trimmed width and height.
    pub w: f64,
    pub h: f64,
    /// The margin to keep words and pictures inside.
    pub m: f64,
    /// How far a picture that runs off the page goes past the trim.
    pub b: f64,
}

/// The group of templates to show first for a kind of book.
pub fn home_group(kind: BookKind) -> &'static str {
    match kind {
        BookKind::Picture | BookKind::Illustrated => "Picture book",
        BookKind::Novel => "Chapter openers",
        BookKind::Textbook => "Textbook",
        BookKind::Poetry => "Poetry",
        BookKind::Memoir => "Photos",
    }
}

/// The groups, in the order the gallery lists them.
pub const GROUPS: [&str; 6] = [
    "Picture book",
    "Textbook",
    "Photos",
    "Poetry",
    "Chapter openers",
    "Text",
];

/// One frame of a template, in inches in spread space.
enum Piece {
    Text {
        x: f64,
        y: f64,
        w: f64,
        h: f64,
        style: &'static str,
        text: &'static str,
        turn: f64,
    },
    Image {
        x: f64,
        y: f64,
        w: f64,
        h: f64,
        label: &'static str,
        turn: f64,
    },
    Table {
        x: f64,
        y: f64,
        w: f64,
        h: f64,
        rows: Vec<Vec<&'static str>>,
        look: TableLook,
    },
    Bubble {
        x: f64,
        y: f64,
        w: f64,
        h: f64,
        text: &'static str,
    },
}

fn txt(x: f64, y: f64, w: f64, h: f64, style: &'static str, text: &'static str) -> Piece {
    Piece::Text {
        x,
        y,
        w,
        h,
        style,
        text,
        turn: 0.0,
    }
}

fn img(x: f64, y: f64, w: f64, h: f64, label: &'static str) -> Piece {
    Piece::Image {
        x,
        y,
        w,
        h,
        label,
        turn: 0.0,
    }
}

fn turned(piece: Piece, degrees: f64) -> Piece {
    match piece {
        Piece::Text {
            x,
            y,
            w,
            h,
            style,
            text,
            ..
        } => Piece::Text {
            x,
            y,
            w,
            h,
            style,
            text,
            turn: degrees,
        },
        Piece::Image {
            x, y, w, h, label, ..
        } => Piece::Image {
            x,
            y,
            w,
            h,
            label,
            turn: degrees,
        },
        other => other,
    }
}

struct Template {
    id: &'static str,
    group: &'static str,
    name: &'static str,
    build: fn(&Metrics) -> Vec<Piece>,
}

/// Every template, in the order the gallery lists them within a group.
const CATALOGUE: &[Template] = &[
    Template {
        id: "pb-bleed",
        group: "Picture book",
        name: "Full-bleed spread",
        build: |k| {
            vec![
                img(
                    -k.b,
                    -k.b,
                    2.0 * k.w + 2.0 * k.b,
                    k.h + 2.0 * k.b,
                    "One picture across both pages",
                ),
                txt(
                    k.w + k.m,
                    k.h - k.m - 1.6,
                    k.w - 2.0 * k.m,
                    1.5,
                    "story",
                    "And the words sit quietly in the corner of the picture.",
                ),
            ]
        },
    },
    Template {
        id: "pb-split",
        group: "Picture book",
        name: "Picture and words",
        build: |k| {
            vec![
                img(-k.b, -k.b, k.w + k.b, k.h + 2.0 * k.b, "Full-page picture"),
                txt(
                    k.w + k.m,
                    k.h * 0.35,
                    k.w - 2.0 * k.m,
                    k.h * 0.3,
                    "storyC",
                    "Words on their own page, centred and calm.",
                ),
            ]
        },
    },
    Template {
        id: "pb-vignette",
        group: "Picture book",
        name: "Vignettes",
        build: |k| {
            let side = (k.w * 0.6).min(k.h * 0.55);
            let top = k.h * 0.12;
            vec![
                img(k.w * 0.5 - side / 2.0, top, side, side, "Vignette"),
                txt(
                    k.m,
                    top + side + k.w * 0.06,
                    k.w - 2.0 * k.m,
                    1.2,
                    "storyC",
                    "A small moment.",
                ),
                img(k.w * 1.5 - side / 2.0, top, side, side, "Vignette"),
                txt(
                    k.w + k.m,
                    top + side + k.w * 0.06,
                    k.w - 2.0 * k.m,
                    1.2,
                    "storyC",
                    "And another.",
                ),
            ]
        },
    },
    Template {
        id: "pb-panorama",
        group: "Picture book",
        name: "Panorama with words below",
        build: |k| {
            vec![
                img(
                    -k.b,
                    -k.b,
                    2.0 * k.w + 2.0 * k.b,
                    k.h * 0.69 + k.b,
                    "One wide picture across the top of both pages",
                ),
                txt(
                    k.m + 0.25,
                    k.h * 0.74,
                    k.w - 2.0 * k.m - 0.25,
                    k.h * 0.19,
                    "story",
                    "Words for the left page sit under the picture.",
                ),
                txt(
                    k.w + k.m,
                    k.h * 0.74,
                    k.w - 2.0 * k.m - 0.25,
                    k.h * 0.19,
                    "story",
                    "And the story carries on over here.",
                ),
            ]
        },
    },
    Template {
        id: "pb-panorama-low",
        group: "Picture book",
        name: "Words above a panorama",
        build: |k| {
            vec![
                txt(
                    k.m + 0.25,
                    k.m,
                    k.w - 2.0 * k.m - 0.25,
                    k.h * 0.2,
                    "story",
                    "Words up top on the left…",
                ),
                txt(
                    k.w + k.m,
                    k.m,
                    k.w - 2.0 * k.m - 0.25,
                    k.h * 0.2,
                    "story",
                    "…and on the right.",
                ),
                img(
                    -k.b,
                    k.h * 0.3,
                    2.0 * k.w + 2.0 * k.b,
                    k.h * 0.7 + k.b,
                    "One wide picture across the bottom of both pages",
                ),
            ]
        },
    },
    Template {
        id: "pb-words-left",
        group: "Picture book",
        name: "Words and picture",
        build: |k| {
            vec![
                txt(
                    k.m,
                    k.h * 0.33,
                    k.w - 2.0 * k.m,
                    k.h * 0.3,
                    "storyC",
                    "Words on the left, the picture gets the right-hand page.",
                ),
                img(k.w, -k.b, k.w + k.b, k.h + 2.0 * k.b, "Full-page picture"),
            ]
        },
    },
    Template {
        id: "pb-overlay",
        group: "Picture book",
        name: "Words over the picture",
        build: |k| {
            vec![
                img(
                    -k.b,
                    -k.b,
                    2.0 * k.w + 2.0 * k.b,
                    k.h + 2.0 * k.b,
                    "One picture across both pages",
                ),
                txt(
                    k.w + k.m,
                    k.m,
                    k.w - 2.0 * k.m,
                    k.h * 0.24,
                    "callout",
                    "Words sit in a soft box on top of the picture, where the sky is quiet.",
                ),
            ]
        },
    },
    Template {
        id: "pb-two",
        group: "Picture book",
        name: "Two pictures, two captions",
        build: |k| {
            vec![
                img(k.m, k.m, k.w - 2.0 * k.m, k.h * 0.62, "Picture"),
                txt(
                    k.m,
                    k.m + k.h * 0.66,
                    k.w - 2.0 * k.m,
                    k.h * 0.2,
                    "story",
                    "Something happens here.",
                ),
                img(k.w + k.m, k.m, k.w - 2.0 * k.m, k.h * 0.62, "Picture"),
                txt(
                    k.w + k.m,
                    k.m + k.h * 0.66,
                    k.w - 2.0 * k.m,
                    k.h * 0.2,
                    "story",
                    "And then this.",
                ),
            ]
        },
    },
    Template {
        id: "pb-panels",
        group: "Picture book",
        name: "Comic panels",
        build: |k| {
            let gap = 0.15;
            let pw = (k.w - 2.0 * k.m - gap) / 2.0;
            let ph = (k.h - 2.0 * k.m - gap) / 2.0;
            let mut out = Vec::new();
            for page in 0..2 {
                for row in 0..2 {
                    for col in 0..2 {
                        out.push(img(
                            f64::from(page) * k.w + k.m + f64::from(col) * (pw + gap),
                            k.m + f64::from(row) * (ph + gap),
                            pw,
                            ph,
                            "Panel",
                        ));
                    }
                }
            }
            out.push(Piece::Bubble {
                x: k.m + 0.2,
                y: k.m + 0.15,
                w: pw * 0.7,
                h: ph * 0.4,
                text: "Oh!",
            });
            out
        },
    },
    Template {
        id: "pb-text",
        group: "Picture book",
        name: "Words only (opening or ending)",
        build: |k| {
            vec![
                txt(
                    k.w + k.m,
                    k.h * 0.38,
                    k.w - 2.0 * k.m,
                    k.h * 0.24,
                    "storyC",
                    "The end.",
                ),
                txt(k.w + k.m, k.h * 0.64, k.w - 2.0 * k.m, 0.5, "ornament", "❦"),
            ]
        },
    },
    Template {
        id: "lesson-2col",
        group: "Textbook",
        name: "Two-column reading",
        build: |k| {
            let col = (k.w - 2.0 * k.m - 0.25) / 2.0;
            vec![
                txt(k.m, k.m, k.w - 2.0 * k.m, 0.8, "h1", "Section title"),
                txt(
                    k.m,
                    k.m + 1.0,
                    col,
                    k.h - 2.0 * k.m - 1.0,
                    "body",
                    "A long reading set in two columns, so lines stay short and easy to follow.",
                ),
                txt(
                    k.m + col + 0.25,
                    k.m + 1.0,
                    col,
                    k.h - 2.0 * k.m - 1.0,
                    "body",
                    "The second column carries on.",
                ),
                txt(
                    k.w + k.m,
                    k.m,
                    k.w - 2.0 * k.m,
                    1.4,
                    "callout",
                    "Key idea · a one-line summary of the section.",
                ),
                img(k.w + k.m, k.m + 1.7, k.w - 2.0 * k.m, k.h * 0.4, "Diagram"),
                txt(
                    k.w + k.m,
                    k.m + 1.8 + k.h * 0.4,
                    k.w - 2.0 * k.m,
                    0.5,
                    "caption",
                    "Figure · what the diagram shows.",
                ),
            ]
        },
    },
    Template {
        id: "lesson",
        group: "Textbook",
        name: "Lesson and worksheet",
        build: |k| {
            vec![
                txt(k.m, k.m, k.w - 2.0 * k.m, 0.35, "label", "Lesson 4"),
                txt(
                    k.m,
                    k.m + 0.4,
                    k.w - 2.0 * k.m,
                    0.9,
                    "h1big",
                    "Lesson title",
                ),
                txt(
                    k.m,
                    k.m + 1.45,
                    k.w - 2.0 * k.m,
                    1.1,
                    "callout",
                    "You'll learn to… · one thing · another thing",
                ),
                txt(
                    k.m,
                    k.m + 2.8,
                    k.w - 2.0 * k.m,
                    (k.h - 2.0 * k.m - 2.8).max(1.0),
                    "bodyL",
                    "Introduce the idea in a paragraph or two.",
                ),
                txt(
                    k.w + k.m,
                    k.m,
                    k.w - 2.0 * k.m,
                    0.45,
                    "h2",
                    "Try it yourself",
                ),
                Piece::Table {
                    x: k.w + k.m,
                    y: k.m + 0.55,
                    w: k.w - 2.0 * k.m,
                    h: 1.8,
                    rows: vec![
                        vec!["Step", "What you did", "What happened"],
                        vec!["1", "", ""],
                        vec!["2", "", ""],
                        vec!["3", "", ""],
                    ],
                    look: TableLook::Grid,
                },
                img(
                    k.w + k.m,
                    k.m + 2.6,
                    k.w - 2.0 * k.m,
                    (k.h - 2.0 * k.m - 3.3).max(1.0),
                    "Figure",
                ),
                txt(
                    k.w + k.m,
                    k.h - k.m - 0.5,
                    k.w - 2.0 * k.m,
                    0.5,
                    "caption",
                    "Figure 4.1 · A caption explains the picture.",
                ),
            ]
        },
    },
    Template {
        id: "collage",
        group: "Photos",
        name: "Photo collage",
        build: |k| {
            let gap = 0.15;
            let cw = (2.0 * k.w - 2.0 * k.m - 2.0 * gap) / 3.0;
            let ch = (k.h - 2.0 * k.m - gap) / 2.0;
            vec![
                img(k.m, k.m, cw * 2.0 + gap, ch, "Photo"),
                img(k.m + 2.0 * (cw + gap), k.m, cw, ch, "Photo"),
                img(k.m, k.m + ch + gap, cw, ch, "Photo"),
                img(k.m + cw + gap, k.m + ch + gap, cw, ch, "Photo"),
                img(
                    k.m + 2.0 * (cw + gap),
                    k.m + ch + gap,
                    cw,
                    ch * 0.7,
                    "Photo",
                ),
                txt(
                    k.m + 2.0 * (cw + gap),
                    k.m + ch + gap + ch * 0.74,
                    cw,
                    ch * 0.26,
                    "caption",
                    "Summer, the year the porch went up.",
                ),
            ]
        },
    },
    Template {
        id: "memoir-photo",
        group: "Photos",
        name: "Photo page and story",
        build: |k| {
            vec![
                img(-k.b, -k.b, k.w + k.b, k.h * 0.7 + k.b, "Photo"),
                txt(
                    k.m,
                    k.h * 0.72,
                    k.w - 2.0 * k.m,
                    0.7,
                    "caption",
                    "A caption: who, where, which summer.",
                ),
                txt(
                    k.w + k.m,
                    k.m,
                    k.w - 2.0 * k.m,
                    k.h - 2.0 * k.m,
                    "body",
                    "The story that goes with the photo.",
                ),
            ]
        },
    },
    Template {
        id: "memoir-polaroids",
        group: "Photos",
        name: "Scattered snapshots",
        build: |k| {
            let big = (k.w * 0.5).min(k.h * 0.45);
            let small = (k.w * 0.48).min(k.h * 0.4);
            vec![
                turned(img(k.m + 0.2, k.m + 0.3, big, big, "Snapshot"), -6.0),
                turned(
                    img(k.w * 0.42, k.h - k.m - small, small, small, "Snapshot"),
                    5.0,
                ),
                turned(img(k.w + k.m + 0.3, k.m + 0.2, big, big, "Snapshot"), 4.0),
                turned(
                    txt(
                        k.w + k.m,
                        k.h * 0.62,
                        k.w - 2.0 * k.m,
                        k.h * 0.25,
                        "bodyL",
                        "A few lines about the day.",
                    ),
                    -1.0,
                ),
            ]
        },
    },
    Template {
        id: "poem-facing",
        group: "Poetry",
        name: "Two poems facing",
        build: |k| {
            let mut out = Vec::new();
            for (page, title) in [(0.0, "First poem"), (1.0, "Second poem")] {
                out.push(txt(
                    page * k.w + k.m,
                    k.h * 0.16,
                    k.w - 2.0 * k.m,
                    0.6,
                    "h1",
                    title,
                ));
                out.push(txt(
                    page * k.w + k.m,
                    k.h * 0.16 + 0.9,
                    k.w - 2.0 * k.m,
                    k.h * 0.62,
                    "poem",
                    "A first line\nand the next,\n\na second stanza\nto close.",
                ));
            }
            out
        },
    },
    Template {
        id: "poem",
        group: "Poetry",
        name: "Poem, centred",
        build: |k| {
            vec![
                txt(k.m, k.h * 0.45, k.w - 2.0 * k.m, 0.6, "ornament", "❦"),
                txt(k.w + k.m, k.h * 0.18, k.w - 2.0 * k.m, 0.6, "h1", "Title"),
                txt(
                    k.w + k.m,
                    k.h * 0.18 + 0.9,
                    k.w - 2.0 * k.m,
                    k.h * 0.6,
                    "poem",
                    "The first line\nand the turn of the second,\n\nthen a quiet stanza\nto end on.",
                ),
            ]
        },
    },
    Template {
        id: "op-photo",
        group: "Chapter openers",
        name: "Photo opener",
        build: |k| {
            vec![
                img(-k.b, -k.b, k.w + k.b, k.h + 2.0 * k.b, "Chapter photo"),
                txt(k.w + k.m, k.m + 0.4, k.w - 2.0 * k.m, 1.2, "numeral", "3"),
                txt(
                    k.w + k.m,
                    k.m + 1.7,
                    k.w - 2.0 * k.m,
                    0.9,
                    "h1big",
                    "Chapter title",
                ),
                txt(
                    k.w + k.m,
                    k.m + 2.9,
                    k.w - 2.0 * k.m,
                    (k.h - 2.0 * k.m - 2.9).max(1.0),
                    "body",
                    "Opening paragraph.",
                ),
            ]
        },
    },
    Template {
        id: "op-part",
        group: "Chapter openers",
        name: "Part title",
        build: |k| {
            vec![
                txt(
                    k.w + k.m,
                    k.h * 0.36,
                    k.w - 2.0 * k.m,
                    0.4,
                    "label",
                    "Part two",
                ),
                txt(
                    k.w + k.m,
                    k.h * 0.36 + 0.45,
                    k.w - 2.0 * k.m,
                    1.0,
                    "h1big",
                    "The long summer",
                ),
                txt(
                    k.w + k.m,
                    k.h * 0.36 + 1.55,
                    k.w - 2.0 * k.m,
                    0.5,
                    "ornament",
                    "⁂",
                ),
            ]
        },
    },
    Template {
        id: "op-minimal",
        group: "Chapter openers",
        name: "Minimal opener",
        build: |k| {
            vec![
                txt(k.w + k.m, k.h * 0.42, k.w - 2.0 * k.m, 1.4, "numeral", "4"),
                txt(
                    k.w + k.m,
                    k.h * 0.42 + 1.4,
                    k.w - 2.0 * k.m,
                    0.6,
                    "h1",
                    "Chapter title",
                ),
                txt(
                    k.w + k.m,
                    k.h * 0.42 + 2.2,
                    k.w - 2.0 * k.m,
                    (k.h * 0.58 - k.m - 2.2).max(1.0),
                    "body",
                    "Opening paragraph.",
                ),
            ]
        },
    },
    Template {
        id: "body-plain",
        group: "Text",
        name: "Plain text spread",
        build: |k| {
            vec![
                txt(
                    k.m,
                    k.m,
                    k.w - 2.0 * k.m,
                    k.h - 2.0 * k.m,
                    "body",
                    "Body text on the left page.",
                ),
                txt(
                    k.w + k.m,
                    k.m,
                    k.w - 2.0 * k.m,
                    k.h - 2.0 * k.m,
                    "body",
                    "Body text continues on the right.",
                ),
            ]
        },
    },
    Template {
        id: "body-quote",
        group: "Text",
        name: "Text with a pull quote",
        build: |k| {
            vec![
                txt(
                    k.m,
                    k.m,
                    k.w - 2.0 * k.m,
                    k.h - 2.0 * k.m,
                    "body",
                    "Body text on the left page.",
                ),
                txt(
                    k.w + k.m,
                    k.m,
                    k.w - 2.0 * k.m,
                    k.h * 0.4,
                    "body",
                    "Body text continues…",
                ),
                txt(
                    k.w + k.m + 0.3,
                    k.h * 0.45,
                    k.w - 2.0 * k.m - 0.6,
                    k.h * 0.2,
                    "epigraph",
                    "“A line worth repeating, set large and quiet.”",
                ),
                txt(
                    k.w + k.m,
                    k.h * 0.68,
                    k.w - 2.0 * k.m,
                    k.h * 0.32 - k.m,
                    "body",
                    "…and after the quote.",
                ),
            ]
        },
    },
];

/// Every template, in the order the gallery lists them: by group, then as written.
pub fn list() -> Vec<TemplateInfo> {
    GROUPS
        .iter()
        .flat_map(|group| CATALOGUE.iter().filter(move |t| t.group == *group))
        .map(|t| TemplateInfo {
            id: t.id.to_owned(),
            group: t.group.to_owned(),
            name: t.name.to_owned(),
        })
        .collect()
}

/// The styles a template names, for a check that the kinds all have them.
#[doc(hidden)]
pub fn styles_named() -> Vec<&'static str> {
    let k = Metrics {
        w: 8.5,
        h: 8.5,
        m: 0.5,
        b: 0.125,
    };
    let mut names: Vec<&'static str> = CATALOGUE
        .iter()
        .flat_map(|t| (t.build)(&k))
        .filter_map(|piece| match piece {
            Piece::Text { style, .. } => Some(style),
            Piece::Bubble { .. } => Some("bubble"),
            _ => None,
        })
        .collect();
    names.sort_unstable();
    names.dedup();
    names
}

fn base(
    ids: &mut dyn FnMut(&str) -> String,
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    turn: f64,
) -> FrameBase {
    FrameBase {
        id: FrameId(ids("fr")),
        rect: Rect {
            x: Pt::from_in(x),
            y: Pt::from_in(y),
            w: Pt::from_in(w),
            h: Pt::from_in(h),
        },
        rotation: turn,
        locked: false,
        from_master: None,
        description: None,
    }
}

fn words(text: &str) -> InlineContent {
    InlineContent {
        inline: vec![Run {
            text: text.to_owned(),
            style: None,
            bold: false,
            italic: false,
            font: None,
            colour: None,
        }],
    }
}

/// The frames of template `id` for a book of size `k`, with new ids from `ids`; `None` when there is
/// no such template.
pub fn build(id: &str, k: &Metrics, ids: &mut dyn FnMut(&str) -> String) -> Option<Vec<Frame>> {
    let template = CATALOGUE.iter().find(|t| t.id == id)?;
    let frames = (template.build)(k)
        .into_iter()
        .map(|piece| match piece {
            Piece::Text {
                x,
                y,
                w,
                h,
                style,
                text,
                turn,
            } => Frame::Text(TextFrame {
                base: base(ids, x, y, w, h, turn),
                style: StyleId(style.to_owned()),
                content: Some(words(text)),
                flow: None,
                columns: 1,
                baseline_lock: false,
                overrides: None,
            }),
            Piece::Image {
                x,
                y,
                w,
                h,
                label,
                turn,
            } => Frame::Image(ImageFrame {
                base: base(ids, x, y, w, h, turn),
                image: ImageRef {
                    asset: None,
                    fit: Fit::Fill,
                    focus: (0.5, 0.5),
                    crop: None,
                },
                placeholder_label: Some(label.to_owned()),
            }),
            Piece::Table {
                x,
                y,
                w,
                h,
                rows,
                look,
            } => {
                let cols = rows.first().map_or(1, Vec::len).max(1);
                Frame::Table(TableFrame {
                    base: base(ids, x, y, w, h, 0.0),
                    cols: vec![1.0 / cols as f64; cols],
                    rows: rows
                        .iter()
                        .map(|row| {
                            row.iter()
                                .map(|cell| TableCell {
                                    content: (!cell.is_empty()).then(|| words(cell)),
                                })
                                .collect()
                        })
                        .collect(),
                    head: 1,
                    look,
                    style: None,
                    overrides: None,
                })
            }
            Piece::Bubble { x, y, w, h, text } => Frame::Shape(ShapeFrame {
                base: base(ids, x, y, w, h, 0.0),
                shape: Shape::Speech,
                fill: Some(folio_model::Colour("#ffffff".into())),
                style: None,
                stroke: Some(folio_model::Colour("#1c1b20".into())),
                stroke_width: Some(Pt(1.5)),
                radius: None,
                flip: false,
                content: Some(words(text)),
                overrides: None,
            }),
        })
        .collect();
    Some(frames)
}
