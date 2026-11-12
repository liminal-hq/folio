// Drives `SetParagraphLook` through a session: each paragraph keeps its own look, and edits carry it
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use folio_model::{
    Align, Book, EditStep, Frame, FrameId, Intent, ParaChange, ParaField, ParaLook, TextInsert,
    TextPos, TextTarget,
};
use folio_session::{Session, SessionError};
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
                { "id": "fr_text", "type": "text", "box": { "x": 10, "y": 120, "w": 200, "h": 200 },
                  "style": "body", "content": { "inline": [{ "text": "One\nTwo\nThree" }] } },
                { "id": "fr_flowed", "type": "text", "box": { "x": 0, "y": 300, "w": 50, "h": 50 },
                  "style": "body", "flow": { "id": "fl_main", "index": 0 } }
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

fn frame_target() -> TextTarget {
    TextTarget::Frame {
        frame: FrameId("fr_text".into()),
    }
}

fn after(points: f64) -> ParaChange {
    ParaChange {
        set: ParaLook {
            space_after: Some(points),
            ..ParaLook::default()
        },
        clear: Vec::new(),
    }
}

fn set(target: TextTarget, from: u32, to: u32, change: ParaChange) -> Intent {
    Intent::SetParagraphLook {
        target,
        from,
        to,
        change,
    }
}

/// The `space_after` of each paragraph of the frame's own words.
fn spaces(s: &Session) -> Vec<Option<f64>> {
    let Frame::Text(t) = &s.book().spreads[1].frames[0] else {
        panic!("not text");
    };
    folio_model::paragraphs_of(t.content.as_ref())
        .iter()
        .map(|p| p.look.as_ref().and_then(|l| l.space_after))
        .collect()
}

fn text_of(s: &Session) -> String {
    let Frame::Text(t) = &s.book().spreads[1].frames[0] else {
        panic!("not text");
    };
    folio_model::text_of(&t.content.as_ref().unwrap().inline)
}

#[test]
fn a_look_reaches_only_the_paragraphs_named_and_undo_takes_it_back() {
    let mut s = Session::new(book());
    s.apply(set(frame_target(), 1, 1, after(12.0))).unwrap();
    assert_eq!(spaces(&s), vec![None, Some(12.0), None]);
    assert_eq!(text_of(&s), "One\nTwo\nThree");
    s.apply(set(frame_target(), 0, 2, after(4.0))).unwrap();
    assert_eq!(spaces(&s), vec![Some(4.0); 3]);
    s.undo().unwrap();
    assert_eq!(spaces(&s), vec![None, Some(12.0), None]);
    s.undo().unwrap();
    assert_eq!(spaces(&s), vec![None; 3]);
    s.redo().unwrap();
    assert_eq!(spaces(&s), vec![None, Some(12.0), None]);
}

#[test]
fn a_field_can_be_cleared_back_to_the_styles() {
    let mut s = Session::new(book());
    s.apply(set(frame_target(), 0, 0, after(9.0))).unwrap();
    s.apply(set(
        frame_target(),
        0,
        0,
        ParaChange {
            set: ParaLook::default(),
            clear: vec![ParaField::SpaceAfter],
        },
    ))
    .unwrap();
    assert_eq!(spaces(&s), vec![None; 3]);
}

#[test]
fn bad_values_and_places_are_refused_and_change_nothing() {
    let mut s = Session::new(book());
    let revision = s.revision();
    for bad in [after(-1.0), after(100_000.0), after(f64::NAN)] {
        assert!(matches!(
            s.apply(set(frame_target(), 0, 0, bad)),
            Err(SessionError::BadEdit(_))
        ));
    }
    assert!(s.apply(set(frame_target(), 0, 3, after(4.0))).is_err());
    assert!(s.apply(set(frame_target(), 2, 1, after(4.0))).is_err());
    assert_eq!(s.revision(), revision);
}

#[test]
fn enter_copies_the_look_and_editing_elsewhere_keeps_the_rest() {
    let mut s = Session::new(book());
    s.apply(set(frame_target(), 1, 1, after(12.0))).unwrap();
    // Enter in the middle of "Two".
    s.apply(Intent::EditText {
        target: frame_target(),
        from: TextPos { para: 1, offset: 1 },
        to: TextPos { para: 1, offset: 1 },
        insert: TextInsert::Text { text: "\n".into() },
        look: None,
        step: EditStep::Own,
    })
    .unwrap();
    assert_eq!(spaces(&s), vec![None, Some(12.0), Some(12.0), None]);
    // Typing in the last paragraph leaves the others' looks where they were.
    s.apply(Intent::EditText {
        target: frame_target(),
        from: TextPos { para: 3, offset: 0 },
        to: TextPos { para: 3, offset: 0 },
        insert: TextInsert::Text { text: "x".into() },
        look: None,
        step: EditStep::Typing,
    })
    .unwrap();
    assert_eq!(spaces(&s), vec![None, Some(12.0), Some(12.0), None]);
}

#[test]
fn a_story_paragraph_takes_a_look_of_its_own() {
    let mut s = Session::new(book());
    let story = TextTarget::Story {
        flow: folio_model::FlowId("fl_main".into()),
    };
    s.apply(set(
        story,
        1,
        1,
        ParaChange {
            set: ParaLook {
                align: Some(Align::Center),
                indent: Some(18.0),
                ..ParaLook::default()
            },
            clear: Vec::new(),
        },
    ))
    .unwrap();
    let look = |i: usize| s.book().flows[0].paragraphs[i].look.clone();
    assert_eq!(look(0), None);
    assert_eq!(look(1).and_then(|l| l.indent), Some(18.0));
    // Splitting a paragraph of the story gives both halves the look.
    let mut s = s;
    s.apply(Intent::SplitParagraph {
        flow: folio_model::FlowId("fl_main".into()),
        para: 1,
        at: 3,
    })
    .unwrap();
    assert_eq!(s.book().flows[0].paragraphs.len(), 3);
    assert_eq!(
        s.book().flows[0].paragraphs[2]
            .look
            .as_ref()
            .and_then(|l| l.align),
        Some(Align::Center)
    );
}

#[test]
fn a_book_written_before_looks_opens_and_one_without_looks_writes_none() {
    let json = serde_json::to_value(book()).unwrap();
    assert!(!json.to_string().contains("looks"));
    assert!(!json.to_string().contains("\"look\""));
}
