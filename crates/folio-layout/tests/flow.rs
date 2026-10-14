// Tests for pouring a story through frames and columns
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use folio_layout::flow::{flow, FlowOptions, Flowed, FrameBox, PlacedLine, StoryParagraph};
use folio_layout::paragraph::{Span, TextStyle};
use folio_layout::FontBook;

const SENTENCE: &str =
    "The otter counted the stars over the river and then the pebbles under it until morning. ";

fn style() -> TextStyle {
    let mut s = TextStyle::new("Literata", 12.0);
    s.line_height = 1.25;
    s
}

fn paragraph(sentences: usize) -> StoryParagraph {
    StoryParagraph::new(vec![Span::new(&SENTENCE.repeat(sentences), style())])
}

fn run(story: &[StoryParagraph], frames: &[FrameBox], options: &FlowOptions) -> Flowed {
    flow(&FontBook::bundled(), story, frames, "en-CA", options).unwrap()
}

fn of_frame(out: &Flowed, frame: usize) -> Vec<&PlacedLine> {
    out.lines.iter().filter(|l| l.frame == frame).collect()
}

fn text_of(out: &Flowed) -> String {
    out.lines
        .iter()
        .map(|l| {
            l.line
                .runs
                .iter()
                .filter(|r| !r.hyphen)
                .map(|r| r.text.clone())
                .collect::<Vec<_>>()
                .join(" ")
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[test]
fn text_that_fits_stays_in_the_first_frame() {
    let out = run(
        &[paragraph(1)],
        &[FrameBox::new(300.0, 400.0)],
        &FlowOptions::default(),
    );
    assert!(!out.overset);
    assert!(out.lines.iter().all(|l| l.frame == 0));
    assert!(out.used_height[0] > 0.0 && out.used_height[0] < 400.0);
}

#[test]
fn lines_stack_down_the_frame_without_overlapping() {
    let out = run(
        &[paragraph(4)],
        &[FrameBox::new(200.0, 600.0)],
        &FlowOptions::default(),
    );
    for pair in out.lines.windows(2) {
        assert!(pair[1].top >= pair[0].top + pair[0].line.height - 1e-6);
        assert!(pair[1].baseline > pair[0].baseline);
    }
    for l in &out.lines {
        assert!(l.baseline > l.top && l.baseline < l.top + l.line.height);
        assert!(l.top + l.line.height <= 600.0 + 1e-6);
    }
}

#[test]
fn text_runs_on_into_the_next_frame_of_the_chain() {
    let frames = [FrameBox::new(200.0, 100.0), FrameBox::new(200.0, 400.0)];
    let out = run(&[paragraph(6)], &frames, &FlowOptions::default());
    assert!(!out.overset);
    assert!(!of_frame(&out, 0).is_empty() && !of_frame(&out, 1).is_empty());
    let first = of_frame(&out, 0);
    assert!(first.last().unwrap().top + first.last().unwrap().line.height <= 100.0 + 1e-6);
    // Every word of the story is placed once, in order.
    let words: Vec<String> = text_of(&out).split_whitespace().map(String::from).collect();
    let original: Vec<&str> = SENTENCE.repeat(6).leak().split_whitespace().collect();
    let squashed = |v: &[String]| v.join("").replace('-', "");
    assert_eq!(squashed(&words), original.join("").replace('-', ""));
}

#[test]
fn text_with_nowhere_to_go_is_overset_and_says_where_it_stops() {
    let out = run(
        &[paragraph(10)],
        &[FrameBox::new(200.0, 80.0)],
        &FlowOptions::default(),
    );
    assert!(out.overset);
    let (p, offset) = out.overset_from.unwrap();
    assert_eq!(p, 0);
    assert!(offset > 0, "some text was placed before it ran out");
    assert!(!out.lines.is_empty());
}

#[test]
fn a_second_paragraph_that_does_not_fit_at_all_is_overset_from_its_start() {
    let out = run(
        &[paragraph(1), paragraph(1)],
        &[FrameBox::new(200.0, 40.0)],
        &FlowOptions::default(),
    );
    assert!(out.overset);
    let (p, _) = out.overset_from.unwrap();
    assert!(p <= 1);
}

#[test]
fn columns_fill_left_to_right() {
    let mut frame = FrameBox::new(420.0, 120.0);
    frame.columns = 2;
    frame.gutter = 20.0;
    let out = run(&[paragraph(8)], &[frame], &FlowOptions::default());
    let left: Vec<_> = out.lines.iter().filter(|l| l.column == 0).collect();
    let right: Vec<_> = out.lines.iter().filter(|l| l.column == 1).collect();
    assert!(!left.is_empty() && !right.is_empty());
    // Columns are (420 - 20) / 2 = 200 wide, the second starting at 220.
    assert!(left.iter().all(|l| l.x < 200.0));
    assert!(right.iter().all(|l| l.x >= 220.0 - 1e-6));
    for l in &out.lines {
        assert!(l.x - (if l.column == 0 { 0.0 } else { 220.0 }) + l.line.width <= 200.0 + 1.0);
    }
    assert!(right[0].top < 1.0, "the right column starts at the top");
}

#[test]
fn the_inset_keeps_text_off_the_edges() {
    let mut frame = FrameBox::new(300.0, 200.0);
    frame.inset = 10.0;
    let out = run(&[paragraph(2)], &[frame], &FlowOptions::default());
    assert!(out.lines.iter().all(|l| l.x >= 10.0 && l.top >= 10.0));
    for l in &out.lines {
        assert!(l.x + l.line.width <= 290.0 + 1.0);
    }
}

#[test]
fn paragraph_spacing_separates_paragraphs_but_not_at_a_column_top() {
    let mut a = paragraph(1);
    a.space_after = 20.0;
    let mut b = paragraph(1);
    b.space_before = 10.0;
    let out = run(
        &[a, b],
        &[FrameBox::new(300.0, 600.0)],
        &FlowOptions::default(),
    );
    let last_a = out.lines.iter().rfind(|l| l.paragraph == 0).unwrap();
    let first_b = out.lines.iter().find(|l| l.paragraph == 1).unwrap();
    let gap = first_b.top - (last_a.top + last_a.line.height);
    assert!(
        (gap - 20.0).abs() < 1e-6 || (gap - 30.0).abs() < 1e-6,
        "{gap}"
    );
    assert_eq!(out.lines[0].top, 0.0, "no space above the first paragraph");
}

#[test]
fn orphans_and_widows_are_never_left_alone() {
    // Try many frame heights; whenever a paragraph splits, each side holds at least two lines.
    for height in (50..400).step_by(7) {
        let frames = [
            FrameBox::new(180.0, f64::from(height)),
            FrameBox::new(180.0, 600.0),
        ];
        let out = run(&[paragraph(5)], &frames, &FlowOptions::default());
        let in_first = of_frame(&out, 0).len();
        let in_second = of_frame(&out, 1).len();
        if in_second > 0 && in_first > 0 {
            assert!(in_first >= 2, "orphan at height {height}");
            assert!(in_second >= 2, "widow at height {height}");
        }
    }
}

#[test]
fn a_short_paragraph_moves_whole_rather_than_splitting() {
    // Three lines: any split would leave an orphan or a widow.
    let frames = [FrameBox::new(180.0, 36.0), FrameBox::new(180.0, 400.0)];
    let short = StoryParagraph::new(vec![Span::new(
        "The otter slept, and the river kept on running past her door all night.",
        style(),
    )]);
    let out = run(&[short], &frames, &FlowOptions::default());
    assert!(of_frame(&out, 0).is_empty());
    assert!(!of_frame(&out, 1).is_empty());
}

#[test]
fn turning_widow_control_off_lets_a_paragraph_split_anywhere() {
    let options = FlowOptions {
        orphans: 1,
        widows: 1,
        ..FlowOptions::default()
    };
    let frames = [FrameBox::new(180.0, 20.0), FrameBox::new(180.0, 400.0)];
    let out = run(&[paragraph(2)], &frames, &options);
    assert_eq!(of_frame(&out, 0).len(), 1);
}

#[test]
fn baselines_lock_to_the_grid() {
    let options = FlowOptions {
        baseline_grid: Some((15.0, 0.0)),
        ..FlowOptions::default()
    };
    let out = run(&[paragraph(3)], &[FrameBox::new(220.0, 600.0)], &options);
    assert!(!out.lines.is_empty());
    for l in &out.lines {
        let rem = (l.baseline / 15.0).round() * 15.0 - l.baseline;
        assert!(rem.abs() < 1e-6, "baseline {} is off the grid", l.baseline);
    }
    for pair in out.lines.windows(2) {
        assert!(pair[1].baseline - pair[0].baseline >= 15.0 - 1e-6);
    }
}

#[test]
fn the_grid_honours_its_offset() {
    let options = FlowOptions {
        baseline_grid: Some((14.0, 3.0)),
        ..FlowOptions::default()
    };
    let out = run(&[paragraph(2)], &[FrameBox::new(220.0, 600.0)], &options);
    for l in &out.lines {
        let k = (l.baseline - 3.0) / 14.0;
        assert!((k - k.round()).abs() < 1e-6);
    }
}

#[test]
fn an_empty_story_places_nothing() {
    let out = run(&[], &[FrameBox::new(200.0, 100.0)], &FlowOptions::default());
    assert!(out.lines.is_empty() && !out.overset);
}

#[test]
fn a_frame_too_short_for_even_one_line_is_overset() {
    let out = run(
        &[paragraph(1)],
        &[FrameBox::new(200.0, 5.0)],
        &FlowOptions::default(),
    );
    assert!(out.overset);
    assert!(out.lines.is_empty());
    assert_eq!(out.overset_from, Some((0, 0)));
}

#[test]
fn the_continuation_starts_where_the_last_line_left_off() {
    let frames = [FrameBox::new(200.0, 70.0), FrameBox::new(200.0, 70.0)];
    let out = run(&[paragraph(8)], &frames, &FlowOptions::default());
    let second = of_frame(&out, 1);
    assert!(!second.is_empty());
    assert!(second[0].offset > 0);
    assert_eq!(of_frame(&out, 0)[0].offset, 0);
}

#[test]
fn a_poem_split_after_a_blank_line_is_not_repeated() {
    let options = FlowOptions {
        orphans: 1,
        widows: 1,
        ..FlowOptions::default()
    };
    let mut poem = StoryParagraph::new(vec![Span::new(
        "one one\ntwo two\n\nthree three\nfour four",
        style(),
    )]);
    poem.wrap = folio_layout::paragraph::Wrap::Preserve;
    // Room for three lines in the first frame: the blank line is the last one placed.
    let frames = [FrameBox::new(200.0, 48.0), FrameBox::new(200.0, 400.0)];
    let out = run(&[poem], &frames, &options);
    let said = text_of(&out);
    assert_eq!(said.matches("one").count(), 2, "{said}");
    assert_eq!(said.matches("three").count(), 2, "{said}");
}

#[test]
fn a_frame_with_room_for_one_line_still_shows_it() {
    // A caption box a line deep, and a paragraph of several lines: the orphan rule cannot be met,
    // and no later frame has more room, so the first line is drawn instead of nothing.
    let out = run(
        &[paragraph(2)],
        &[FrameBox::new(150.0, 18.0)],
        &FlowOptions::default(),
    );
    assert_eq!(out.lines.len(), 1);
    assert!(out.overset);
}

#[test]
fn a_zero_baseline_increment_means_no_grid() {
    let options = FlowOptions {
        baseline_grid: Some((0.0, 0.0)),
        ..FlowOptions::default()
    };
    let out = run(&[paragraph(1)], &[FrameBox::new(300.0, 400.0)], &options);
    assert!(!out.lines.is_empty());
    assert!(out
        .lines
        .iter()
        .all(|l| l.top.is_finite() && l.baseline.is_finite()));
}

#[test]
fn an_empty_paragraph_takes_a_line_of_space() {
    let story = [
        StoryParagraph::new(vec![Span::new("First", style())]),
        StoryParagraph::new(vec![Span::new("", style())]),
        StoryParagraph::new(vec![Span::new("Second", style())]),
    ];
    let out = run(
        &story,
        &[FrameBox::new(200.0, 400.0)],
        &FlowOptions::default(),
    );
    let tops: Vec<f64> = out.lines.iter().map(|l| l.top).collect();
    assert_eq!(tops.len(), 3);
    assert!(tops[2] - tops[0] >= 2.0 * 12.0, "{tops:?}");
}
