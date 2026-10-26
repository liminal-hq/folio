// Editing a frame's words while keeping the emphasis, typeface and colour of stretches of them
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use crate::colour::Colour;
use crate::frame::Run;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// A change to the look of some words. A field left out is left as it is; an empty `font` or
/// `colour` takes the override away.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
#[serde(rename_all = "camelCase", default)]
pub struct RunChange {
    #[ts(optional)]
    pub bold: Option<bool>,
    #[ts(optional)]
    pub italic: Option<bool>,
    #[ts(optional)]
    pub font: Option<String>,
    #[ts(optional)]
    pub colour: Option<String>,
}

impl RunChange {
    /// Whether this changes nothing.
    pub fn is_empty(&self) -> bool {
        self.bold.is_none() && self.italic.is_none() && self.font.is_none() && self.colour.is_none()
    }

    fn apply(&self, run: &mut Run) {
        if let Some(bold) = self.bold {
            run.bold = bold;
        }
        if let Some(italic) = self.italic {
            run.italic = italic;
        }
        if let Some(font) = &self.font {
            run.font = Some(font.clone()).filter(|f| !f.is_empty());
        }
        if let Some(colour) = &self.colour {
            run.colour = Some(Colour(colour.clone())).filter(|c| !c.0.is_empty());
        }
    }
}

/// The words of `runs` as one string.
pub fn text_of(runs: &[Run]) -> String {
    runs.iter().map(|r| r.text.as_str()).collect()
}

/// Whether two runs look the same, whatever words they hold.
fn alike(a: &Run, b: &Run) -> bool {
    a.style == b.style
        && a.bold == b.bold
        && a.italic == b.italic
        && a.font == b.font
        && a.colour == b.colour
}

/// Joins neighbouring runs that look the same, and drops empty ones (keeping one if all are).
fn tidy(runs: Vec<Run>) -> Vec<Run> {
    let mut out: Vec<Run> = Vec::new();
    for run in runs.into_iter().filter(|r| !r.text.is_empty()) {
        match out.last_mut() {
            Some(last) if alike(last, &run) => last.text.push_str(&run.text),
            _ => out.push(run),
        }
    }
    if out.is_empty() {
        out.push(plain(String::new()));
    }
    out
}

fn plain(text: String) -> Run {
    Run {
        text,
        style: None,
        bold: false,
        italic: false,
        font: None,
        colour: None,
    }
}

/// The runs for `new` text, given the `old` runs it replaces: what is typed in a stretch takes the
/// look of the words before it, and words that were not touched keep their own.
pub fn retext(old: &[Run], new: &str) -> Vec<Run> {
    let old_chars: Vec<(char, usize)> = old
        .iter()
        .enumerate()
        .flat_map(|(i, r)| r.text.chars().map(move |c| (c, i)))
        .collect();
    let new_chars: Vec<char> = new.chars().collect();
    if old_chars.is_empty() {
        return match old.first() {
            // Typing into empty words keeps how the empty words were set.
            Some(first) => tidy_one(first, new),
            None => vec![plain(new.to_owned())],
        };
    }
    let prefix = old_chars
        .iter()
        .zip(&new_chars)
        .take_while(|((a, _), b)| a == *b)
        .count();
    let room = old_chars.len().min(new_chars.len()) - prefix;
    let suffix = old_chars
        .iter()
        .rev()
        .zip(new_chars.iter().rev())
        .take(room)
        .take_while(|((a, _), b)| a == *b)
        .count();
    // Which run each new character comes from.
    let inserted_from = old_chars[prefix.saturating_sub(1).min(old_chars.len() - 1)].1;
    let mut source: Vec<usize> = Vec::with_capacity(new_chars.len());
    source.extend(old_chars[..prefix].iter().map(|&(_, i)| i));
    source.extend(std::iter::repeat_n(
        inserted_from,
        new_chars.len() - prefix - suffix,
    ));
    source.extend(
        old_chars[old_chars.len() - suffix..]
            .iter()
            .map(|&(_, i)| i),
    );

    let mut runs: Vec<Run> = Vec::new();
    for (c, &from) in new_chars.iter().zip(&source) {
        match runs.last_mut() {
            Some(last) if alike(last, &old[from]) => last.text.push(*c),
            _ => {
                let mut run = old[from].clone();
                run.text = c.to_string();
                runs.push(run);
            }
        }
    }
    tidy(runs)
}

fn tidy_one(like: &Run, text: &str) -> Vec<Run> {
    let mut run = like.clone();
    run.text = text.to_owned();
    vec![run]
}

/// The runs split in two at character `at` (clamped to the words): what comes before and after.
/// Either side may be empty, and then holds one empty run that looks like its neighbour.
pub fn split_runs(runs: &[Run], at: usize) -> (Vec<Run>, Vec<Run>) {
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
    let like = runs.last().cloned().unwrap_or_else(|| plain(String::new()));
    for side in [&mut before, &mut after] {
        if side.is_empty() {
            let mut run = like.clone();
            run.text.clear();
            side.push(run);
        }
    }
    (tidy(before), tidy(after))
}

/// Joins two stretches of runs into one, merging neighbours that look alike.
pub fn join_runs(first: &[Run], second: &[Run]) -> Vec<Run> {
    tidy(first.iter().chain(second).cloned().collect())
}

/// Applies `change` to the characters `start..end` of the words in `runs`, splitting runs where the
/// stretch begins and ends. Offsets count characters, not bytes, and are clamped to the words.
pub fn style_range(runs: &[Run], start: usize, end: usize, change: &RunChange) -> Vec<Run> {
    let mut out = Vec::new();
    let mut at = 0;
    for run in runs {
        let chars: Vec<char> = run.text.chars().collect();
        let (from, to) = (at, at + chars.len());
        at = to;
        // The parts of this run before, inside and after the stretch.
        let a = start.clamp(from, to) - from;
        let b = end.clamp(from, to) - from;
        for (lo, hi, inside) in [(0, a, false), (a, b, true), (b, chars.len(), false)] {
            if lo >= hi {
                continue;
            }
            let mut piece = run.clone();
            piece.text = chars[lo..hi].iter().collect();
            if inside {
                change.apply(&mut piece);
            }
            out.push(piece);
        }
    }
    tidy(out)
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

    fn shown(runs: &[Run]) -> Vec<(String, bool)> {
        runs.iter().map(|r| (r.text.clone(), r.bold)).collect()
    }

    #[test]
    fn a_stretch_is_split_out_and_styled_and_the_rest_is_left() {
        let runs = [run("Hello brave world", false)];
        let bold = RunChange {
            bold: Some(true),
            ..RunChange::default()
        };
        let out = style_range(&runs, 6, 11, &bold);
        assert_eq!(
            shown(&out),
            vec![
                ("Hello ".into(), false),
                ("brave".into(), true),
                (" world".into(), false)
            ]
        );
        // Styling it back joins the runs up again.
        let back = RunChange {
            bold: Some(false),
            ..RunChange::default()
        };
        assert_eq!(
            shown(&style_range(&out, 6, 11, &back)),
            vec![("Hello brave world".into(), false)]
        );
    }

    #[test]
    fn a_stretch_may_cross_runs_and_offsets_count_characters() {
        let runs = [run("héllo ", true), run("wörld", false)];
        let bold = RunChange {
            bold: Some(true),
            ..RunChange::default()
        };
        let out = style_range(&runs, 3, 9, &bold);
        assert_eq!(
            shown(&out),
            vec![("héllo wör".into(), true), ("ld".into(), false)]
        );
        // Past the end is clamped; an empty stretch changes nothing.
        assert_eq!(style_range(&runs, 20, 30, &bold), runs.to_vec());
        assert_eq!(style_range(&runs, 4, 4, &bold), runs.to_vec());
    }

    #[test]
    fn font_and_colour_are_set_and_taken_away() {
        let runs = [run("abc", false)];
        let set = RunChange {
            font: Some("Zed Sans".into()),
            colour: Some("#a23b2a".into()),
            ..RunChange::default()
        };
        let out = style_range(&runs, 1, 2, &set);
        assert_eq!(out.len(), 3);
        assert_eq!(out[1].font.as_deref(), Some("Zed Sans"));
        assert_eq!(
            out[1].colour.as_ref().map(|c| c.0.as_str()),
            Some("#a23b2a")
        );
        let clear = RunChange {
            font: Some(String::new()),
            colour: Some(String::new()),
            ..RunChange::default()
        };
        assert_eq!(style_range(&out, 1, 2, &clear), runs.to_vec());
    }

    #[test]
    fn editing_the_words_keeps_the_look_of_the_words_left_alone() {
        let runs = [
            run("Hello ", false),
            run("brave", true),
            run(" world", false),
        ];
        // Words typed inside the bold stretch are bold.
        let out = retext(&runs, "Hello bXrave world");
        assert_eq!(
            shown(&out),
            vec![
                ("Hello ".into(), false),
                ("bXrave".into(), true),
                (" world".into(), false)
            ]
        );
        // Words added at the very end take the look of the last words.
        assert_eq!(
            shown(&retext(&runs, "Hello brave world!")),
            vec![
                ("Hello ".into(), false),
                ("brave".into(), true),
                (" world!".into(), false)
            ]
        );
        // Deleting the bold words leaves the rest as it was.
        assert_eq!(
            shown(&retext(&runs, "Hello  world")),
            vec![("Hello  world".into(), false)]
        );
        // Unchanged words give the same runs.
        assert_eq!(retext(&runs, "Hello brave world"), runs.to_vec());
    }

    #[test]
    fn runs_split_and_join_where_the_caret_is() {
        let runs = [
            run("Hello ", false),
            run("brave", true),
            run(" world", false),
        ];
        let (a, b) = split_runs(&runs, 8);
        assert_eq!(
            shown(&a),
            vec![("Hello ".into(), false), ("br".into(), true)]
        );
        assert_eq!(
            shown(&b),
            vec![("ave".into(), true), (" world".into(), false)]
        );
        assert_eq!(join_runs(&a, &b), runs.to_vec());
        // At the ends one side is empty, set like the words beside it.
        let (start, rest) = split_runs(&runs, 0);
        assert_eq!(start.len(), 1);
        assert!(start[0].text.is_empty());
        assert_eq!(rest, runs.to_vec());
        let (all, end) = split_runs(&runs, 99);
        assert_eq!(all, runs.to_vec());
        assert!(end[0].text.is_empty());
    }

    #[test]
    fn typing_into_empty_words_and_clearing_them() {
        let empty = [run("", true)];
        assert_eq!(shown(&retext(&empty, "x")), vec![("x".into(), true)]);
        assert_eq!(retext(&[run("abc", true)], ""), vec![plain(String::new())]);
        assert_eq!(retext(&[], "hi"), vec![plain("hi".into())]);
    }
}
