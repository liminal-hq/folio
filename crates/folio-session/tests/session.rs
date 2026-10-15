// Drives a session the way the UI will: intents in, patches out, undo and redo
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::time::{Duration, Instant};

use folio_model::{
    Axis, Book, Frame, FrameId, Grid, Intent, Margins, Pt, Rect, ReorderTo, Size, SpreadId,
    SpreadKind, StyleId, StyleOverrides, Surface,
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
        on: folio_model::Surface::Spread(spread("sp_1")),
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
            on: folio_model::Surface::Spread(spread("sp_1")),
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
            on: folio_model::Surface::Spread(spread("sp_1")),
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
        on: folio_model::Surface::Spread(spread("sp_1")),
        frame: added.clone(),
    })
    .unwrap();
    assert_eq!(s.book().spreads[1].frames.len(), 4);
    assert_eq!(
        s.apply(Intent::AddFrame {
            on: folio_model::Surface::Spread(spread("sp_1")),
            frame: added
        })
        .unwrap_err(),
        SessionError::DuplicateFrame("fr_box".into()),
        "an id is used once in the whole book"
    );

    s.apply(Intent::Reorder {
        on: folio_model::Surface::Spread(spread("sp_1")),
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
        on: folio_model::Surface::Spread(spread("sp_1")),
        frame: frame("fr_pic"),
        to: ReorderTo::Back,
    })
    .unwrap();
    assert_eq!(ids(&s)[0], "fr_pic");
    s.apply(Intent::Reorder {
        on: folio_model::Surface::Spread(spread("sp_1")),
        frame: frame("fr_pic"),
        to: ReorderTo::Index(2),
    })
    .unwrap();
    assert_eq!(ids(&s)[2], "fr_pic");

    s.apply(Intent::DeleteFrame {
        on: folio_model::Surface::Spread(spread("sp_1")),
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

// ── Fixtures the TypeScript side checks its own patch code against ──

/// Where the cases are kept: next to the code in the app that applies patches.
fn fixture_path() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../apps/folio/src/books/fixtures/patchCases.json")
}

/// Each case is a book, a patch produced for it by a real session, and the book after.
/// The TypeScript `applyPatch` must turn the first into the last. Run with `UPDATE_FIXTURES=1` to
/// rewrite the file after a deliberate change; otherwise the file must match what the code makes.
#[test]
fn the_patch_cases_the_ui_is_tested_against_match_what_the_session_produces() {
    let mut cases = Vec::new();
    let mut record =
        |name: &str, s: &mut Session, step: &dyn Fn(&mut Session) -> folio_session::Patch| {
            let before = serde_json::to_value(s.book()).unwrap();
            let patch = step(s);
            let after = serde_json::to_value(s.book()).unwrap();
            cases.push(json!({
                "name": name,
                "before": before,
                "patch": serde_json::to_value(&patch).unwrap(),
                "after": after,
            }));
        };

    let mut s = session();
    record("move a frame", &mut s, &|s| {
        s.apply(move_pic(40.0)).unwrap()
    });
    record("add a blank spread after the cover", &mut s, &|s| {
        s.apply(Intent::AddSpread {
            after: spread("sp_cover"),
            duplicate: false,
        })
        .unwrap()
    });
    record("duplicate a spread", &mut s, &|s| {
        s.apply(Intent::AddSpread {
            after: spread("sp_1"),
            duplicate: true,
        })
        .unwrap()
    });
    record("move a spread", &mut s, &|s| {
        s.apply(Intent::MoveSpread {
            spread: spread("sp_2"),
            to: 1,
        })
        .unwrap()
    });
    record("delete a spread", &mut s, &|s| {
        s.apply(Intent::DeleteSpread {
            spread: spread("sp_1"),
        })
        .unwrap()
    });
    record("undo the delete", &mut s, &|s| s.undo().unwrap());
    record("change the page geometry", &mut s, &|s| {
        s.apply(Intent::SetBookGeometry {
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
        .unwrap()
    });
    record("put page numbers on the outside", &mut s, &|s| {
        s.apply(Intent::SetPageNumbers {
            position: folio_model::PageNumbers::Outside,
        })
        .unwrap()
    });
    record("undo the page numbers", &mut s, &|s| s.undo().unwrap());

    let text = serde_json::to_string_pretty(&cases).unwrap() + "\n";
    let path = fixture_path();
    if std::env::var_os("UPDATE_FIXTURES").is_some() {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, &text).unwrap();
    }
    let kept = std::fs::read_to_string(&path).unwrap_or_default();
    assert_eq!(
        kept, text,
        "the patch cases are out of date: run `UPDATE_FIXTURES=1 cargo test -p folio-session` and commit the file"
    );
}

#[test]
fn a_duplicate_is_a_new_frame_beside_the_first_and_undoes_away() {
    let mut s = session();
    s.apply(Intent::DuplicateFrame {
        on: folio_model::Surface::Spread(spread("sp_1")),
        frame: frame("fr_pic"),
    })
    .unwrap();
    let frames = &s.book().spreads[1].frames;
    assert_eq!(frames.len(), 4);
    let copy = frames.last().unwrap().base();
    assert_eq!(copy.id.0, "fr_new1");
    assert_eq!((copy.rect.x.0, copy.rect.y.0), (12.0, 12.0));
    assert_eq!(copy.rect.w.0, 100.0);
    s.undo().unwrap();
    assert_eq!(s.book().spreads[1].frames.len(), 3);
}

#[test]
fn a_locked_frame_cannot_be_moved_or_deleted_until_it_is_unlocked() {
    let mut s = session();
    let lock = |locked| Intent::LockFrame {
        on: folio_model::Surface::Spread(spread("sp_1")),
        frame: frame("fr_pic"),
        locked,
    };
    s.apply(lock(true)).unwrap();
    assert_eq!(
        s.apply(move_pic(40.0)).unwrap_err(),
        SessionError::Locked("fr_pic".into())
    );
    assert_eq!(
        s.apply(Intent::DeleteFrame {
            on: folio_model::Surface::Spread(spread("sp_1")),
            frame: frame("fr_pic"),
        })
        .unwrap_err(),
        SessionError::Locked("fr_pic".into())
    );
    assert_eq!(pic_x(&s), 0.0);
    s.apply(lock(false)).unwrap();
    s.apply(move_pic(40.0)).unwrap();
    assert_eq!(pic_x(&s), 40.0);
}

#[test]
fn a_copy_of_a_locked_frame_is_not_locked() {
    let mut s = session();
    s.apply(Intent::LockFrame {
        on: folio_model::Surface::Spread(spread("sp_1")),
        frame: frame("fr_pic"),
        locked: true,
    })
    .unwrap();
    s.apply(Intent::DuplicateFrame {
        on: folio_model::Surface::Spread(spread("sp_1")),
        frame: frame("fr_pic"),
    })
    .unwrap();
    assert!(!s.book().spreads[1].frames.last().unwrap().base().locked);
}

#[test]
fn a_picture_is_set_and_fitted_and_only_on_a_picture_frame() {
    let mut s = session();
    s.apply(Intent::SetImage {
        frame: frame("fr_pic"),
        asset: Some("assets/images/abc.jpg".into()),
        fit: None,
    })
    .unwrap();
    let image = |s: &Session| match &s.book().spreads[1].frames[0] {
        Frame::Image(i) => i.image.clone(),
        _ => unreachable!(),
    };
    assert_eq!(image(&s).asset.as_deref(), Some("assets/images/abc.jpg"));
    // Leaving the asset out keeps the picture and changes the fit alone.
    s.apply(Intent::SetImage {
        frame: frame("fr_pic"),
        asset: None,
        fit: Some(folio_model::Fit::Fit),
    })
    .unwrap();
    assert_eq!(image(&s).asset.as_deref(), Some("assets/images/abc.jpg"));
    assert_eq!(image(&s).fit, folio_model::Fit::Fit);
    s.undo().unwrap();
    assert_eq!(image(&s).fit, folio_model::Fit::Fill);
    assert_eq!(
        s.apply(Intent::SetImage {
            frame: frame("fr_text"),
            asset: Some("x".into()),
            fit: None,
        })
        .unwrap_err(),
        SessionError::NotImage("fr_text".into())
    );
}

fn master_of_pages(s: &Session) -> Vec<Option<String>> {
    s.book()
        .spreads
        .iter()
        .flat_map(|sp| {
            sp.pages
                .iter()
                .map(|p| p.master.as_ref().map(|m| m.0.clone()))
        })
        .collect()
}

#[test]
fn page_numbers_make_a_master_and_give_every_page_after_the_cover_its_master() {
    let mut s = session();
    let patch = s
        .apply(Intent::SetPageNumbers {
            position: folio_model::PageNumbers::Centre,
        })
        .unwrap();
    assert!(patch
        .ops
        .iter()
        .any(|op| matches!(op, PatchOp::SetMasters { masters, .. } if masters.len() == 1)));
    assert_eq!(
        master_of_pages(&s),
        vec![
            None,
            Some("mst_a".into()),
            Some("mst_a".into()),
            Some("mst_a".into())
        ]
    );
    assert_eq!(s.book().master_pages[0].frames.len(), 1);
    // Taking them off, and undoing that, are each one step.
    s.apply(Intent::SetPageNumbers {
        position: folio_model::PageNumbers::None,
    })
    .unwrap();
    assert!(s.book().master_pages.is_empty());
    assert!(master_of_pages(&s).iter().all(Option::is_none));
    s.undo().unwrap();
    assert_eq!(s.book().page_numbers, folio_model::PageNumbers::Centre);
    assert_eq!(s.book().master_pages.len(), 1);
}

#[test]
fn numbers_on_the_outside_use_a_master_for_each_side_of_the_spread() {
    let mut s = session();
    s.apply(Intent::SetPageNumbers {
        position: folio_model::PageNumbers::Outside,
    })
    .unwrap();
    assert_eq!(s.book().master_pages.len(), 2);
    // The opener is a lone right-hand page; the pair after it is left then right.
    assert_eq!(
        master_of_pages(&s),
        vec![
            None,
            Some("mst_a".into()),
            Some("mst_b".into()),
            Some("mst_a".into())
        ]
    );
}

#[test]
fn pages_added_later_are_numbered_too_and_it_all_undoes_together() {
    let mut s = session();
    s.apply(Intent::SetPageNumbers {
        position: folio_model::PageNumbers::Centre,
    })
    .unwrap();
    s.apply(Intent::AddSpread {
        after: spread("sp_2"),
        duplicate: false,
    })
    .unwrap();
    let masters = master_of_pages(&s);
    assert_eq!(masters.len(), 6);
    assert!(masters[1..].iter().all(|m| m.as_deref() == Some("mst_a")));
    s.undo().unwrap();
    assert_eq!(master_of_pages(&s).len(), 4);
}

#[test]
fn a_master_item_is_hidden_on_one_page_and_shown_again() {
    let mut s = session();
    s.apply(Intent::SetPageNumbers {
        position: folio_model::PageNumbers::Centre,
    })
    .unwrap();
    let hide = |hide| Intent::HideMasterItem {
        spread: spread("sp_1"),
        page: folio_model::PageId("pg_1".into()),
        frame: frame("fr_pn_a"),
        hide,
    };
    s.apply(hide(true)).unwrap();
    let hidden = |s: &Session| s.book().spreads[1].master_overrides.clone();
    assert_eq!(hidden(&s).len(), 1);
    assert_eq!(hidden(&s)[0].hide, vec![frame("fr_pn_a")]);
    s.apply(hide(false)).unwrap();
    assert!(hidden(&s).is_empty());
    // A frame that is not on the page's master is refused.
    assert_eq!(
        s.apply(Intent::HideMasterItem {
            spread: spread("sp_1"),
            page: folio_model::PageId("pg_1".into()),
            frame: frame("fr_other"),
            hide: true,
        })
        .unwrap_err(),
        SessionError::NoSuchFrame("fr_other".into())
    );
}

fn rect_at(x: f64, y: f64) -> Rect {
    Rect {
        x: Pt(x),
        y: Pt(y),
        w: Pt(100.0),
        h: Pt(100.0),
    }
}

fn two_moves(x: f64) -> Intent {
    Intent::MoveFrames {
        on: folio_model::Surface::Spread(spread("sp_1")),
        moves: vec![
            folio_model::FrameMove {
                frame: frame("fr_pic"),
                rect: rect_at(x, 0.0),
            },
            folio_model::FrameMove {
                frame: frame("fr_text"),
                rect: rect_at(x, 150.0),
            },
        ],
    }
}

#[test]
fn moving_several_frames_together_is_one_undo_step_even_across_a_drag() {
    let mut s = session();
    let t0 = Instant::now();
    s.apply_at(two_moves(10.0), t0).unwrap();
    s.apply_at(two_moves(20.0), t0 + Duration::from_millis(100))
        .unwrap();
    s.apply_at(two_moves(30.0), t0 + Duration::from_millis(200))
        .unwrap();
    assert_eq!(pic_x(&s), 30.0);
    s.undo().unwrap();
    assert_eq!(
        pic_x(&s),
        0.0,
        "one undo takes the whole drag back, for every frame"
    );
    assert!(s.undo().is_none());
}

#[test]
fn a_locked_frame_is_held_against_every_change_not_only_moving() {
    let mut s = session();
    s.apply(Intent::LockFrame {
        on: folio_model::Surface::Spread(spread("sp_1")),
        frame: frame("fr_text"),
        locked: true,
    })
    .unwrap();
    let locked = || SessionError::Locked("fr_text".into());
    assert_eq!(
        s.apply(Intent::SetText {
            frame: frame("fr_text"),
            text: "x".into()
        })
        .unwrap_err(),
        locked()
    );
    assert_eq!(
        s.apply(Intent::SetStyle {
            frame: frame("fr_text"),
            style: StyleId("title".into())
        })
        .unwrap_err(),
        locked()
    );
    assert_eq!(
        s.apply(Intent::Reorder {
            on: folio_model::Surface::Spread(spread("sp_1")),
            frame: frame("fr_text"),
            to: ReorderTo::Front
        })
        .unwrap_err(),
        locked()
    );
    assert_eq!(
        s.apply(Intent::MoveFrames {
            on: folio_model::Surface::Spread(spread("sp_1")),
            moves: vec![folio_model::FrameMove {
                frame: frame("fr_text"),
                rect: rect_at(1.0, 1.0)
            }]
        })
        .unwrap_err(),
        locked()
    );
}

#[test]
fn a_picture_must_be_one_of_the_books_own_assets() {
    let mut s = session();
    for bad in [
        "/etc/passwd",
        "assets/images/../../x.png",
        "assets/fonts/a.ttf",
        "",
    ] {
        assert_eq!(
            s.apply(Intent::SetImage {
                frame: frame("fr_pic"),
                asset: Some(bad.into()),
                fit: None
            })
            .unwrap_err(),
            SessionError::BadAsset(bad.into()),
            "{bad}"
        );
    }
}

#[test]
fn page_geometry_that_cannot_be_laid_out_is_refused() {
    let mut s = session();
    let geometry =
        |trim: Option<Size>, margins: Option<Margins>, bleed: Option<Pt>| Intent::SetBookGeometry {
            trim,
            margins,
            bleed,
            grid: None,
        };
    let margins = |top, bottom, inside, outside| Margins {
        top: Pt(top),
        bottom: Pt(bottom),
        inside: Pt(inside),
        outside: Pt(outside),
    };
    assert!(matches!(
        s.apply(geometry(None, Some(margins(-1.0, 36.0, 36.0, 36.0)), None)),
        Err(SessionError::BadGeometry(_))
    ));
    assert!(matches!(
        s.apply(geometry(
            None,
            Some(margins(36.0, 36.0, 400.0, 300.0)),
            None
        )),
        Err(SessionError::BadGeometry(_))
    ));
    assert!(matches!(
        s.apply(geometry(None, None, Some(Pt(f64::NAN)))),
        Err(SessionError::BadGeometry(_))
    ));
    assert!(matches!(
        s.apply(geometry(None, None, Some(Pt(500.0)))),
        Err(SessionError::BadGeometry(_))
    ));
    assert!(matches!(
        s.apply(geometry(
            Some(Size {
                w: Pt(0.0),
                h: Pt(100.0)
            }),
            None,
            None
        )),
        Err(SessionError::BadGeometry(_))
    ));
    assert!(matches!(
        s.apply(Intent::SetBookGeometry {
            trim: None,
            margins: None,
            bleed: None,
            grid: Some(Grid {
                columns: 13,
                gutter: Pt(0.0),
                baseline: None
            })
        }),
        Err(SessionError::BadGeometry(_))
    ));
    assert_eq!(s.revision(), 0);
    s.apply(geometry(
        None,
        Some(margins(10.0, 10.0, 10.0, 10.0)),
        Some(Pt(9.0)),
    ))
    .unwrap();
}

#[test]
fn page_numbers_follow_a_change_of_margins_and_forget_what_was_hidden_when_they_are_rebuilt() {
    let mut s = session();
    s.apply(Intent::SetPageNumbers {
        position: folio_model::PageNumbers::Centre,
    })
    .unwrap();
    let number_y = |s: &Session| match &s.book().master_pages[0].frames[0] {
        Frame::Text(t) => t.base.rect.y.0,
        _ => unreachable!(),
    };
    let before = number_y(&s);
    s.apply(Intent::SetBookGeometry {
        trim: None,
        margins: Some(Margins {
            top: Pt(36.0),
            bottom: Pt(90.0),
            inside: Pt(36.0),
            outside: Pt(36.0),
        }),
        bleed: None,
        grid: None,
    })
    .unwrap();
    assert!(
        number_y(&s) < before,
        "a taller bottom margin moves the number up with it"
    );

    s.apply(Intent::HideMasterItem {
        spread: spread("sp_1"),
        page: folio_model::PageId("pg_1".into()),
        frame: frame("fr_pn_a"),
        hide: true,
    })
    .unwrap();
    s.apply(Intent::SetPageNumbers {
        position: folio_model::PageNumbers::None,
    })
    .unwrap();
    assert!(s.book().spreads[1].master_overrides.is_empty());
    s.apply(Intent::SetPageNumbers {
        position: folio_model::PageNumbers::Centre,
    })
    .unwrap();
    assert!(
        s.book().spreads[1].master_overrides.is_empty(),
        "the number is back on every page"
    );
}

#[test]
fn the_same_page_numbers_again_change_nothing_and_a_master_from_elsewhere_is_kept() {
    let mut with_master = book();
    with_master.master_pages.push(
        serde_json::from_value(json!({
            "id": "mst_theirs", "name": "Theirs", "frames": []
        }))
        .unwrap(),
    );
    let mut s = Session::new(with_master);
    s.apply(Intent::SetPageNumbers {
        position: folio_model::PageNumbers::Centre,
    })
    .unwrap();
    let revision = s.revision();
    let patch = s
        .apply(Intent::SetPageNumbers {
            position: folio_model::PageNumbers::Centre,
        })
        .unwrap();
    assert!(patch.ops.is_empty());
    assert_eq!(s.revision(), revision);
    assert!(s.book().master_pages.iter().any(|m| m.id.0 == "mst_theirs"));
    s.apply(Intent::SetPageNumbers {
        position: folio_model::PageNumbers::None,
    })
    .unwrap();
    assert_eq!(s.book().master_pages.len(), 1);
}

fn master<'a>(s: &'a Session, id: &str) -> Option<&'a folio_model::Master> {
    s.book().master_pages.iter().find(|m| m.id.0 == id)
}

fn user_frame() -> Frame {
    serde_json::from_value(json!({
        "type": "text", "id": "fr_mine", "box": { "x": 10.0, "y": 10.0, "w": 100.0, "h": 20.0 },
        "style": "folio", "content": { "inline": [{ "text": "Running head" }] },
        "columns": 1, "baselineLock": false
    }))
    .unwrap()
}

#[test]
fn a_frame_the_person_put_on_a_master_survives_page_numbers_and_new_pages() {
    let mut s = session();
    s.apply(Intent::SetPageNumbers {
        position: folio_model::PageNumbers::Outside,
    })
    .unwrap();
    // The person edits master A through the model, as the master view will.
    let mut edited = s.book().clone();
    edited
        .master_pages
        .iter_mut()
        .find(|m| m.id.0 == "mst_a")
        .unwrap()
        .frames
        .push(user_frame());
    let mut s = Session::new(edited);
    for numbers in [
        folio_model::PageNumbers::Centre,
        folio_model::PageNumbers::Outside,
    ] {
        s.apply(Intent::SetPageNumbers { position: numbers })
            .unwrap();
        let a = master(&s, "mst_a").unwrap();
        assert!(
            a.frames.iter().any(|f| f.base().id.0 == "fr_mine"),
            "the running head stays when the numbers are {numbers:?}"
        );
    }
    s.apply(Intent::AddSpread {
        after: spread("sp_1"),
        duplicate: false,
    })
    .unwrap();
    s.apply(Intent::SetBookGeometry {
        trim: None,
        margins: None,
        bleed: Some(folio_model::Pt(9.0)),
        grid: None,
    })
    .unwrap();
    let a = master(&s, "mst_a").unwrap();
    assert!(a.frames.iter().any(|f| f.base().id.0 == "fr_mine"));
    assert_eq!(a.frames.len(), 2);
}

#[test]
fn a_number_the_person_reworded_keeps_its_words_when_the_numbers_move() {
    let mut s = session();
    s.apply(Intent::SetPageNumbers {
        position: folio_model::PageNumbers::Outside,
    })
    .unwrap();
    let mut edited = s.book().clone();
    if let Some(Frame::Text(t)) = edited
        .master_pages
        .iter_mut()
        .find(|m| m.id.0 == "mst_a")
        .unwrap()
        .frames
        .iter_mut()
        .find(|f| f.base().id.0 == "fr_pn_a")
    {
        t.content = Some(folio_model::InlineContent {
            inline: vec![folio_model::Run {
                text: "Page {page}".into(),
                style: None,
                bold: false,
                italic: false,
            }],
        });
    }
    let mut s = Session::new(edited);
    s.apply(Intent::SetPageNumbers {
        position: folio_model::PageNumbers::Centre,
    })
    .unwrap();
    match &master(&s, "mst_a").unwrap().frames[0] {
        Frame::Text(t) => {
            assert_eq!(t.content.as_ref().unwrap().inline[0].text, "Page {page}");
            assert_eq!(
                t.overrides.as_ref().unwrap().align,
                Some(folio_model::Align::Center)
            );
        }
        _ => unreachable!(),
    }
    assert!(master(&s, "mst_b").is_none(), "B only held the left number");
}

#[test]
fn turning_the_numbers_off_keeps_a_master_that_holds_more_than_numbers() {
    let mut s = session();
    s.apply(Intent::SetPageNumbers {
        position: folio_model::PageNumbers::Outside,
    })
    .unwrap();
    let mut edited = s.book().clone();
    edited
        .master_pages
        .iter_mut()
        .find(|m| m.id.0 == "mst_a")
        .unwrap()
        .guides
        .push(folio_model::Guide {
            axis: Axis::Y,
            pos: folio_model::Pt(36.0),
            locked: false,
        });
    let mut s = Session::new(edited);
    s.apply(Intent::SetPageNumbers {
        position: folio_model::PageNumbers::None,
    })
    .unwrap();
    assert!(master(&s, "mst_a").unwrap().frames.is_empty());
    assert_eq!(master(&s, "mst_a").unwrap().guides.len(), 1);
    assert!(master(&s, "mst_b").is_none(), "B held only a number");
}

fn on_master(id: &str) -> folio_model::Surface {
    folio_model::Surface::Master(folio_model::MasterId(id.into()))
}

fn with_master_a() -> Session {
    let mut s = session();
    s.apply(Intent::SetPageNumbers {
        position: folio_model::PageNumbers::Centre,
    })
    .unwrap();
    s
}

#[test]
fn frames_on_a_master_are_added_moved_and_undone_one_step_each() {
    let mut s = with_master_a();
    let before = s.revision();
    let patch = s
        .apply(Intent::AddFrame {
            on: on_master("mst_a"),
            frame: user_frame(),
        })
        .unwrap();
    assert!(
        matches!(patch.ops.as_slice(), [PatchOp::SetMasters { .. }]),
        "a master edit is patched as the masters, and no spread changes"
    );
    let moved = folio_model::Rect {
        x: folio_model::Pt(40.0),
        y: folio_model::Pt(50.0),
        w: folio_model::Pt(100.0),
        h: folio_model::Pt(20.0),
    };
    s.apply(Intent::MoveFrame {
        on: on_master("mst_a"),
        frame: frame("fr_mine"),
        rect: moved,
    })
    .unwrap();
    let found = master(&s, "mst_a")
        .unwrap()
        .frames
        .iter()
        .find(|f| f.base().id.0 == "fr_mine")
        .unwrap();
    assert_eq!(found.base().rect, moved);
    s.apply(Intent::LockFrame {
        on: on_master("mst_a"),
        frame: frame("fr_mine"),
        locked: true,
    })
    .unwrap();
    assert!(matches!(
        s.apply(Intent::DeleteFrame {
            on: on_master("mst_a"),
            frame: frame("fr_mine"),
        }),
        Err(SessionError::Locked(_))
    ));
    s.undo().unwrap();
    s.undo().unwrap();
    s.undo().unwrap();
    assert_eq!(s.revision(), before + 6, "three changes and three undos");
    assert!(master(&s, "mst_a")
        .unwrap()
        .frames
        .iter()
        .all(|f| f.base().id.0 != "fr_mine"));
}

#[test]
fn frame_only_intents_find_a_frame_on_a_master_and_ids_stay_unique() {
    let mut s = with_master_a();
    s.apply(Intent::AddFrame {
        on: on_master("mst_a"),
        frame: user_frame(),
    })
    .unwrap();
    s.apply(Intent::SetText {
        frame: frame("fr_mine"),
        text: "Running head".into(),
    })
    .unwrap();
    s.apply(Intent::SetStyle {
        frame: frame("fr_mine"),
        style: folio_model::StyleId("folio".into()),
    })
    .unwrap();
    assert!(matches!(
        s.apply(Intent::AddFrame {
            on: Surface::Spread(spread("sp_1")),
            frame: user_frame(),
        }),
        Err(SessionError::DuplicateFrame(_))
    ));
    assert!(matches!(
        s.apply(Intent::AddFrame {
            on: on_master("mst_nope"),
            frame: user_frame(),
        }),
        Err(SessionError::NoSuchMaster(_))
    ));
}

#[test]
fn guides_on_a_master_are_added_moved_locked_and_removed() {
    let mut s = with_master_a();
    let guides = |s: &Session| master(s, "mst_a").unwrap().guides.clone();
    s.apply(Intent::AddGuide {
        on: on_master("mst_a"),
        axis: Axis::Y,
        pos: folio_model::Pt(36.0),
    })
    .unwrap();
    s.apply(Intent::MoveGuide {
        on: on_master("mst_a"),
        index: 0,
        pos: folio_model::Pt(48.0),
    })
    .unwrap();
    assert_eq!(guides(&s)[0].pos.0, 48.0);
    s.apply(Intent::LockGuide {
        on: on_master("mst_a"),
        index: 0,
        locked: true,
    })
    .unwrap();
    assert!(matches!(
        s.apply(Intent::RemoveGuide {
            on: on_master("mst_a"),
            index: 0
        }),
        Err(SessionError::GuideLocked(0))
    ));
    s.apply(Intent::LockGuide {
        on: on_master("mst_a"),
        index: 0,
        locked: false,
    })
    .unwrap();
    s.apply(Intent::RemoveGuide {
        on: on_master("mst_a"),
        index: 0,
    })
    .unwrap();
    assert!(guides(&s).is_empty());
    s.undo().unwrap();
    assert_eq!(guides(&s).len(), 1);
}

#[test]
fn making_the_masters_gives_a_book_without_page_numbers_a_master_for_every_page_after_the_cover() {
    let mut s = session();
    assert!(s.book().master_pages.is_empty());
    s.apply(Intent::EnsureMasters).unwrap();
    assert_eq!(s.book().master_pages.len(), 1);
    assert_eq!(s.book().master_pages[0].id.0, "mst_a");
    for spread in &s.book().spreads {
        for page in &spread.pages {
            assert_eq!(
                page.master.is_some(),
                spread.kind != SpreadKind::Cover,
                "{:?}",
                spread.kind
            );
        }
    }
    // Asking again changes nothing, and one undo takes the masters back.
    let revision = s.revision();
    assert!(s.apply(Intent::EnsureMasters).unwrap().ops.is_empty());
    assert_eq!(s.revision(), revision);
    s.undo().unwrap();
    assert!(s.book().master_pages.is_empty());
}

#[test]
fn making_the_masters_with_the_numbers_outside_makes_a_verso_too() {
    let mut s = session();
    s.apply(Intent::SetPageNumbers {
        position: folio_model::PageNumbers::None,
    })
    .unwrap();
    let mut outside = s.book().clone();
    outside.page_numbers = folio_model::PageNumbers::Outside;
    let mut s = Session::new(outside);
    s.apply(Intent::EnsureMasters).unwrap();
    let ids: Vec<_> = s
        .book()
        .master_pages
        .iter()
        .map(|m| m.id.0.clone())
        .collect();
    assert_eq!(ids, ["mst_a", "mst_b"]);
}

fn add_guide(s: &mut Session, at: &str, axis: Axis, pos: f64) {
    s.apply(Intent::AddGuide {
        on: Surface::Spread(spread(at)),
        axis,
        pos: Pt(pos),
    })
    .unwrap();
}

fn master_guides(s: &Session, id: &str) -> Vec<(Axis, f64)> {
    master(s, id)
        .map(|m| m.guides.iter().map(|g| (g.axis, g.pos.0)).collect())
        .unwrap_or_default()
}

#[test]
fn a_horizontal_guide_put_on_every_page_goes_to_the_masters_its_pages_use_and_leaves_the_spread() {
    let mut s = session();
    add_guide(&mut s, "sp_2", Axis::Y, 60.0);
    let patch = s
        .apply(Intent::PromoteGuide {
            spread: spread("sp_2"),
            index: 0,
        })
        .unwrap();
    assert!(
        patch.ops.iter().any(|op| matches!(
            op,
            PatchOp::SetSpread { spread, .. } if spread.id.0 == "sp_1"
        )),
        "the other spreads, which were given their master, are patched"
    );
    assert!(guides(&s, 2).is_empty());
    assert_eq!(master_guides(&s, "mst_a"), [(Axis::Y, 60.0)]);
    // Both pages of the spread use master A, so it holds the guide once.
    s.undo().unwrap();
    assert_eq!(guides(&s, 2), [(Axis::Y, 60.0, false)]);
    assert!(
        master(&s, "mst_a").is_none(),
        "the masters made for it are undone too"
    );
    assert!(
        s.book().spreads[1].pages[0].master.is_none(),
        "and so are the pages' masters"
    );
}

#[test]
fn only_on_this_spread_takes_a_horizontal_guide_off_every_master_the_spread_uses() {
    let s = session();
    let mut outside = s.book().clone();
    outside.page_numbers = folio_model::PageNumbers::Outside;
    let mut s = Session::new(outside);
    add_guide(&mut s, "sp_2", Axis::Y, 60.0);
    s.apply(Intent::PromoteGuide {
        spread: spread("sp_2"),
        index: 0,
    })
    .unwrap();
    assert_eq!(master_guides(&s, "mst_a"), [(Axis::Y, 60.0)]);
    assert_eq!(master_guides(&s, "mst_b"), [(Axis::Y, 60.0)]);
    s.apply(Intent::DemoteGuide {
        master: folio_model::MasterId("mst_a".into()),
        index: 0,
        spread: spread("sp_2"),
    })
    .unwrap();
    assert!(master_guides(&s, "mst_a").is_empty());
    assert!(master_guides(&s, "mst_b").is_empty());
    assert_eq!(
        guides(&s, 2),
        [(Axis::Y, 60.0, false)],
        "once, on the spread"
    );
}

#[test]
fn a_vertical_guide_goes_to_the_master_of_the_page_it_is_over_and_is_mirrored_on_the_facing_one() {
    let s = session();
    let mut outside = s.book().clone();
    outside.page_numbers = folio_model::PageNumbers::Outside;
    let mut s = Session::new(outside);
    let w = s.book().trim.w.0;
    add_guide(&mut s, "sp_2", Axis::X, w + 30.0);
    s.apply(Intent::PromoteGuide {
        spread: spread("sp_2"),
        index: 0,
    })
    .unwrap();
    // The second page of a spread is a right-hand page, which uses master A.
    assert_eq!(master_guides(&s, "mst_a"), [(Axis::X, 30.0)]);
    assert_eq!(master_guides(&s, "mst_b"), [(Axis::X, w - 30.0)]);
}

#[test]
fn a_guide_in_the_gutter_off_the_pages_on_the_cover_or_locked_is_not_put_on_every_page() {
    let mut s = session();
    let w = s.book().trim.w.0;
    add_guide(&mut s, "sp_2", Axis::X, w);
    add_guide(&mut s, "sp_2", Axis::X, 2.0 * w + 5.0);
    add_guide(&mut s, "sp_cover", Axis::Y, 10.0);
    add_guide(&mut s, "sp_2", Axis::Y, 12.0);
    for (spread_id, index) in [("sp_2", 0), ("sp_2", 1), ("sp_cover", 0)] {
        assert!(
            matches!(
                s.apply(Intent::PromoteGuide {
                    spread: spread(spread_id),
                    index
                }),
                Err(SessionError::CannotPromote)
            ),
            "{spread_id} {index}"
        );
    }
    s.apply(Intent::LockGuide {
        on: Surface::Spread(spread("sp_2")),
        index: 2,
        locked: true,
    })
    .unwrap();
    assert!(matches!(
        s.apply(Intent::PromoteGuide {
            spread: spread("sp_2"),
            index: 2
        }),
        Err(SessionError::GuideLocked(2))
    ));
}

#[test]
fn a_guide_put_back_on_one_spread_leaves_the_masters_and_lands_where_the_page_is() {
    let s = session();
    let mut outside = s.book().clone();
    outside.page_numbers = folio_model::PageNumbers::Outside;
    let mut s = Session::new(outside);
    let w = s.book().trim.w.0;
    add_guide(&mut s, "sp_2", Axis::X, w + 30.0);
    s.apply(Intent::PromoteGuide {
        spread: spread("sp_2"),
        index: 0,
    })
    .unwrap();
    s.apply(Intent::DemoteGuide {
        master: folio_model::MasterId("mst_a".into()),
        index: 0,
        spread: spread("sp_2"),
    })
    .unwrap();
    assert!(master_guides(&s, "mst_a").is_empty());
    assert!(master_guides(&s, "mst_b").is_empty(), "the mirror goes too");
    assert_eq!(guides(&s, 2), [(Axis::X, w + 30.0, false)]);
    assert!(matches!(
        s.apply(Intent::DemoteGuide {
            master: folio_model::MasterId("mst_a".into()),
            index: 0,
            spread: spread("sp_2"),
        }),
        Err(SessionError::NoSuchGuide(0, _))
    ));
}

#[test]
fn a_copy_of_a_flowing_frame_is_not_another_slot_in_the_flow() {
    let mut s = session();
    s.apply(Intent::DuplicateFrame {
        on: folio_model::Surface::Spread(spread("sp_1")),
        frame: frame("fr_flowed"),
    })
    .unwrap();
    match s.book().spreads[1].frames.last().unwrap() {
        Frame::Text(t) => assert!(t.flow.is_none()),
        _ => unreachable!(),
    }
}

fn guides(s: &Session, at: usize) -> Vec<(Axis, f64, bool)> {
    s.book().spreads[at]
        .guides
        .iter()
        .map(|g| (g.axis, g.pos.0, g.locked))
        .collect()
}

#[test]
fn guides_are_added_moved_locked_and_removed_and_each_is_one_undo_step() {
    let mut s = session();
    let add = |axis, at: f64| Intent::AddGuide {
        on: folio_model::Surface::Spread(spread("sp_1")),
        axis,
        pos: Pt(at),
    };
    s.apply(add(Axis::X, 100.0)).unwrap();
    s.apply(add(Axis::Y, 50.0)).unwrap();
    assert_eq!(
        guides(&s, 1),
        vec![(Axis::X, 100.0, false), (Axis::Y, 50.0, false)]
    );

    s.apply(Intent::MoveGuide {
        on: folio_model::Surface::Spread(spread("sp_1")),
        index: 0,
        pos: Pt(120.0),
    })
    .unwrap();
    assert_eq!(guides(&s, 1)[0], (Axis::X, 120.0, false));
    s.undo().unwrap();
    assert_eq!(guides(&s, 1)[0], (Axis::X, 100.0, false));

    s.apply(Intent::RemoveGuide {
        on: folio_model::Surface::Spread(spread("sp_1")),
        index: 0,
    })
    .unwrap();
    assert_eq!(guides(&s, 1), vec![(Axis::Y, 50.0, false)]);
    s.undo().unwrap();
    assert_eq!(guides(&s, 1).len(), 2);
}

#[test]
fn a_locked_guide_is_not_moved_or_removed_or_cleared_until_it_is_freed() {
    let mut s = session();
    s.apply(Intent::AddGuide {
        on: folio_model::Surface::Spread(spread("sp_1")),
        axis: Axis::X,
        pos: Pt(10.0),
    })
    .unwrap();
    let lock = |locked| Intent::LockGuide {
        on: folio_model::Surface::Spread(spread("sp_1")),
        index: 0,
        locked,
    };
    s.apply(lock(true)).unwrap();
    assert_eq!(
        s.apply(Intent::MoveGuide {
            on: folio_model::Surface::Spread(spread("sp_1")),
            index: 0,
            pos: Pt(30.0),
        })
        .unwrap_err(),
        SessionError::GuideLocked(0)
    );
    assert_eq!(
        s.apply(Intent::RemoveGuide {
            on: folio_model::Surface::Spread(spread("sp_1")),
            index: 0,
        })
        .unwrap_err(),
        SessionError::GuideLocked(0)
    );
    s.apply(Intent::ClearGuides { spread: None }).unwrap();
    assert_eq!(guides(&s, 1), vec![(Axis::X, 10.0, true)]);
    s.apply(lock(false)).unwrap();
    s.apply(Intent::ClearGuides {
        spread: Some(spread("sp_1")),
    })
    .unwrap();
    assert!(guides(&s, 1).is_empty());
}

#[test]
fn clearing_guides_clears_one_spread_or_all_and_a_clear_of_nothing_changes_nothing() {
    let mut s = session();
    for sp in ["sp_2", "sp_1"] {
        s.apply(Intent::AddGuide {
            on: folio_model::Surface::Spread(spread(sp)),
            axis: Axis::Y,
            pos: Pt(20.0),
        })
        .unwrap();
    }
    s.apply(Intent::ClearGuides {
        spread: Some(spread("sp_1")),
    })
    .unwrap();
    assert_eq!(guides(&s, 2).len(), 1);
    assert!(guides(&s, 1).is_empty());
    s.apply(Intent::ClearGuides { spread: None }).unwrap();
    assert!(guides(&s, 2).is_empty());

    let revision = s.revision();
    let patch = s.apply(Intent::ClearGuides { spread: None }).unwrap();
    assert!(patch.ops.is_empty());
    assert_eq!(s.revision(), revision);
}

#[test]
fn a_guide_must_be_on_a_spread_that_exists_at_a_position_that_makes_sense() {
    let mut s = session();
    let bad = |at: f64| Intent::AddGuide {
        on: folio_model::Surface::Spread(spread("sp_1")),
        axis: Axis::X,
        pos: Pt(at),
    };
    assert_eq!(s.apply(bad(f64::NAN)).unwrap_err(), SessionError::BadGuide);
    assert_eq!(s.apply(bad(1.0e9)).unwrap_err(), SessionError::BadGuide);
    assert_eq!(
        s.apply(Intent::AddGuide {
            on: folio_model::Surface::Spread(spread("sp_nope")),
            axis: Axis::X,
            pos: Pt(1.0),
        })
        .unwrap_err(),
        SessionError::NoSuchSpread("sp_nope".into())
    );
    assert_eq!(
        s.apply(Intent::MoveGuide {
            on: folio_model::Surface::Spread(spread("sp_1")),
            index: 3,
            pos: Pt(1.0),
        })
        .unwrap_err(),
        SessionError::NoSuchGuide(3, "sp_1".into())
    );
    assert_eq!(s.revision(), 0);
}

/// A session whose master `id` has the person's frame on it, after the numbers were `numbers`.
fn with_frame_on(numbers: folio_model::PageNumbers, id: &str) -> Session {
    let mut s = session();
    s.apply(Intent::SetPageNumbers { position: numbers })
        .unwrap();
    let mut edited = s.book().clone();
    edited
        .master_pages
        .iter_mut()
        .find(|m| m.id.0 == id)
        .unwrap()
        .frames
        .push(user_frame());
    Session::new(edited)
}

#[test]
fn a_new_spread_uses_the_master_the_person_kept_when_the_numbers_are_off() {
    let mut s = with_frame_on(folio_model::PageNumbers::Centre, "mst_a");
    s.apply(Intent::SetPageNumbers {
        position: folio_model::PageNumbers::None,
    })
    .unwrap();
    s.apply(Intent::AddSpread {
        after: spread("sp_1"),
        duplicate: false,
    })
    .unwrap();
    let masters = master_of_pages(&s);
    assert_eq!(masters.len(), 6);
    assert!(
        masters[1..].iter().all(|m| m.as_deref() == Some("mst_a")),
        "{masters:?}"
    );
}

#[test]
fn centring_the_numbers_keeps_a_master_b_the_person_filled_and_its_pages() {
    let mut s = with_frame_on(folio_model::PageNumbers::Outside, "mst_b");
    let before = master_of_pages(&s);
    assert_eq!(
        before,
        vec![
            None,
            Some("mst_a".into()),
            Some("mst_b".into()),
            Some("mst_a".into())
        ]
    );
    s.apply(Intent::SetPageNumbers {
        position: folio_model::PageNumbers::Centre,
    })
    .unwrap();
    assert_eq!(
        master_of_pages(&s),
        before,
        "the left page keeps its master"
    );
    let b = master(&s, "mst_b").unwrap();
    assert!(b.frames.iter().any(|f| f.base().id.0 == "fr_mine"));
    assert!(
        b.frames.iter().any(|f| f.base().id.0 == "fr_pn_b"),
        "so the left page still has a number"
    );
    s.apply(Intent::SetPageNumbers {
        position: folio_model::PageNumbers::None,
    })
    .unwrap();
    let b = master(&s, "mst_b").unwrap();
    assert_eq!(b.frames.len(), 1, "only the number goes");
    // A held only a number, so it went; the left page keeps B.
    assert_eq!(
        master_of_pages(&s),
        vec![None, None, Some("mst_b".into()), None]
    );
}

#[test]
fn deleting_a_master_frame_that_a_page_hid_forgets_the_hiding_and_undoes_with_it() {
    let mut s = with_master_a();
    s.apply(Intent::AddFrame {
        on: on_master("mst_a"),
        frame: user_frame(),
    })
    .unwrap();
    let page = s.book().spreads[1].pages[0].id.clone();
    s.apply(Intent::HideMasterItem {
        spread: spread("sp_1"),
        page,
        frame: frame("fr_mine"),
        hide: true,
    })
    .unwrap();
    assert_eq!(s.book().spreads[1].master_overrides.len(), 1);
    let patch = s
        .apply(Intent::DeleteFrame {
            on: on_master("mst_a"),
            frame: frame("fr_mine"),
        })
        .unwrap();
    assert!(s.book().spreads[1].master_overrides.is_empty());
    assert!(
        patch
            .ops
            .iter()
            .any(|op| matches!(op, PatchOp::SetSpread { .. })),
        "the spread that forgot it is patched"
    );
    s.undo().unwrap();
    assert_eq!(s.book().spreads[1].master_overrides.len(), 1);
    assert!(master(&s, "mst_a")
        .unwrap()
        .frames
        .iter()
        .any(|f| f.base().id.0 == "fr_mine"));
}

#[test]
fn a_page_number_on_a_master_cannot_be_deleted_by_hand() {
    let mut s = with_master_a();
    assert!(matches!(
        s.apply(Intent::DeleteFrame {
            on: on_master("mst_a"),
            frame: frame("fr_pn_a"),
        }),
        Err(SessionError::PageNumber(_))
    ));
}

#[test]
fn a_master_frame_is_duplicated_reordered_and_locked_on_its_master() {
    let mut s = with_master_a();
    s.apply(Intent::AddFrame {
        on: on_master("mst_a"),
        frame: user_frame(),
    })
    .unwrap();
    s.apply(Intent::DuplicateFrame {
        on: on_master("mst_a"),
        frame: frame("fr_mine"),
    })
    .unwrap();
    assert_eq!(master(&s, "mst_a").unwrap().frames.len(), 3);
    s.apply(Intent::Reorder {
        on: on_master("mst_a"),
        frame: frame("fr_mine"),
        to: folio_model::ReorderTo::Back,
    })
    .unwrap();
    assert_eq!(
        master(&s, "mst_a").unwrap().frames[0].base().id.0,
        "fr_mine"
    );
    s.apply(Intent::LockFrame {
        on: on_master("mst_a"),
        frame: frame("fr_mine"),
        locked: true,
    })
    .unwrap();
    assert!(master(&s, "mst_a").unwrap().frames[0].base().locked);
}

fn session_with_shape() -> Session {
    let mut s = session();
    s.apply(Intent::AddFrame {
        on: folio_model::Surface::Spread(spread("sp_2")),
        frame: serde_json::from_value(json!({
            "type": "shape", "id": "fr_shape", "box": { "x": 10, "y": 10, "w": 100, "h": 60 },
            "shape": "rect", "fill": "#c9c4b8", "style": null
        }))
        .unwrap(),
    })
    .unwrap();
    s
}

fn shape_of(s: &Session) -> folio_model::ShapeFrame {
    match &s.book().spreads[2].frames[0] {
        Frame::Shape(shape) => shape.clone(),
        _ => unreachable!(),
    }
}

/// A `SetShape` for the shape on `sp_2`, changing only what is `Some`.
fn shape_set(
    fill: Option<folio_model::Paint>,
    stroke_width: Option<f64>,
    kind: Option<folio_model::Shape>,
) -> Intent {
    Intent::SetShape {
        frame: frame("fr_shape"),
        kind,
        fill,
        stroke: None,
        stroke_width: stroke_width.map(folio_model::Pt),
        radius: None,
        flip: None,
    }
}

#[test]
fn a_shape_changes_its_kind_fill_and_outline_and_leaves_the_rest_alone() {
    let mut s = session_with_shape();
    let colour = |c: &str| folio_model::Paint::Colour {
        colour: folio_model::Colour(c.into()),
    };
    s.apply(Intent::SetShape {
        frame: frame("fr_shape"),
        kind: Some(folio_model::Shape::Round),
        fill: Some(colour("#FFF")),
        stroke: Some(colour("#14110f")),
        stroke_width: Some(folio_model::Pt(2.5)),
        radius: Some(folio_model::Pt(9.0)),
        flip: None,
    })
    .unwrap();
    let shape = shape_of(&s);
    assert_eq!(shape.shape, folio_model::Shape::Round);
    assert_eq!(shape.fill.as_ref().unwrap().0, "#ffffff");
    assert_eq!(shape.stroke.as_ref().unwrap().0, "#14110f");
    assert_eq!(shape.stroke_width, Some(folio_model::Pt(2.5)));
    assert_eq!(shape.radius, Some(folio_model::Pt(9.0)));
    assert!(!shape.flip);
    // Only the fill is cleared; the outline stays.
    s.apply(shape_set(Some(folio_model::Paint::None), None, None))
        .unwrap();
    let shape = shape_of(&s);
    assert!(shape.fill.is_none() && shape.stroke.is_some());
    assert_eq!(shape.shape, folio_model::Shape::Round);
}

#[test]
fn trying_a_shape_out_is_one_undo_step() {
    let mut s = session_with_shape();
    for width in [1.0, 2.0, 3.0] {
        s.apply(shape_set(None, Some(width), None)).unwrap();
    }
    s.undo().unwrap();
    assert_eq!(shape_of(&s).stroke_width, None);
}

#[test]
fn a_shape_setting_is_refused_when_it_makes_no_sense_or_the_frame_is_not_free() {
    let mut s = session_with_shape();
    let red = folio_model::Paint::Colour {
        colour: folio_model::Colour("red".into()),
    };
    assert!(matches!(
        s.apply(shape_set(Some(red), None, None)).unwrap_err(),
        SessionError::BadShape(_)
    ));
    for width in [-1.0, f64::NAN, 500.0] {
        assert!(matches!(
            s.apply(shape_set(None, Some(width), None)).unwrap_err(),
            SessionError::BadShape(_)
        ));
    }
    let mut on_text = shape_set(None, None, None);
    if let Intent::SetShape { frame: f, .. } = &mut on_text {
        *f = frame("fr_text");
    }
    assert_eq!(
        s.apply(on_text).unwrap_err(),
        SessionError::NotShape("fr_text".into())
    );
    s.apply(Intent::LockFrame {
        on: folio_model::Surface::Spread(spread("sp_2")),
        frame: frame("fr_shape"),
        locked: true,
    })
    .unwrap();
    assert_eq!(
        s.apply(shape_set(None, None, None)).unwrap_err(),
        SessionError::Locked("fr_shape".into())
    );
}

#[test]
fn words_go_into_a_shape_and_emptying_them_takes_them_out_but_a_line_has_no_inside() {
    let mut s = session_with_shape();
    s.apply(Intent::SetText {
        frame: frame("fr_shape"),
        text: "Hello".into(),
    })
    .unwrap();
    assert_eq!(
        shape_of(&s).content.unwrap().inline[0].text,
        "Hello",
        "the words are on the shape"
    );
    s.apply(Intent::OverrideStyle {
        frame: frame("fr_shape"),
        overrides: folio_model::StyleOverrides {
            size: Some(folio_model::Pt(20.0)),
            align: None,
            bold: Some(true),
            italic: None,
        },
    })
    .unwrap();
    assert_eq!(shape_of(&s).overrides.unwrap().bold, Some(true));
    s.apply(Intent::SetText {
        frame: frame("fr_shape"),
        text: String::new(),
    })
    .unwrap();
    assert!(shape_of(&s).content.is_none());

    s.apply(Intent::SetShape {
        frame: frame("fr_shape"),
        kind: Some(folio_model::Shape::Line),
        fill: None,
        stroke: None,
        stroke_width: None,
        radius: None,
        flip: None,
    })
    .unwrap();
    assert_eq!(
        s.apply(Intent::SetText {
            frame: frame("fr_shape"),
            text: "no".into()
        })
        .unwrap_err(),
        SessionError::NotText("fr_shape".into())
    );
}

#[test]
fn changing_the_kind_or_clearing_a_colour_is_its_own_undo_step() {
    let mut s = session_with_shape();
    s.apply(shape_set(None, Some(1.0), None)).unwrap();
    s.apply(shape_set(None, Some(2.0), None)).unwrap();
    s.apply(shape_set(None, None, Some(folio_model::Shape::Ellipse)))
        .unwrap();
    s.apply(shape_set(Some(folio_model::Paint::None), None, None))
        .unwrap();
    s.undo().unwrap();
    assert!(
        shape_of(&s).fill.is_some(),
        "clearing the fill is undone alone"
    );
    s.undo().unwrap();
    assert_eq!(
        shape_of(&s).shape,
        folio_model::Shape::Rect,
        "the kind alone"
    );
    s.undo().unwrap();
    assert_eq!(shape_of(&s).stroke_width, None, "the two widths together");
}

#[test]
fn turning_a_shape_into_a_line_takes_its_words_with_it_and_undo_brings_them_back() {
    let mut s = session_with_shape();
    s.apply(Intent::SetText {
        frame: frame("fr_shape"),
        text: "Hello".into(),
    })
    .unwrap();
    s.apply(shape_set(None, None, Some(folio_model::Shape::Line)))
        .unwrap();
    assert!(shape_of(&s).content.is_none());
    s.undo().unwrap();
    assert!(shape_of(&s).content.is_some());
}
