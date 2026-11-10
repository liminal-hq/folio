// Editing words in place: replacing a range of a text with typed or pasted words, keeping each stretch's look
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::flow::StoryPara;
use crate::frame::{InlineContent, Run};
use crate::ids::{FlowId, FrameId};
use crate::para::ParaLook;
use crate::runs::{alike, plain, RunChange};

/// A place in some words: a paragraph, and a character in it. Offsets count Unicode code points,
/// as `StyleRange` does; bytes stay in Rust and UTF-16 units stay in the page.
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, TS,
)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub struct TextPos {
    pub para: u32,
    pub offset: u32,
}

/// Whose words an edit changes.
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum TextTarget {
    /// A text frame that holds its own words.
    Frame { frame: FrameId },
    /// The words of a shape.
    Shape { frame: FrameId },
    /// The words of one cell of a table.
    Cell { frame: FrameId, row: u32, col: u32 },
    /// The story of a flow, which runs through its frames.
    Story { flow: FlowId },
}

/// What goes in place of the range an edit replaces.
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum TextInsert {
    /// Plain words, a line break between paragraphs. They take the look of the words before them,
    /// changed by the edit's `look`.
    Text { text: String },
    /// Paragraphs with their own looks, as a paste with formatting brings them. A paragraph style
    /// of their own is kept; the edit's `look` is not applied.
    Rich { paragraphs: Vec<StoryPara> },
}

/// A selection in some words, where the caret was or goes: `anchor` is where it began and `focus`
/// is the caret. An undo or a redo of an edit puts the editor's caret here.
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
pub struct EditCaret {
    pub target: TextTarget,
    pub anchor: TextPos,
    pub focus: TextPos,
}

/// How an edit joins the undo steps.
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
#[serde(rename_all = "camelCase")]
pub enum EditStep {
    /// A keystroke: it joins the step before it when that was typing that ended where this begins
    /// and came a moment before.
    Typing,
    /// A step of its own: a paste, a cut, Enter, a spelling fix.
    Own,
}

/// Why an edit could not be made.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EditError {
    /// A place that is not in the words.
    OutOfRange(String),
    /// The range ends before it begins.
    Backwards,
}

impl std::fmt::Display for EditError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OutOfRange(what) => write!(f, "{what}"),
            Self::Backwards => write!(f, "the range ends before it begins"),
        }
    }
}

impl std::error::Error for EditError {}

fn chars_in(runs: &[Run]) -> usize {
    runs.iter().map(|r| r.text.chars().count()).sum()
}

/// A run that looks like the words at `at` and holds none: the one before the place, or the first
/// when the place is the start.
pub fn look_at(runs: &[Run], at: usize) -> Run {
    let mut seen = 0;
    for run in runs {
        let len = run.text.chars().count();
        if at > seen && at <= seen + len || at == 0 {
            let mut like = run.clone();
            like.text.clear();
            return like;
        }
        seen += len;
    }
    runs.last().map_or_else(
        || plain(String::new()),
        |r| Run {
            text: String::new(),
            ..r.clone()
        },
    )
}

/// The runs split at character `at`: what is before and after. A side with no words holds one
/// empty run that looks like the words at the cut, so what is typed there takes that look.
fn split(runs: &[Run], at: usize) -> (Vec<Run>, Vec<Run>) {
    let (mut before, mut after) = (Vec::new(), Vec::new());
    let mut seen = 0;
    for run in runs {
        let chars: Vec<char> = run.text.chars().collect();
        let (from, to) = (seen, seen + chars.len());
        seen = to;
        let cut = at.clamp(from, to) - from;
        for (lo, hi, side) in [(0, cut, &mut before), (cut, chars.len(), &mut after)] {
            if lo < hi {
                let mut piece = run.clone();
                piece.text = chars[lo..hi].iter().collect();
                side.push(piece);
            }
        }
    }
    for side in [&mut before, &mut after] {
        if side.is_empty() {
            side.push(look_at(runs, at));
        }
    }
    (before, after)
}

/// Runs joined, neighbours that look alike made one. Empty runs go unless nothing else is there,
/// and then the first stays, for its look.
fn join(parts: &[&[Run]]) -> Vec<Run> {
    let all: Vec<&Run> = parts.iter().flat_map(|p| p.iter()).collect();
    let mut out: Vec<Run> = Vec::new();
    for run in all.iter().filter(|r| !r.text.is_empty()) {
        match out.last_mut() {
            Some(last) if alike(last, run) => last.text.push_str(&run.text),
            _ => out.push((*run).clone()),
        }
    }
    if out.is_empty() {
        out.push(
            all.first()
                .map_or_else(|| plain(String::new()), |r| (*r).clone()),
        );
    }
    out
}

/// The paragraphs of some words: one for each line break. A line break is read from the start of
/// the run that follows it, which is where `content_of` writes it, so an empty paragraph keeps the
/// look of its empty run. Words always have a paragraph.
pub fn paragraphs_of(content: Option<&InlineContent>) -> Vec<StoryPara> {
    let looks = content.and_then(|c| c.looks.as_deref());
    let mut paras: Vec<Vec<Run>> = vec![Vec::new()];
    if let Some(content) = content {
        for run in &content.inline {
            for (i, piece) in run.text.split('\n').enumerate() {
                if i > 0 {
                    paras.push(Vec::new());
                }
                let mut r = run.clone();
                r.text = piece.to_owned();
                paras.last_mut().expect("one is open").push(r);
            }
        }
    }
    paras
        .into_iter()
        .map(|runs| {
            let keep: Vec<Run> = runs.into_iter().collect();
            StoryPara {
                style: None,
                inline: join(&[&keep]),
                look: None,
            }
        })
        .enumerate()
        .map(|(i, mut para)| {
            para.look = looks
                .and_then(|l| l.get(i))
                .filter(|l| !l.is_empty())
                .cloned();
            para
        })
        .collect()
}

/// The looks of `paragraphs` as a frame's own words keep them: one for each paragraph up to the
/// last that sets anything, and none when no paragraph does.
pub fn looks_of(paragraphs: &[StoryPara]) -> Option<Vec<ParaLook>> {
    let used = paragraphs
        .iter()
        .rposition(|p| p.look.as_ref().is_some_and(|l| !l.is_empty()))?;
    Some(
        paragraphs[..=used]
            .iter()
            .map(|p| p.look.clone().unwrap_or_default())
            .collect(),
    )
}

/// Paragraphs written back as the runs of a frame's own words: a line break is put at the start of
/// the first run of each paragraph after the first.
pub fn content_of(paragraphs: &[StoryPara]) -> InlineContent {
    let mut inline: Vec<Run> = Vec::new();
    for (i, para) in paragraphs.iter().enumerate() {
        let runs = if para.inline.is_empty() {
            vec![plain(String::new())]
        } else {
            para.inline.clone()
        };
        for (k, mut run) in runs.into_iter().enumerate() {
            if i > 0 && k == 0 {
                run.text.insert(0, '\n');
            }
            inline.push(run);
        }
    }
    InlineContent {
        inline,
        looks: looks_of(paragraphs),
    }
}

/// The characters of paragraph `para` of `paragraphs`, or why there is no such paragraph.
fn length_of(paragraphs: &[StoryPara], pos: TextPos) -> Result<usize, EditError> {
    let para = paragraphs.get(pos.para as usize).ok_or_else(|| {
        EditError::OutOfRange(format!(
            "paragraph {} in words of {} paragraphs",
            pos.para,
            paragraphs.len()
        ))
    })?;
    let length = chars_in(&para.inline);
    if pos.offset as usize > length {
        return Err(EditError::OutOfRange(format!(
            "character {} in a paragraph of {length}",
            pos.offset
        )));
    }
    Ok(length)
}

/// Checks that a range is in the words and runs forwards.
pub fn check_range(paragraphs: &[StoryPara], from: TextPos, to: TextPos) -> Result<(), EditError> {
    length_of(paragraphs, from)?;
    length_of(paragraphs, to)?;
    if to < from {
        return Err(EditError::Backwards);
    }
    Ok(())
}

/// Where the caret is once `insert` has gone in at `from`.
pub fn caret_after(from: TextPos, insert: &TextInsert) -> TextPos {
    match insert {
        TextInsert::Text { text } => {
            let mut pieces = text.split('\n');
            let first = pieces.next().map_or(0, |p| p.chars().count());
            match pieces.next_back() {
                None => TextPos {
                    para: from.para,
                    offset: from.offset + first as u32,
                },
                Some(last) => TextPos {
                    para: from.para + 1 + pieces.count() as u32,
                    offset: last.chars().count() as u32,
                },
            }
        }
        TextInsert::Rich { paragraphs } => match paragraphs.len() {
            0 => from,
            1 => TextPos {
                para: from.para,
                offset: from.offset + chars_in(&paragraphs[0].inline) as u32,
            },
            n => TextPos {
                para: from.para + n as u32 - 1,
                offset: chars_in(&paragraphs[n - 1].inline) as u32,
            },
        },
    }
}

/// Replaces the words from `from` to `to` with `insert`, and answers where the caret goes.
///
/// What is typed takes the look of the words before it (of the first words, at the start), changed
/// by `look`; words that were not touched keep their own. A paragraph left with no words keeps an
/// empty run with its look, so the next thing typed there is set the same way. A line break in
/// the new words starts a paragraph in the style of the one it was typed in.
pub fn replace_range(
    paragraphs: &mut Vec<StoryPara>,
    from: TextPos,
    to: TextPos,
    insert: &TextInsert,
    look: Option<&RunChange>,
) -> Result<TextPos, EditError> {
    check_range(paragraphs, from, to)?;
    let first = from.para as usize;
    let last = to.para as usize;
    let head = split(&paragraphs[first].inline, from.offset as usize).0;
    let tail = split(&paragraphs[last].inline, to.offset as usize).1;
    let style = paragraphs[first].style.clone();
    // A paragraph made by a line break is set as the one it was typed in, and a join keeps the
    // look of the first.
    let own_look = paragraphs[first].look.clone();

    // The new words, a list of runs for each paragraph they make.
    let mut inherited = look_at(&paragraphs[first].inline, from.offset as usize);
    // Words typed at the end of a link do not go on linking; words typed inside it do.
    if inherited.href.is_some()
        && !tail
            .first()
            .is_some_and(|next| !next.text.is_empty() && next.href == inherited.href)
    {
        inherited.href = None;
    }
    let typed = |piece: &str| {
        let mut run = inherited.clone();
        run.text = piece.to_owned();
        if let Some(change) = look {
            change.apply(&mut run);
        }
        run
    };
    type Made = (Option<crate::ids::StyleId>, Option<ParaLook>, Vec<Run>);
    let made: Vec<Made> = match insert {
        TextInsert::Text { text } if text.is_empty() => Vec::new(),
        TextInsert::Text { text } => text
            .split('\n')
            .map(|piece| (None, None, vec![typed(piece)]))
            .collect(),
        TextInsert::Rich { paragraphs } => paragraphs
            .iter()
            .map(|p| (p.style.clone(), p.look.clone(), p.inline.clone()))
            .collect(),
    };

    let rebuilt: Vec<StoryPara> = match made.len() {
        0 => vec![StoryPara {
            style,
            inline: join(&[&head, &tail]),
            look: own_look,
        }],
        1 => vec![StoryPara {
            style,
            inline: join(&[&head, &made[0].2, &tail]),
            look: own_look,
        }],
        n => {
            let mut out = Vec::with_capacity(n);
            for (i, (own, pasted, runs)) in made.iter().enumerate() {
                let style = own.clone().or_else(|| style.clone());
                let look = if i == 0 {
                    own_look.clone()
                } else {
                    pasted.clone().or_else(|| own_look.clone())
                };
                let inline = if i == 0 {
                    join(&[&head, runs])
                } else if i == n - 1 {
                    join(&[runs, &tail])
                } else {
                    join(&[runs])
                };
                out.push(StoryPara {
                    style,
                    inline,
                    look,
                });
            }
            out
        }
    };
    paragraphs.splice(first..=last, rebuilt);
    Ok(caret_after(from, insert))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(text: &str, bold: bool) -> Run {
        Run {
            bold,
            ..plain(text.to_owned())
        }
    }

    fn para(runs: Vec<Run>) -> StoryPara {
        StoryPara {
            style: None,
            inline: runs,
            look: None,
        }
    }

    fn at(para: u32, offset: u32) -> TextPos {
        TextPos { para, offset }
    }

    fn text(s: &str) -> TextInsert {
        TextInsert::Text { text: s.into() }
    }

    fn shown(p: &StoryPara) -> Vec<(String, bool)> {
        p.inline.iter().map(|r| (r.text.clone(), r.bold)).collect()
    }

    #[test]
    fn words_typed_inside_a_link_link_and_at_its_end_do_not() {
        let link = |t: &str| Run {
            href: Some("https://example.org".into()),
            ..plain(t.into())
        };
        let mut inside = vec![para(vec![link("click"), plain(" here".into())])];
        replace_range(&mut inside, at(0, 2), at(0, 2), &text("X"), None).unwrap();
        assert_eq!(inside[0].inline[0].text, "clXick");
        assert!(inside[0].inline[0].href.is_some());
        let mut end = vec![para(vec![link("click"), plain(" here".into())])];
        replace_range(&mut end, at(0, 5), at(0, 5), &text("X"), None).unwrap();
        assert_eq!(end[0].inline[0].text, "click");
        assert!(end[0].inline[1].href.is_none());
        assert_eq!(end[0].inline[1].text, "X here");
    }

    #[test]
    fn typing_takes_the_look_of_the_words_before() {
        let mut ps = vec![para(vec![
            run("Hello ", false),
            run("brave", true),
            run(" world", false),
        ])];
        let caret = replace_range(&mut ps, at(0, 11), at(0, 11), &text("X"), None).unwrap();
        assert_eq!(caret, at(0, 12));
        assert_eq!(
            shown(&ps[0]),
            vec![
                ("Hello ".into(), false),
                ("braveX".into(), true),
                (" world".into(), false)
            ]
        );
        // At the very start, the first words' look.
        replace_range(&mut ps, at(0, 0), at(0, 0), &text("Y"), None).unwrap();
        assert_eq!(ps[0].inline[0].text, "YHello ");
    }

    #[test]
    fn deleting_a_range_across_runs_joins_what_is_left() {
        let mut ps = vec![para(vec![
            run("Hello ", false),
            run("brave", true),
            run(" world", false),
        ])];
        let caret = replace_range(&mut ps, at(0, 4), at(0, 13), &text(""), None).unwrap();
        assert_eq!(caret, at(0, 4));
        assert_eq!(shown(&ps[0]), vec![("Hellorld".into(), false)]);
    }

    #[test]
    fn a_line_break_splits_the_paragraph_and_both_halves_keep_their_look() {
        let mut ps = vec![para(vec![run("ab", false), run("cd", true)])];
        let caret = replace_range(&mut ps, at(0, 3), at(0, 3), &text("\n"), None).unwrap();
        assert_eq!(caret, at(1, 0));
        assert_eq!(ps.len(), 2);
        assert_eq!(
            shown(&ps[0]),
            vec![("ab".into(), false), ("c".into(), true)]
        );
        assert_eq!(shown(&ps[1]), vec![("d".into(), true)]);
    }

    #[test]
    fn enter_at_the_end_of_bold_words_starts_an_empty_bold_paragraph() {
        let mut ps = vec![para(vec![run("ab", true)])];
        replace_range(&mut ps, at(0, 2), at(0, 2), &text("\n"), None).unwrap();
        assert_eq!(shown(&ps[1]), vec![("".into(), true)]);
        // Typing there is bold.
        replace_range(&mut ps, at(1, 0), at(1, 0), &text("x"), None).unwrap();
        assert_eq!(shown(&ps[1]), vec![("x".into(), true)]);
    }

    #[test]
    fn deleting_every_word_keeps_the_look_for_the_next_ones() {
        let mut ps = vec![para(vec![run("abc", true)])];
        replace_range(&mut ps, at(0, 0), at(0, 3), &text(""), None).unwrap();
        assert_eq!(shown(&ps[0]), vec![("".into(), true)]);
        replace_range(&mut ps, at(0, 0), at(0, 0), &text("z"), None).unwrap();
        assert_eq!(shown(&ps[0]), vec![("z".into(), true)]);
    }

    #[test]
    fn a_range_across_paragraphs_joins_them() {
        let mut ps = vec![
            para(vec![run("one", false)]),
            para(vec![run("two", true)]),
            para(vec![run("three", false)]),
        ];
        let caret = replace_range(&mut ps, at(0, 2), at(2, 2), &text(""), None).unwrap();
        assert_eq!(caret, at(0, 2));
        assert_eq!(ps.len(), 1);
        assert_eq!(shown(&ps[0]), vec![("onree".into(), false)]);
    }

    #[test]
    fn pasted_text_with_line_breaks_makes_paragraphs_and_a_look_applies_to_it() {
        let mut ps = vec![para(vec![run("ab", false)])];
        let bold = RunChange {
            bold: Some(true),
            ..RunChange::default()
        };
        let caret =
            replace_range(&mut ps, at(0, 1), at(0, 1), &text("X\nY\nZ"), Some(&bold)).unwrap();
        assert_eq!(caret, at(2, 1));
        assert_eq!(ps.len(), 3);
        assert_eq!(shown(&ps[0]), vec![("a".into(), false), ("X".into(), true)]);
        assert_eq!(shown(&ps[1]), vec![("Y".into(), true)]);
        assert_eq!(shown(&ps[2]), vec![("Z".into(), true), ("b".into(), false)]);
    }

    #[test]
    fn rich_paragraphs_keep_their_looks_and_styles() {
        let mut ps = vec![para(vec![run("ab", false)])];
        let rich = TextInsert::Rich {
            paragraphs: vec![
                StoryPara {
                    style: None,
                    inline: vec![run("X", true)],
                    look: None,
                },
                StoryPara {
                    style: Some(crate::ids::StyleId("h1".into())),
                    inline: vec![run("Y", false)],
                    look: None,
                },
            ],
        };
        let caret = replace_range(&mut ps, at(0, 1), at(0, 1), &rich, None).unwrap();
        assert_eq!(caret, at(1, 1));
        assert_eq!(shown(&ps[0]), vec![("a".into(), false), ("X".into(), true)]);
        assert_eq!(ps[1].style.as_ref().map(|s| s.0.as_str()), Some("h1"));
        assert_eq!(shown(&ps[1]), vec![("Yb".into(), false)]);
    }

    #[test]
    fn places_outside_the_words_and_backwards_ranges_are_refused() {
        let mut ps = vec![para(vec![run("ab", false)])];
        assert!(replace_range(&mut ps, at(0, 3), at(0, 3), &text("x"), None).is_err());
        assert!(replace_range(&mut ps, at(1, 0), at(1, 0), &text("x"), None).is_err());
        assert_eq!(
            replace_range(&mut ps, at(0, 2), at(0, 1), &text("x"), None),
            Err(EditError::Backwards)
        );
        assert_eq!(shown(&ps[0]), vec![("ab".into(), false)]);
    }

    #[test]
    fn offsets_count_code_points() {
        let mut ps = vec![para(vec![run("a\u{1f600}b", false)])];
        let caret = replace_range(&mut ps, at(0, 2), at(0, 2), &text("\u{e9}"), None).unwrap();
        assert_eq!(caret, at(0, 3));
        assert_eq!(ps[0].inline[0].text, "a\u{1f600}\u{e9}b");
    }

    #[test]
    fn a_frames_own_words_round_trip_through_paragraphs() {
        let content = InlineContent::of(vec![run("ab", true), run("\n", false), run("cd", false)]);
        let ps = paragraphs_of(Some(&content));
        assert_eq!(ps.len(), 2);
        assert_eq!(shown(&ps[1]), vec![("cd".into(), false)]);
        let back = content_of(&ps);
        assert_eq!(
            back.inline
                .iter()
                .map(|r| r.text.as_str())
                .collect::<String>(),
            "ab\ncd"
        );
        // An empty paragraph keeps its look through the round trip.
        let ps = vec![para(vec![run("ab", false)]), para(vec![run("", true)])];
        let again = paragraphs_of(Some(&content_of(&ps)));
        assert_eq!(shown(&again[1]), vec![("".into(), true)]);
        assert_eq!(paragraphs_of(None).len(), 1);
    }

    fn spaced(runs: Vec<Run>, after: f64) -> StoryPara {
        StoryPara {
            look: Some(ParaLook {
                space_after: Some(after),
                ..ParaLook::default()
            }),
            ..para(runs)
        }
    }

    fn after(p: &StoryPara) -> Option<f64> {
        p.look.as_ref().and_then(|l| l.space_after)
    }

    #[test]
    fn enter_gives_the_new_paragraph_the_look_of_the_one_it_split() {
        let mut ps = vec![spaced(vec![run("abcd", false)], 12.0)];
        replace_range(&mut ps, at(0, 2), at(0, 2), &text("\n"), None).unwrap();
        assert_eq!((after(&ps[0]), after(&ps[1])), (Some(12.0), Some(12.0)));
    }

    #[test]
    fn joining_paragraphs_keeps_the_look_of_the_first_and_others_are_left_alone() {
        let mut ps = vec![
            spaced(vec![run("one", false)], 4.0),
            spaced(vec![run("two", false)], 8.0),
            spaced(vec![run("three", false)], 18.0),
        ];
        replace_range(&mut ps, at(0, 3), at(1, 0), &text(""), None).unwrap();
        assert_eq!(ps.len(), 2);
        assert_eq!((after(&ps[0]), after(&ps[1])), (Some(4.0), Some(18.0)));
        // Typing inside a paragraph leaves every look where it was.
        replace_range(&mut ps, at(1, 1), at(1, 1), &text("x"), None).unwrap();
        assert_eq!(after(&ps[1]), Some(18.0));
    }

    #[test]
    fn a_frames_own_words_keep_each_paragraphs_look_through_the_round_trip() {
        let ps = vec![
            para(vec![run("a", false)]),
            spaced(vec![run("b", false)], 8.0),
            para(vec![run("c", false)]),
        ];
        let content = content_of(&ps);
        // The list stops at the last paragraph that sets anything.
        assert_eq!(content.looks.as_ref().map(Vec::len), Some(2));
        let back = paragraphs_of(Some(&content));
        assert_eq!(
            back.iter().map(after).collect::<Vec<_>>(),
            vec![None, Some(8.0), None]
        );
        assert_eq!(content_of(&[para(vec![run("a", false)])]).looks, None);
    }
}
