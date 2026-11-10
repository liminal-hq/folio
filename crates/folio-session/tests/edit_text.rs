// Drives `EditText` through a session: what is typed, how it joins undo steps, and what it refuses
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::time::{Duration, Instant};

use folio_model::{
    Book, EditStep, FlowId, Frame, FrameId, Intent, RunChange, TextInsert, TextPos, TextTarget,
};
use folio_session::{Session, SessionError, COALESCE_WINDOW};
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
                { "id": "fr_text", "type": "text", "box": { "x": 10, "y": 120, "w": 200, "h": 40 },
                  "style": "body", "content": { "inline": [{ "text": "Once " }, { "text": "upon", "bold": true }, { "text": " a time" }] } },
                { "id": "fr_locked", "type": "text", "locked": true, "box": { "x": 10, "y": 220, "w": 200, "h": 40 },
                  "style": "body", "content": { "inline": [{ "text": "Fixed" }] } },
                { "id": "fr_flowed", "type": "text", "box": { "x": 0, "y": 300, "w": 50, "h": 50 },
                  "style": "body", "flow": { "id": "fl_main", "index": 0 } },
                { "id": "fr_table", "type": "table", "box": { "x": 0, "y": 400, "w": 200, "h": 40 },
                  "cols": [0.5, 0.5], "rows": [[{ "content": { "inline": [{ "text": "a" }] } }, {}]] }
            ] }
        ],
        "flows": [{ "id": "fl_main", "frames": ["fr_flowed"], "paragraphs": [
            { "inline": [{ "text": "A chapter" }] },
            { "inline": [{ "text": "The otter slept." }] }
        ] }],
        "metadata": { "isbn": { "paperback": null, "hardcover": null, "ebook": null }, "description": "" }
    }))
    .unwrap()
}

fn at(para: u32, offset: u32) -> TextPos {
    TextPos { para, offset }
}

fn frame_target() -> TextTarget {
    TextTarget::Frame {
        frame: FrameId("fr_text".into()),
    }
}

fn typed(target: TextTarget, from: TextPos, to: TextPos, text: &str, step: EditStep) -> Intent {
    Intent::EditText {
        target,
        from,
        to,
        insert: TextInsert::Text { text: text.into() },
        look: None,
        step,
    }
}

fn words(s: &Session, frame: &str) -> Vec<(String, bool)> {
    let sp = &s.book().spreads[1];
    match sp.frames.iter().find(|f| f.base().id.0 == frame).unwrap() {
        Frame::Text(t) => t
            .content
            .as_ref()
            .unwrap()
            .inline
            .iter()
            .map(|r| (r.text.clone(), r.bold))
            .collect(),
        _ => panic!("not text"),
    }
}

#[test]
fn typing_in_a_frame_keeps_the_look_of_the_words_around_it() {
    let mut s = Session::new(book());
    s.apply(typed(
        frame_target(),
        at(0, 9),
        at(0, 9),
        "X",
        EditStep::Typing,
    ))
    .unwrap();
    assert_eq!(
        words(&s, "fr_text"),
        vec![
            ("Once ".into(), false),
            ("uponX".into(), true),
            (" a time".into(), false)
        ]
    );
}

#[test]
fn keystrokes_that_follow_on_make_one_step_and_a_jump_makes_a_new_one() {
    let mut s = Session::new(book());
    let t0 = Instant::now();
    let step = Duration::from_millis(100);
    for (i, ch) in ["a", "b", "c"].iter().enumerate() {
        let n = i as u32;
        s.apply_at(
            typed(frame_target(), at(0, n), at(0, n), ch, EditStep::Typing),
            t0 + step * n,
        )
        .unwrap();
    }
    assert_eq!(s.undo_depth(), 1, "typing in a row is one step");
    // A caret that jumped: this one begins away from where the last ended.
    s.apply_at(
        typed(frame_target(), at(0, 10), at(0, 10), "z", EditStep::Typing),
        t0 + step * 4,
    )
    .unwrap();
    assert_eq!(s.undo_depth(), 2);
    // Backspace over what was just typed joins the typing too.
    s.apply_at(
        typed(frame_target(), at(0, 10), at(0, 11), "", EditStep::Typing),
        t0 + step * 5,
    )
    .unwrap();
    assert_eq!(s.undo_depth(), 2);
    s.undo().unwrap();
    s.undo().unwrap();
    assert_eq!(words(&s, "fr_text")[0].0, "Once ");
}

#[test]
fn a_pause_of_more_than_a_second_starts_a_new_step_and_a_paste_is_its_own() {
    let mut s = Session::new(book());
    let t0 = Instant::now();
    s.apply_at(
        typed(frame_target(), at(0, 0), at(0, 0), "a", EditStep::Typing),
        t0,
    )
    .unwrap();
    // Within a second but past the general window: still the same typing.
    s.apply_at(
        typed(frame_target(), at(0, 1), at(0, 1), "b", EditStep::Typing),
        t0 + COALESCE_WINDOW + Duration::from_millis(100),
    )
    .unwrap();
    assert_eq!(s.undo_depth(), 1);
    s.apply_at(
        typed(frame_target(), at(0, 2), at(0, 2), "c", EditStep::Typing),
        t0 + Duration::from_millis(2500),
    )
    .unwrap();
    assert_eq!(s.undo_depth(), 2);
    s.apply_at(
        typed(frame_target(), at(0, 3), at(0, 3), "paste", EditStep::Own),
        t0 + Duration::from_millis(2600),
    )
    .unwrap();
    s.apply_at(
        typed(frame_target(), at(0, 8), at(0, 8), "d", EditStep::Typing),
        t0 + Duration::from_millis(2700),
    )
    .unwrap();
    assert_eq!(
        s.undo_depth(),
        4,
        "a paste does not merge with typing either side"
    );
}

#[test]
fn enter_splits_a_story_paragraph_and_the_look_set_at_the_caret_applies() {
    let mut s = Session::new(book());
    let story = TextTarget::Story {
        flow: FlowId("fl_main".into()),
    };
    s.apply(typed(
        story.clone(),
        at(1, 3),
        at(1, 3),
        "\n",
        EditStep::Own,
    ))
    .unwrap();
    let flow = &s.book().flows[0];
    assert_eq!(flow.paragraphs.len(), 3);
    assert_eq!(flow.paragraphs[1].text(), "The");
    assert_eq!(flow.paragraphs[2].text(), " otter slept.");
    let bold = Intent::EditText {
        target: story,
        from: at(2, 0),
        to: at(2, 0),
        insert: TextInsert::Text { text: "Hi".into() },
        look: Some(RunChange {
            bold: Some(true),
            ..RunChange::default()
        }),
        step: EditStep::Typing,
    };
    s.apply(bold).unwrap();
    assert!(s.book().flows[0].paragraphs[2].inline[0].bold);
    assert_eq!(s.book().flows[0].paragraphs[2].inline[0].text, "Hi");
}

#[test]
fn a_cell_takes_edits_and_a_locked_or_flowed_frame_does_not() {
    let mut s = Session::new(book());
    s.apply(typed(
        TextTarget::Cell {
            frame: FrameId("fr_table".into()),
            row: 0,
            col: 1,
        },
        at(0, 0),
        at(0, 0),
        "hello",
        EditStep::Typing,
    ))
    .unwrap();
    let Frame::Table(t) = s.book().spreads[1]
        .frames
        .iter()
        .find(|f| f.base().id.0 == "fr_table")
        .unwrap()
    else {
        panic!()
    };
    assert_eq!(
        t.rows[0][1].content.as_ref().unwrap().inline[0].text,
        "hello"
    );

    let locked = typed(
        TextTarget::Frame {
            frame: FrameId("fr_locked".into()),
        },
        at(0, 0),
        at(0, 0),
        "x",
        EditStep::Own,
    );
    assert!(matches!(s.apply(locked), Err(SessionError::Locked(_))));
    let flowed = typed(
        TextTarget::Frame {
            frame: FrameId("fr_flowed".into()),
        },
        at(0, 0),
        at(0, 0),
        "x",
        EditStep::Own,
    );
    assert!(matches!(s.apply(flowed), Err(SessionError::FlowText(_))));
}

#[test]
fn a_range_outside_the_words_is_refused_and_changes_nothing() {
    let mut s = Session::new(book());
    let rev = s.revision();
    for (from, to) in [
        (at(0, 99), at(0, 99)),
        (at(3, 0), at(3, 0)),
        (at(0, 5), at(0, 2)),
    ] {
        let err = s
            .apply(typed(frame_target(), from, to, "x", EditStep::Own))
            .unwrap_err();
        assert!(matches!(err, SessionError::BadEdit(_)), "{err:?}");
    }
    assert_eq!(s.revision(), rev);
}

#[test]
fn emptying_a_frame_keeps_the_look_of_its_last_words() {
    let mut s = Session::new(book());
    s.apply(typed(
        frame_target(),
        at(0, 0),
        at(0, 16),
        "",
        EditStep::Own,
    ))
    .unwrap();
    let w = words(&s, "fr_text");
    assert_eq!(w.len(), 1);
    assert_eq!(w[0].0, "");
    s.apply(typed(
        frame_target(),
        at(0, 0),
        at(0, 0),
        "new",
        EditStep::Typing,
    ))
    .unwrap();
    assert_eq!(words(&s, "fr_text")[0].0, "new");
}

#[test]
fn undo_and_redo_say_where_the_caret_goes() {
    let mut s = Session::new(book());
    let t0 = Instant::now();
    // Typing "ab" at the end of the first paragraph, one key after the other, is one step.
    s.apply_at(
        typed(frame_target(), at(0, 16), at(0, 16), "a", EditStep::Typing),
        t0,
    )
    .unwrap();
    s.apply_at(
        typed(frame_target(), at(0, 17), at(0, 17), "b", EditStep::Typing),
        t0 + Duration::from_millis(100),
    )
    .unwrap();
    assert!(s.restored_caret().is_none(), "a new change forgets it");
    s.undo().unwrap();
    let back = s
        .restored_caret()
        .expect("an edit says where the caret was");
    assert_eq!((back.anchor, back.focus), (at(0, 16), at(0, 16)));
    s.redo().unwrap();
    let forward = s.restored_caret().unwrap();
    assert_eq!((forward.anchor, forward.focus), (at(0, 18), at(0, 18)));
    // A replaced selection comes back as a selection.
    s.apply(typed(
        frame_target(),
        at(0, 0),
        at(0, 4),
        "X",
        EditStep::Own,
    ))
    .unwrap();
    s.undo().unwrap();
    let selected = s.restored_caret().unwrap();
    assert_eq!((selected.anchor, selected.focus), (at(0, 0), at(0, 4)));
    // A step that is not an edit to words has none.
    s.apply(Intent::SetText {
        frame: FrameId("fr_text".into()),
        text: "plain".into(),
    })
    .unwrap();
    s.undo().unwrap();
    assert!(s.restored_caret().is_none());
}
