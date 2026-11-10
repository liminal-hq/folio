// Checks the offsets and caret stops a laid-out line carries, so the page can put a caret in text
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use folio_layout::wire::{FrameLayout, LayoutLine};
use folio_layout::{FontBook, Layouter, StyleSheet};
use folio_model::{
    Book, BookKind, Frame, FrameBase, FrameId, InlineContent, Pt, Rect, Run, StyleId, TextFrame,
};
use folio_presets::{new_book, NewBookSpec, Words};

fn lay(style: &str, width: f64, text: &str) -> FrameLayout {
    lay_proofed(style, width, text, None)
}

fn lay_proofed(style: &str, width: f64, text: &str, proofer: Option<&Proofer>) -> FrameLayout {
    let made = new_book(&NewBookSpec {
        kind: BookKind::Poetry,
        words: Words::Samples,
        ..NewBookSpec::default()
    })
    .unwrap();
    let mut book: Book = made.book;
    let styles = StyleSheet::from_styles(&book.styles);
    book.spreads[1].frames.push(Frame::Text(TextFrame {
        base: FrameBase {
            id: FrameId("tx".into()),
            rect: Rect {
                x: Pt(20.0),
                y: Pt(20.0),
                w: Pt(width),
                h: Pt(600.0),
            },
            rotation: 0.0,
            locked: false,
            from_master: None,
            description: None,
        },
        style: StyleId(style.into()),
        content: Some(InlineContent {
            inline: vec![Run {
                text: text.into(),
                style: None,
                bold: false,
                italic: false,
                font: None,
                colour: None,
                ..Default::default()
            }],
            looks: None,
        }),
        flow: None,
        columns: 1,
        baseline_lock: false,
        overrides: None,
    }));
    let fonts = FontBook::bundled();
    let mut layouter = Layouter::new(&book, &styles, &fonts);
    if let Some(find) = proofer {
        layouter = layouter.with_proofing(find);
    }
    let id = book.spreads[1].id.clone();
    let spread = layouter.layout_spread(&id).unwrap().unwrap();
    spread
        .frames
        .into_iter()
        .find(|f| f.frame.0 == "tx")
        .expect("the frame is laid out")
}

type Proofer = dyn Fn(&str) -> Vec<std::ops::Range<usize>>;

fn paragraphs(text: &str) -> Vec<Vec<char>> {
    text.split('\n').map(|p| p.chars().collect()).collect()
}

/// What every line of a layout must satisfy, whatever the words.
fn check(layout: &FrameLayout, text: &str) {
    let paras = paragraphs(text);
    let mut seen: Vec<Option<&LayoutLine>> = vec![None; paras.len()];
    for line in &layout.lines {
        let p = line.paragraph.expect("every line names its paragraph") as usize;
        let chars = &paras[p];
        assert!(line.start <= line.end, "{line:?}");
        assert!(line.end as usize <= chars.len(), "{line:?}");
        match seen[p] {
            None => assert_eq!(line.start, 0, "a paragraph's first line starts at 0"),
            Some(prev) => assert_eq!(prev.end, line.start, "lines tile the paragraph"),
        }
        seen[p] = Some(line);
        let mut at = line.start;
        for run in &line.runs {
            let n = run.text.chars().count();
            if run.hyphen {
                assert!(run.stops.is_empty());
                assert!(run.start >= at, "the hyphen sits after the word it ends");
                continue;
            }
            assert!(run.start >= at, "runs go forward: {run:?}");
            assert!(run.start as usize + n <= line.end as usize, "{run:?}");
            let shown: String = chars[run.start as usize..run.start as usize + n]
                .iter()
                .collect();
            assert_eq!(shown, run.text, "a run is where it says it is");
            assert_eq!(run.stops.len(), n + 1, "a stop for each boundary: {run:?}");
            assert_eq!(run.stops[0], 0.0);
            assert!(run.stops.windows(2).all(|w| w[0] <= w[1] + 1e-9), "{run:?}");
            assert!((run.stops[n] - run.width).abs() < 0.02, "{run:?}");
            at = run.start + n as u32;
        }
    }
    for (p, last) in seen.iter().enumerate() {
        if let Some(line) = last {
            assert_eq!(
                line.end as usize,
                paras[p].len(),
                "the last line ends the paragraph"
            );
        }
    }
}

#[test]
fn a_plain_frame_names_the_paragraph_of_every_line() {
    let text = "The first paragraph is long enough to wrap onto a second line in a narrow frame.\nAnd a second one.";
    let layout = lay("body", 160.0, text);
    assert!(layout.lines.len() > 2);
    check(&layout, text);
    let second: Vec<_> = layout
        .lines
        .iter()
        .filter(|l| l.paragraph == Some(1))
        .collect();
    assert!(!second.is_empty());
}

#[test]
fn a_hyphenated_word_has_runs_on_both_sides_of_the_break() {
    let text = "An extraordinarily uncharacteristically internationalization of everything.";
    let layout = lay("body", 90.0, text);
    check(&layout, text);
    let hyphens = layout
        .lines
        .iter()
        .filter(|l| l.runs.iter().any(|r| r.hyphen))
        .count();
    assert!(hyphens > 0, "a narrow measure breaks a long word");
}

#[test]
fn a_word_with_hyphens_in_it_keeps_its_offsets() {
    let text = "A six-years-old child asked for a well-known, twenty-one-word-long answer.";
    let layout = lay("body", 110.0, text);
    check(&layout, text);
}

#[test]
fn the_spaces_a_line_ends_on_belong_to_it_and_runs_of_spaces_are_counted() {
    let text = "one   two   three   four   five   six   seven   eight   nine   ten  ";
    let layout = lay("body", 100.0, text);
    check(&layout, text);
    assert!(layout.lines.len() > 1);
    let first = &layout.lines[0];
    let last_run = first.runs.last().unwrap();
    let text_end = last_run.start + last_run.text.chars().count() as u32;
    assert!(
        first.end > text_end,
        "the spaces after the last word are in the line"
    );
}

#[test]
fn ligatures_combining_marks_and_joined_emoji_keep_one_stop_for_each_code_point() {
    let text =
        "office affluent \u{fb01}nal cafe\u{301} \u{1f469}\u{200d}\u{1f469}\u{200d}\u{1f467} done";
    let layout = lay("body", 400.0, text);
    check(&layout, text);
}

#[test]
fn misspelling_marks_count_code_points_from_the_start_of_the_run() {
    let text = "caf\u{e9} \u{1f600} zorb word";
    let find = |t: &str| -> Vec<std::ops::Range<usize>> {
        t.match_indices("zorb")
            .map(|(i, w)| i..i + w.len())
            .collect()
    };
    let layout = lay_proofed("body", 400.0, text, Some(&find));
    check(&layout, text);
    let marked: Vec<_> = layout
        .lines
        .iter()
        .flat_map(|l| l.runs.iter())
        .filter(|r| !r.miss.is_empty())
        .collect();
    assert_eq!(marked.len(), 1);
    assert_eq!(marked[0].text, "zorb");
    assert_eq!(marked[0].miss, vec![(0, 4)]);
    assert_eq!(
        marked[0].start, 7,
        "after caf\u{e9}, a space, the face and a space"
    );
}

#[test]
fn verse_is_a_paragraph_for_each_line_break_and_a_blank_line_has_a_place() {
    let text =
        "Roses are red\n\nViolets are blue, and this line is far too long for the frame\nSugar";
    let layout = lay("poem", 120.0, text);
    check(&layout, text);
    let paras: Vec<u32> = layout.lines.iter().filter_map(|l| l.paragraph).collect();
    assert_eq!(*paras.iter().max().unwrap(), 3);
    assert!(layout
        .lines
        .iter()
        .any(|l| l.paragraph == Some(1) && l.start == 0 && l.end == 0));
}
