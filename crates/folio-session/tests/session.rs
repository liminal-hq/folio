// Drives a session the way the UI will: intents in, patches out, undo and redo
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::time::{Duration, Instant};

use folio_model::{
    Book, Frame, FrameId, Grid, Intent, Margins, Pt, Rect, ReorderTo, Size, SpreadId, SpreadKind,
    StyleId, StyleOverrides,
};
use folio_session::{PatchOp, Session, SessionError, COALESCE_WINDOW, UNDO_DEPTH};
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
                { "id": "fr_pic", "type": "image", "box": { "x": 0, "y": 0, "w": 100, "h": 100 },
                  "image": { "asset": null, "fit": "fill", "focus": [0.5, 0.5], "crop": null } },
                { "id": "fr_text", "type": "text", "box": { "x": 10, "y": 120, "w": 200, "h": 40 },
                  "style": "body", "content": { "inline": [{ "text": "Once upon a time" }] } },
                { "id": "fr_flowed", "type": "text", "box": { "x": 0, "y": 200, "w": 50, "h": 50 },
                  "style": "body", "flow": { "id": "fl_main", "index": 0 } }
            ] },
            { "id": "sp_2", "kind": "spread", "pages": [{ "id": "pg_2", "master": null }, { "id": "pg_3", "master": null }], "frames": [] }
        ],
        "metadata": { "isbn": { "paperback": null, "hardcover": null, "ebook": null }, "description": "" }
    }))
    .unwrap()
}

/// A session whose new ids count up, so tests can name them.
fn session() -> Session {
    let mut next = 0;
    Session::with_ids(
        book(),
        Box::new(move |prefix| {
            next += 1;
            format!("{prefix}_new{next}")
        }),
    )
}

fn rect(x: f64) -> Rect {
    Rect {
        x: Pt(x),
        y: Pt(0.0),
        w: Pt(100.0),
        h: Pt(100.0),
    }
}

fn spread(id: &str) -> SpreadId {
    SpreadId(id.into())
}

fn frame(id: &str) -> FrameId {
    FrameId(id.into())
}

fn move_pic(x: f64) -> Intent {
    Intent::MoveFrame {
        spread: spread("sp_1"),
        frame: frame("fr_pic"),
        rect: rect(x),
    }
}

fn pic_x(s: &Session) -> f64 {
    let sp = s.book().spreads.iter().find(|s| s.id.0 == "sp_1").unwrap();
    sp.frames
        .iter()
        .find(|f| f.base().id.0 == "fr_pic")
        .unwrap()
        .base()
        .rect
        .x
        .0
}

#[test]
fn an_intent_changes_the_book_and_answers_with_a_patch() {
    let mut s = session();
    let patch = s.apply(move_pic(40.0)).unwrap();
    assert_eq!(pic_x(&s), 40.0);
    assert_eq!(patch.revision, 1);
    assert_eq!(patch.ops.len(), 1);
    assert!(matches!(&patch.ops[0], PatchOp::SetSpread { spread } if spread.id.0 == "sp_1"));
    assert!(s.is_dirty());
}

#[test]
fn undo_and_redo_walk_the_changes_and_each_is_a_new_revision() {
    let mut s = session();
    let start = Instant::now();
    s.apply_at(move_pic(40.0), start).unwrap();
    // Far enough apart that they are separate steps.
    s.apply_at(move_pic(80.0), start + COALESCE_WINDOW * 3)
        .unwrap();
    assert_eq!(pic_x(&s), 80.0);

    let undone = s.undo().unwrap();
    assert_eq!(pic_x(&s), 40.0);
    assert_eq!(undone.revision, 3);
    s.undo().unwrap();
    assert_eq!(pic_x(&s), 0.0);
    assert!(s.undo().is_none(), "nothing left to undo");

    s.redo().unwrap();
    assert_eq!(pic_x(&s), 40.0);
    let redone = s.redo().unwrap();
    assert_eq!(pic_x(&s), 80.0);
    assert_eq!(redone.revision, 6);
    assert!(s.redo().is_none());
}

#[test]
fn a_new_change_clears_what_could_be_redone() {
    let mut s = session();
    s.apply(move_pic(40.0)).unwrap();
    s.undo().unwrap();
    assert!(s.can_redo());
    s.apply(Intent::SetText {
        frame: frame("fr_text"),
        text: "New".into(),
    })
    .unwrap();
    assert!(!s.can_redo());
}

#[test]
fn a_drag_is_one_undo_step_but_a_pause_starts_a_new_one() {
    let mut s = session();
    let start = Instant::now();
    for (i, x) in [10.0, 20.0, 30.0, 40.0].into_iter().enumerate() {
        s.apply_at(move_pic(x), start + Duration::from_millis(100 * i as u64))
            .unwrap();
    }
    assert_eq!(
        s.revision(),
        4,
        "every move is a revision, so the UI hears each"
    );
    s.undo().unwrap();
    assert_eq!(pic_x(&s), 0.0, "the whole drag undoes at once");

    // After a pause longer than the window, a move is its own step.
    let mut s = session();
    s.apply_at(move_pic(10.0), start).unwrap();
    s.apply_at(
        move_pic(20.0),
        start + COALESCE_WINDOW + Duration::from_millis(1),
    )
    .unwrap();
    s.undo().unwrap();
    assert_eq!(pic_x(&s), 10.0);
}

#[test]
fn moves_of_different_frames_are_not_merged() {
    let mut s = session();
    let start = Instant::now();
    s.apply_at(move_pic(10.0), start).unwrap();
    s.apply_at(
        Intent::MoveFrame {
            spread: spread("sp_1"),
            frame: frame("fr_text"),
            rect: rect(5.0),
        },
        start + Duration::from_millis(50),
    )
    .unwrap();
    s.undo().unwrap();
    assert_eq!(pic_x(&s), 10.0, "only the text frame's move was undone");
}

#[test]
fn a_refused_intent_changes_nothing_and_says_why() {
    let mut s = session();
    let refused = s
        .apply(Intent::MoveFrame {
            spread: spread("sp_1"),
            frame: frame("nope"),
            rect: rect(1.0),
        })
        .unwrap_err();
    assert_eq!(refused, SessionError::NoSuchFrame("nope".into()));
    assert_eq!(s.revision(), 0);
    assert!(!s.is_dirty());
    assert!(!s.can_undo());
}

#[test]
fn a_move_to_where_the_frame_already_is_changes_nothing() {
    let mut s = session();
    let patch = s.apply(move_pic(0.0)).unwrap();
    assert!(patch.ops.is_empty());
    assert_eq!(patch.revision, 0);
    assert!(!s.can_undo());
}

#[test]
fn text_is_set_in_place_but_not_on_pictures_or_flowed_frames() {
    let mut s = session();
    s.apply(Intent::SetText {
        frame: frame("fr_text"),
        text: "Hello".into(),
    })
    .unwrap();
    let sp = &s.book().spreads[1];
    let Frame::Text(t) = &sp.frames[1] else {
        panic!("text")
    };
    assert_eq!(t.content.as_ref().unwrap().inline[0].text, "Hello");

    assert_eq!(
        s.apply(Intent::SetText {
            frame: frame("fr_pic"),
            text: "x".into()
        })
        .unwrap_err(),
        SessionError::NotText("fr_pic".into())
    );
    assert_eq!(
        s.apply(Intent::SetText {
            frame: frame("fr_flowed"),
            text: "x".into()
        })
        .unwrap_err(),
        SessionError::FlowText("fr_flowed".into())
    );
}

#[test]
fn styles_are_set_and_overridden_on_text_frames() {
    let mut s = session();
    s.apply(Intent::SetStyle {
        frame: frame("fr_text"),
        style: StyleId("display".into()),
    })
    .unwrap();
    s.apply(Intent::OverrideStyle {
        frame: frame("fr_text"),
        overrides: StyleOverrides {
            size: Some(Pt(24.0)),
            ..StyleOverrides::default()
        },
    })
    .unwrap();
    let Frame::Text(t) = &s.book().spreads[1].frames[1] else {
        panic!("text")
    };
    assert_eq!(t.style.0, "display");
    assert_eq!(t.overrides.as_ref().unwrap().size, Some(Pt(24.0)));
    assert!(s
        .apply(Intent::SetStyle {
            frame: frame("fr_pic"),
            style: StyleId("x".into())
        })
        .is_err());
}

#[test]
fn frames_are_added_deleted_and_reordered() {
    let mut s = session();
    let added: Frame = serde_json::from_value(json!({
        "type": "shape", "id": "fr_box", "box": { "x": 0, "y": 0, "w": 5, "h": 5 },
        "shape": "rect", "fill": null, "style": null
    }))
    .unwrap();
    s.apply(Intent::AddFrame {
        spread: spread("sp_1"),
        frame: added.clone(),
    })
    .unwrap();
    assert_eq!(s.book().spreads[1].frames.len(), 4);
    assert_eq!(
        s.apply(Intent::AddFrame {
            spread: spread("sp_1"),
            frame: added
        })
        .unwrap_err(),
        SessionError::DuplicateFrame("fr_box".into()),
        "an id is used once in the whole book"
    );

    s.apply(Intent::Reorder {
        spread: spread("sp_1"),
        frame: frame("fr_pic"),
        to: ReorderTo::Front,
    })
    .unwrap();
    let ids = |s: &Session| -> Vec<String> {
        s.book().spreads[1]
            .frames
            .iter()
            .map(|f| f.base().id.0.clone())
            .collect()
    };
    assert_eq!(ids(&s), ["fr_text", "fr_flowed", "fr_box", "fr_pic"]);
    s.apply(Intent::Reorder {
        spread: spread("sp_1"),
        frame: frame("fr_pic"),
        to: ReorderTo::Back,
    })
    .unwrap();
    assert_eq!(ids(&s)[0], "fr_pic");
    s.apply(Intent::Reorder {
        spread: spread("sp_1"),
        frame: frame("fr_pic"),
        to: ReorderTo::Index(2),
    })
    .unwrap();
    assert_eq!(ids(&s)[2], "fr_pic");

    s.apply(Intent::DeleteFrame {
        spread: spread("sp_1"),
        frame: frame("fr_box"),
    })
    .unwrap();
    assert_eq!(s.book().spreads[1].frames.len(), 3);
    s.undo().unwrap();
    s.undo().unwrap();
    s.undo().unwrap();
    s.undo().unwrap();
    s.undo().unwrap();
    assert_eq!(
        s.book().spreads[1].frames.len(),
        3,
        "back to the three the book started with"
    );
    assert_eq!(s.book().spreads[1].frames[0].base().id.0, "fr_pic");
}

#[test]
fn a_new_spread_goes_after_the_one_named_with_fresh_pages() {
    let mut s = session();
    let patch = s
        .apply(Intent::AddSpread {
            after: spread("sp_1"),
            duplicate: false,
        })
        .unwrap();
    let ids: Vec<_> = s.book().spreads.iter().map(|s| s.id.0.as_str()).collect();
    assert_eq!(ids, ["sp_cover", "sp_1", "sp_new1", "sp_2"]);
    let made = &s.book().spreads[2];
    assert_eq!(made.kind, SpreadKind::Spread);
    assert_eq!(made.pages.len(), 2);
    assert!(patch
        .ops
        .iter()
        .any(|op| matches!(op, PatchOp::SetOrder { .. })));

    // After the cover, the first page opens on the right.
    s.apply(Intent::AddSpread {
        after: spread("sp_cover"),
        duplicate: false,
    })
    .unwrap();
    assert_eq!(s.book().spreads[1].kind, SpreadKind::Opener);
    assert_eq!(s.book().spreads[1].pages.len(), 1);
}

#[test]
fn a_duplicated_spread_shares_no_ids_with_its_source() {
    let mut s = session();
    s.apply(Intent::AddSpread {
        after: spread("sp_1"),
        duplicate: true,
    })
    .unwrap();
    let source = s.book().spreads[1].clone();
    let copy = s.book().spreads[2].clone();
    assert_eq!(copy.frames.len(), source.frames.len());
    assert_eq!(copy.kind, source.kind);
    for (a, b) in source.frames.iter().zip(&copy.frames) {
        assert_ne!(a.base().id, b.base().id);
        assert_eq!(a.base().rect, b.base().rect);
    }
    assert_ne!(source.pages[0].id, copy.pages[0].id);
}

#[test]
fn spreads_are_deleted_and_moved_but_the_cover_and_last_spread_stay() {
    let mut s = session();
    assert_eq!(
        s.apply(Intent::DeleteSpread {
            spread: spread("sp_cover")
        })
        .unwrap_err(),
        SessionError::CoverFixed
    );
    assert_eq!(
        s.apply(Intent::MoveSpread {
            spread: spread("sp_cover"),
            to: 2
        })
        .unwrap_err(),
        SessionError::CoverFixed
    );

    // Nothing moves in front of the cover.
    s.apply(Intent::MoveSpread {
        spread: spread("sp_2"),
        to: 0,
    })
    .unwrap();
    let ids: Vec<_> = s.book().spreads.iter().map(|s| s.id.0.as_str()).collect();
    assert_eq!(ids, ["sp_cover", "sp_2", "sp_1"]);

    s.apply(Intent::DeleteSpread {
        spread: spread("sp_1"),
    })
    .unwrap();
    s.apply(Intent::DeleteSpread {
        spread: spread("sp_2"),
    })
    .unwrap();
    assert_eq!(s.book().spreads.len(), 1);

    let mut only = Session::new(
        serde_json::from_value(json!({
            "schema": 1, "id": "b", "title": "T", "authors": [], "language": "en", "kind": "novel", "layout": "flowing",
            "trim": { "w": 360, "h": 576 }, "bleed": 0,
            "margins": { "top": 1, "bottom": 1, "inside": 1, "outside": 1 },
            "grid": { "columns": 1, "gutter": 0, "baseline": null },
            "spreads": [{ "id": "only", "kind": "single", "pages": [{ "id": "p", "master": null }], "frames": [] }],
            "metadata": { "isbn": { "paperback": null, "hardcover": null, "ebook": null }, "description": "" }
        }))
        .unwrap(),
    );
    assert_eq!(
        only.apply(Intent::DeleteSpread {
            spread: spread("only")
        })
        .unwrap_err(),
        SessionError::LastSpread
    );
}

#[test]
fn deleting_and_moving_spreads_undo_to_the_same_order() {
    let mut s = session();
    s.apply(Intent::DeleteSpread {
        spread: spread("sp_1"),
    })
    .unwrap();
    s.undo().unwrap();
    let ids: Vec<_> = s.book().spreads.iter().map(|s| s.id.0.as_str()).collect();
    assert_eq!(ids, ["sp_cover", "sp_1", "sp_2"]);

    s.apply(Intent::MoveSpread {
        spread: spread("sp_2"),
        to: 1,
    })
    .unwrap();
    s.undo().unwrap();
    let ids: Vec<_> = s.book().spreads.iter().map(|s| s.id.0.as_str()).collect();
    assert_eq!(ids, ["sp_cover", "sp_1", "sp_2"]);
}

#[test]
fn page_geometry_changes_by_parts_and_undoes() {
    let mut s = session();
    let patch = s
        .apply(Intent::SetBookGeometry {
            trim: Some(Size {
                w: Pt(504.0),
                h: Pt(504.0),
            }),
            margins: None,
            bleed: Some(Pt(0.0)),
            grid: Some(Grid {
                columns: 12,
                gutter: Pt(6.0),
                baseline: None,
            }),
        })
        .unwrap();
    assert!(matches!(&patch.ops[0], PatchOp::SetGeometry { bleed, .. } if *bleed == Pt(0.0)));
    assert_eq!(s.book().trim.w, Pt(504.0));
    assert_eq!(
        s.book().margins,
        Margins {
            top: Pt(36.0),
            bottom: Pt(36.0),
            inside: Pt(36.0),
            outside: Pt(36.0)
        }
    );
    s.undo().unwrap();
    assert_eq!(s.book().trim.w, Pt(612.0));
    assert_eq!(s.book().grid.columns, 6);
}

#[test]
fn the_book_is_clean_again_after_a_save_and_after_undoing_back_to_it() {
    let mut s = session();
    s.apply(move_pic(40.0)).unwrap();
    assert!(s.is_dirty());
    s.mark_saved();
    assert!(!s.is_dirty());
    s.apply(Intent::SetText {
        frame: frame("fr_text"),
        text: "x".into(),
    })
    .unwrap();
    assert!(s.is_dirty());
}

#[test]
fn only_the_last_200_steps_can_be_undone() {
    let mut s = session();
    let start = Instant::now();
    for i in 0..(UNDO_DEPTH + 20) {
        // Each is far from the last, so none merge.
        s.apply_at(
            move_pic(i as f64 + 1.0),
            start + COALESCE_WINDOW * 3 * i as u32,
        )
        .unwrap();
    }
    let mut undone = 0;
    while s.undo().is_some() {
        undone += 1;
    }
    assert_eq!(undone, UNDO_DEPTH);
}
