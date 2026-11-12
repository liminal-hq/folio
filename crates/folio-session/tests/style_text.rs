// Drives `StyleText` through a session: a stretch of a story's or a cell's words takes a look
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use folio_model::{Book, FlowId, Frame, FrameId, Intent, RunChange, TextPos, TextTarget};
use folio_session::Session;
use serde_json::json;

fn book() -> Book {
    serde_json::from_value(json!({
        "schema": 1, "id": "bk_t", "title": "T", "authors": [], "language": "en-CA",
        "kind": "picture", "layout": "fixed",
        "trim": { "w": 612, "h": 612 }, "bleed": 9,
        "margins": { "top": 36, "bottom": 36, "inside": 36, "outside": 36 },
        "grid": { "columns": 6, "gutter": 12, "baseline": null },
        "spreads": [
            { "id": "sp_cover", "kind": "cover", "pages": [{ "id": "pg_c", "master": null }], "frames": [] },
            { "id": "sp_1", "kind": "opener", "pages": [{ "id": "pg_1", "master": null }], "frames": [
                { "id": "fr_flowed", "type": "text", "box": { "x": 0, "y": 300, "w": 50, "h": 50 },
                  "style": "body", "flow": { "id": "fl_main", "index": 0 } },
                { "id": "fr_locked", "type": "text", "locked": true, "box": { "x": 10, "y": 220, "w": 200, "h": 40 },
                  "style": "body", "content": { "inline": [{ "text": "Fixed" }] } },
                { "id": "fr_table", "type": "table", "box": { "x": 0, "y": 400, "w": 200, "h": 40 },
                  "cols": [0.5, 0.5], "rows": [[{ "content": { "inline": [{ "text": "one two\nthree" }] } }, {}]] }
            ] }
        ],
        "flows": [{ "id": "fl_main", "frames": ["fr_flowed"], "paragraphs": [
            { "inline": [{ "text": "A chapter" }] },
            { "inline": [{ "text": "The otter slept." }] },
            { "inline": [{ "text": "The end" }] }
        ] }],
        "metadata": { "isbn": { "paperback": null, "hardcover": null, "ebook": null }, "description": "" }
    }))
    .unwrap()
}

fn at(para: u32, offset: u32) -> TextPos {
    TextPos { para, offset }
}

fn bold() -> RunChange {
    RunChange {
        bold: Some(true),
        ..RunChange::default()
    }
}

fn story() -> TextTarget {
    TextTarget::Story {
        flow: FlowId("fl_main".into()),
    }
}

fn cell() -> TextTarget {
    TextTarget::Cell {
        frame: FrameId("fr_table".into()),
        row: 0,
        col: 0,
    }
}

fn style(target: TextTarget, from: TextPos, to: TextPos, change: RunChange) -> Intent {
    Intent::StyleText {
        target,
        from,
        to,
        change,
    }
}

/// The bold stretches of each paragraph of the story.
fn story_bold(s: &Session) -> Vec<Vec<String>> {
    s.book().flows[0]
        .paragraphs
        .iter()
        .map(|p| {
            p.inline
                .iter()
                .filter(|r| r.bold)
                .map(|r| r.text.clone())
                .collect()
        })
        .collect()
}

fn cell_runs(s: &Session) -> Vec<(String, bool)> {
    let Frame::Table(t) = &s.book().spreads[1].frames[2] else {
        panic!("not a table");
    };
    t.rows[0][0]
        .content
        .as_ref()
        .unwrap()
        .inline
        .iter()
        .map(|r| (r.text.clone(), r.bold))
        .collect()
}

#[test]
fn a_stretch_inside_one_paragraph_of_a_story_takes_the_look() {
    let mut s = Session::new(book());
    s.apply(style(story(), at(1, 4), at(1, 9), bold())).unwrap();
    assert_eq!(
        story_bold(&s),
        vec![vec![], vec!["otter".to_owned()], vec![]]
    );
}

#[test]
fn a_stretch_across_paragraphs_styles_the_end_of_one_and_the_start_of_the_next() {
    let mut s = Session::new(book());
    s.apply(style(story(), at(0, 2), at(2, 3), bold())).unwrap();
    assert_eq!(
        story_bold(&s),
        vec![
            vec!["chapter".to_owned()],
            vec!["The otter slept.".to_owned()],
            vec!["The".to_owned()],
        ]
    );
    // One step undoes all three paragraphs.
    s.undo().unwrap();
    assert_eq!(story_bold(&s), vec![Vec::<String>::new(); 3]);
}

#[test]
fn a_stretch_of_a_cells_words_takes_the_look_across_its_paragraphs() {
    let mut s = Session::new(book());
    s.apply(style(cell(), at(0, 4), at(1, 2), bold())).unwrap();
    assert_eq!(
        cell_runs(&s),
        vec![
            ("one ".into(), false),
            ("two".into(), true),
            ("\nth".into(), true),
            ("ree".into(), false)
        ]
    );
}

#[test]
fn nothing_selected_a_bad_place_a_bad_colour_and_a_locked_frame_are_refused() {
    let mut s = Session::new(book());
    let before = s.revision();
    for bad in [
        style(story(), at(1, 3), at(1, 3), bold()),
        style(story(), at(1, 3), at(1, 99), bold()),
        style(story(), at(2, 0), at(1, 0), bold()),
        style(story(), at(1, 0), at(1, 3), RunChange::default()),
        style(
            story(),
            at(1, 0),
            at(1, 3),
            RunChange {
                colour: Some("red".into()),
                ..RunChange::default()
            },
        ),
        style(
            TextTarget::Cell {
                frame: FrameId("fr_table".into()),
                row: 4,
                col: 0,
            },
            at(0, 0),
            at(0, 1),
            bold(),
        ),
    ] {
        assert!(s.apply(bad).is_err());
    }
    let locked = TextTarget::Frame {
        frame: FrameId("fr_locked".into()),
    };
    assert!(s.apply(style(locked, at(0, 0), at(0, 2), bold())).is_err());
    assert_eq!(s.revision(), before);
}
