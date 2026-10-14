// Tests for laying a paragraph out into lines
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use folio_layout::paragraph::{
    layout_paragraph, Align, ParagraphLayout, ParagraphSpec, Span, TextStyle, Wrap,
};
use folio_layout::FontBook;

const STORY: &str = "The otter who couldn't sleep counted the stars over the river, and then the pebbles under it, and then every single ripple that wandered past her whiskers until the morning came.";

fn style() -> TextStyle {
    TextStyle::new("Literata", 12.0)
}

fn layout(
    spans: &[Span],
    width: f64,
    tweak: impl FnOnce(&mut ParagraphSpec<'_>),
) -> ParagraphLayout {
    let mut spec = ParagraphSpec::new(spans, "en-CA");
    tweak(&mut spec);
    layout_paragraph(&FontBook::bundled(), &spec, width).unwrap()
}

fn words(line: &folio_layout::paragraph::Line) -> String {
    line.runs
        .iter()
        .filter(|r| !r.hyphen)
        .map(|r| r.text.as_str())
        .collect::<Vec<_>>()
        .join("|")
}

#[test]
fn text_wraps_into_lines_that_fit() {
    let spans = [Span::new(STORY, style())];
    let out = layout(&spans, 200.0, |_| {});
    assert!(out.lines.len() >= 4);
    for line in &out.lines {
        assert!(!line.overfull);
        assert!(line.width <= 200.0 + 1.0, "{}", line.width);
    }
    // Ragged lines are scored like justified ones, so the natural ragged edge can need a second
    // tolerance step; what matters is that an optimal fit was found at all.
    assert!(out.tolerance.is_some(), "an optimal fit exists");
}

#[test]
fn every_word_appears_once_and_in_order() {
    let spans = [Span::new(STORY, style())];
    let out = layout(&spans, 180.0, |s| s.hyphenate = false);
    let joined: String = out
        .lines
        .iter()
        .flat_map(|l| l.runs.iter().map(|r| r.text.clone()))
        .collect::<Vec<_>>()
        .join(" ");
    assert_eq!(joined, STORY);
}

#[test]
fn hyphenated_words_are_put_back_together() {
    let long =
        "An extraordinarily unquestionable misunderstanding of electromagnetic representations \
                kept the incomprehensible otter wondering about disestablishmentarianism all night";
    let spans = [Span::new(long, style())];
    let out = layout(&spans, 110.0, |_| {});
    assert!(
        out.lines.iter().any(|l| l.hyphenated),
        "narrow lines need hyphens"
    );
    // Words that continue on the next line have no space between the pieces.
    let mut text = String::new();
    for line in &out.lines {
        for run in line.runs.iter().filter(|r| !r.hyphen) {
            text.push_str(&run.text);
            text.push('\u{1}');
        }
        if !line.hyphenated {
            text.push(' ');
        }
    }
    let rebuilt = text.replace("\u{1}\u{1}", "").replace('\u{1}', " ");
    let squashed = |s: &str| s.split_whitespace().collect::<Vec<_>>().join("");
    assert_eq!(squashed(&rebuilt), squashed(long));
}

#[test]
fn a_hyphenated_line_ends_with_a_drawn_hyphen() {
    let spans = [Span::new(STORY, style())];
    let out = layout(&spans, 120.0, |_| {});
    for line in out.lines.iter().filter(|l| l.hyphenated) {
        let last = line.runs.last().unwrap();
        assert!(last.hyphen && last.text == "-");
    }
    for line in out.lines.iter().filter(|l| !l.hyphenated) {
        assert!(line.runs.iter().all(|r| !r.hyphen));
    }
}

#[test]
fn no_more_than_two_hyphens_in_a_row() {
    let text = "extraordinary incomprehensible disestablishment interchangeability unquestionably \
                counterproductive electromagnetic representational misunderstanding "
        .repeat(3);
    let spans = [Span::new(&text, style())];
    let out = layout(&spans, 90.0, |_| {});
    let mut run = 0;
    for line in &out.lines {
        run = if line.hyphenated { run + 1 } else { 0 };
        assert!(run <= 2, "three hyphens in a row");
    }
}

#[test]
fn turning_hyphenation_off_keeps_words_whole() {
    let spans = [Span::new(STORY, style())];
    let out = layout(&spans, 120.0, |s| s.hyphenate = false);
    assert!(out.lines.iter().all(|l| !l.hyphenated));
}

#[test]
fn justified_lines_fill_the_measure_except_the_last() {
    let spans = [Span::new(STORY, style())];
    let out = layout(&spans, 220.0, |s| s.align = Align::Justify);
    let (last, rest) = out.lines.split_last().unwrap();
    for line in rest {
        assert!((line.width - 220.0).abs() < 1.0, "{}", line.width);
    }
    assert!(last.width < 220.0);
    assert_eq!(last.x, 0.0);
}

#[test]
fn alignment_places_the_lines() {
    let spans = [Span::new("A short line", style())];
    let left = layout(&spans, 300.0, |_| {});
    let centre = layout(&spans, 300.0, |s| s.align = Align::Center);
    let right = layout(&spans, 300.0, |s| s.align = Align::Right);
    let w = left.lines[0].width;
    assert_eq!(left.lines[0].x, 0.0);
    assert!((centre.lines[0].x - (300.0 - w) / 2.0).abs() < 1e-6);
    assert!((right.lines[0].x - (300.0 - w)).abs() < 1e-6);
}

#[test]
fn the_first_line_is_indented() {
    let spans = [Span::new(STORY, style())];
    let out = layout(&spans, 200.0, |s| s.first_indent = 18.0);
    assert_eq!(out.lines[0].x, 18.0);
    assert_eq!(out.lines[1].x, 0.0);
    assert!(out.lines[0].x + out.lines[0].width <= 200.0 + 1.0);
}

#[test]
fn spans_in_different_styles_keep_their_own_fonts_and_stay_in_one_word() {
    let mut bold = style();
    bold.weight = 700;
    let spans = [
        Span::new("The otter was ", style()),
        Span::new("very", bold.clone()),
        Span::new("sleepy", style()),
        Span::new(" indeed", style()),
    ];
    let out = layout(&spans, 400.0, |s| s.hyphenate = false);
    let runs = &out.lines[0].runs;
    assert_eq!(
        runs.iter()
            .map(|r| (r.span, r.text.as_str()))
            .collect::<Vec<_>>(),
        [
            (0, "The"),
            (0, "otter"),
            (0, "was"),
            (1, "very"),
            (2, "sleepy"),
            (3, "indeed")
        ]
    );
    // "very" and "sleepy" touch: no space between spans that are one word.
    let very = &runs[3];
    let sleepy = &runs[4];
    assert!((very.x + very.width - sleepy.x).abs() < 1e-9);
}

#[test]
fn bold_text_is_wider_than_regular() {
    let regular = [Span::new("Counting stars", style())];
    let mut b = style();
    b.weight = 700;
    let bold = [Span::new("Counting stars", b)];
    let a = layout(&regular, 500.0, |_| {});
    let c = layout(&bold, 500.0, |_| {});
    assert!(c.lines[0].width > a.lines[0].width);
}

#[test]
fn line_height_follows_the_style() {
    let mut tall = style();
    tall.line_height = 2.0;
    let spans = [Span::new("one line", tall)];
    let out = layout(&spans, 300.0, |_| {});
    assert!((out.lines[0].height - 24.0).abs() < 1e-9);
    assert!(out.lines[0].ascent > 0.0 && out.lines[0].descent > 0.0);
}

#[test]
fn a_larger_span_makes_its_line_taller() {
    let mut big = style();
    big.size_pt = 30.0;
    let spans = [
        Span::new("small ", style()),
        Span::new("BIG", big),
        Span::new(" small", style()),
    ];
    let out = layout(&spans, 400.0, |_| {});
    assert!((out.lines[0].height - 39.0).abs() < 1e-9);
}

#[test]
fn poetry_keeps_its_line_breaks() {
    let poem = "The river runs slow\nThe otter drifts by\nThe stars count themselves";
    let spans = [Span::new(poem, style())];
    let out = layout(&spans, 400.0, |s| s.wrap = Wrap::Preserve);
    assert_eq!(out.lines.len(), 3);
    assert!(!out.overlong);
    assert_eq!(words(&out.lines[1]), "The|otter|drifts|by");
}

#[test]
fn an_overlong_poem_line_wraps_hangs_and_is_flagged() {
    let poem = "A line so long that it simply cannot fit inside the measure it was given\nShort";
    let spans = [Span::new(poem, style())];
    let out = layout(&spans, 150.0, |s| {
        s.wrap = Wrap::Preserve;
        s.hang_indent = 12.0;
    });
    assert!(out.overlong);
    assert!(out.lines.len() > 2);
    assert_eq!(out.lines[0].x, 0.0);
    assert_eq!(out.lines[1].x, 12.0);
    assert_eq!(words(out.lines.last().unwrap()), "Short");
}

#[test]
fn blank_lines_in_poetry_are_kept() {
    let poem = "First verse\n\nSecond verse";
    let spans = [Span::new(poem, style())];
    let out = layout(&spans, 300.0, |s| s.wrap = Wrap::Preserve);
    assert_eq!(out.lines.len(), 3);
    assert!(out.lines[1].runs.is_empty());
    assert!(out.lines[1].height > 0.0);
}

#[test]
fn an_empty_paragraph_is_one_empty_line() {
    let spans = [Span::new("", style())];
    let out = layout(&spans, 300.0, |_| {});
    assert_eq!(out.lines.len(), 1);
    assert!(out.lines[0].runs.is_empty());
}

#[test]
fn a_word_wider_than_the_measure_is_overfull_but_laid_out() {
    let spans = [Span::new(
        "Pneumonoultramicroscopicsilicovolcanoconiosis",
        style(),
    )];
    let out = layout(&spans, 40.0, |s| s.hyphenate = false);
    assert_eq!(out.lines.len(), 1);
    assert!(out.lines[0].overfull);
    assert_eq!(out.tolerance, None);
}

#[test]
fn live_typing_uses_first_fit_and_still_fits() {
    let spans = [Span::new(STORY, style())];
    let out = layout(&spans, 200.0, |s| s.optimal = false);
    assert!(out.lines.len() >= 4);
    assert!(out.lines.iter().all(|l| !l.overfull));
}

#[test]
fn leading_and_trailing_spaces_and_runs_of_spaces_collapse() {
    let spans = [Span::new("   one    two  ", style())];
    let out = layout(&spans, 300.0, |_| {});
    assert_eq!(words(&out.lines[0]), "one|two");
}

#[test]
fn a_missing_font_falls_back_rather_than_failing() {
    let spans = [Span::new("An otter", TextStyle::new("No Such Font", 12.0))];
    let out = layout(&spans, 300.0, |_| {});
    assert!(out.lines[0].width > 0.0);
}

#[test]
fn text_ranges_point_into_the_joined_text() {
    let spans = [
        Span::new("Counting ", style()),
        Span::new("the stars", style()),
    ];
    let out = layout(&spans, 300.0, |_| {});
    let range = out.lines[0].text.clone();
    assert_eq!(&out.text[range], "Counting the stars");
}

#[test]
fn a_blank_poem_line_knows_where_it_sits_in_the_text() {
    let poem = "First verse\n\nSecond verse";
    let spans = [Span::new(poem, style())];
    let out = layout(&spans, 300.0, |s| s.wrap = Wrap::Preserve);
    assert_eq!(out.lines[1].text, 12..12);
}

#[test]
fn a_no_break_space_keeps_its_words_together() {
    let text = "aaa\u{a0}bbb\u{a0}ccc\u{a0}ddd\u{a0}eee\u{a0}fff\u{a0}ggg";
    let spans = [Span::new(text, style())];
    let out = layout(&spans, 40.0, |s| s.hyphenate = false);
    // One unbreakable word, wider than the measure, rather than seven short lines.
    assert_eq!(out.lines.len(), 1);
}

#[test]
fn a_word_with_punctuation_is_still_hyphenated() {
    let plain = [Span::new("extraordinarily", style())];
    let comma = [Span::new("extraordinarily,", style())];
    let quoted = [Span::new("(extraordinarily)", style())];
    assert!(layout(&plain, 60.0, |_| {}).lines.len() >= 2);
    assert!(layout(&comma, 60.0, |_| {}).lines.len() >= 2);
    assert!(layout(&quoted, 60.0, |_| {}).lines.len() >= 2);
}
