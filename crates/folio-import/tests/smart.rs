// Smart punctuation on import: Markdown text and the characters of Word, OpenDocument and Rich Text files
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use folio_import::{import_markdown, import_markdown_with, smarten_story};
use folio_model::{text_of, QuoteStyle, Run, StoryPara};

fn words(story: &[StoryPara], para: usize) -> String {
    text_of(&story[para].inline)
}

fn run(text: &str, bold: bool) -> Run {
    Run {
        text: text.into(),
        style: None,
        bold,
        italic: false,
        font: None,
        colour: None,
    }
}

#[test]
fn markdown_double_hyphens_are_an_em_dash_like_everywhere_else() {
    let story = import_markdown("Wait--what... \"No,\" she said.").story;
    assert_eq!(words(&story, 0), "Wait—what… “No,” she said.");
}

#[test]
fn markdown_quotes_follow_the_book_language_and_code_is_left_alone() {
    let fr = import_markdown_with("Il dit \"oui\" et `a--b`.", Some(QuoteStyle::French)).story;
    assert_eq!(words(&fr, 0), "Il dit «\u{a0}oui\u{a0}» et a--b.");
}

#[test]
fn markdown_is_left_as_written_when_asked() {
    let story = import_markdown_with("Wait--what \"No\"", None).story;
    assert_eq!(words(&story, 0), "Wait--what \"No\"");
}

#[test]
fn a_rule_reaches_back_across_a_split_in_the_markdown_text() {
    // The parser splits text at an escape, so the second hyphen arrives as a text of its own.
    let story = import_markdown("a-\\-b").story;
    assert_eq!(words(&story, 0), "a—b");
}

#[test]
fn story_paragraphs_are_set_run_by_run_keeping_each_look() {
    let mut story = vec![StoryPara {
        style: None,
        look: None,
        inline: vec![run("He said \"wait-", false), run("-no\"...", true)],
    }];
    smarten_story(&mut story, QuoteStyle::Curly);
    assert_eq!(words(&story, 0), "He said “wait—no”…");
    // The dash was made where the second hyphen was typed, so it takes that run's look.
    assert!(story[0].inline.last().unwrap().bold);
}

#[test]
fn german_stories_get_low_opening_quotes() {
    let mut story = vec![StoryPara::plain("Er sagte \"nein\".")];
    smarten_story(&mut story, QuoteStyle::German);
    assert_eq!(words(&story, 0), "Er sagte „nein“.");
}
