// Tests for the Markdown importer: structure becomes paragraph styles and runs, and the rest is reported
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use folio_import::import_markdown;

fn style(p: &folio_model::StoryPara) -> Option<&str> {
    p.style.as_ref().map(|s| s.0.as_str())
}

#[test]
fn headings_paragraphs_and_scene_breaks_become_a_styled_story() {
    let made = import_markdown(
        "# The keeper's kettle\n\nMara had kept the light.\n\nShe did not open it.\n\n***\n\n## Later\n\nThe boat came.\n",
    );
    let shown: Vec<(Option<&str>, String)> =
        made.story.iter().map(|p| (style(p), p.text())).collect();
    assert_eq!(
        shown,
        vec![
            (Some("h1"), "The keeper\u{2019}s kettle".into()),
            (None, "Mara had kept the light.".into()),
            (None, "She did not open it.".into()),
            (Some("ornament"), "* * *".into()),
            (Some("h2"), "Later".into()),
            (None, "The boat came.".into()),
        ]
    );
    assert_eq!(made.report.paragraphs, 6);
    assert_eq!(made.report.headings, 2);
    assert_eq!(made.report.scene_breaks, 1);
    assert!(made.report.notes.is_empty());
}

#[test]
fn emphasis_is_kept_as_runs_and_quotes_take_the_epigraph_style() {
    let made = import_markdown("She said *very* **loudly**.\n\n> The sea keeps its own hours.\n");
    let runs = &made.story[0].inline;
    assert!(runs.iter().any(|r| r.italic && r.text == "very"));
    assert!(runs.iter().any(|r| r.bold && r.text == "loudly"));
    assert_eq!(made.story[0].text(), "She said very loudly.");
    assert_eq!(style(&made.story[1]), Some("epigraph"));
}

#[test]
fn plain_text_is_paragraphs_split_at_blank_lines_with_curly_quotes() {
    let made = import_markdown("It was \"quiet\".\nVery quiet.\n\nThen it wasn't.");
    assert_eq!(made.story.len(), 2);
    assert_eq!(
        made.story[0].text(),
        "It was \u{201c}quiet\u{201d}. Very quiet."
    );
    assert_eq!(made.story[1].text(), "Then it wasn\u{2019}t.");
}

#[test]
fn what_cannot_come_across_is_left_out_and_said_once() {
    let made = import_markdown(
        "Words.\n\n![a picture](x.png)\n\n```\ncode\n```\n\n| a | b |\n|---|---|\n| 1 | 2 |\n\n- one\n- two\n\n[a link](https://example.org) here.\n",
    );
    let text: Vec<String> = made.story.iter().map(|p| p.text()).collect();
    assert!(text.contains(&"Words.".to_string()));
    assert!(text.contains(&"\u{2013} one".to_string()));
    assert!(text.contains(&"a link here.".to_string()));
    assert!(!text
        .iter()
        .any(|t| t.contains("code") || t.contains("x.png")));
    for note in ["Pictures", "Code blocks", "Tables", "List items", "Links"] {
        assert_eq!(
            made.report
                .notes
                .iter()
                .filter(|n| n.starts_with(note))
                .count(),
            1,
            "{note}"
        );
    }
}

#[test]
fn an_empty_document_is_an_empty_story() {
    let made = import_markdown("  \n\n");
    assert!(made.story.is_empty());
    assert_eq!(made.report.paragraphs, 0);
}
