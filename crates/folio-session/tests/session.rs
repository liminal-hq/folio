// Drives a session the way the UI will: intents in, patches out, undo and redo
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::time::{Duration, Instant};

use folio_model::{
    Axis, Book, Frame, FrameId, Grid, Intent, Margins, Pt, Rect, ReorderTo, Size, SpreadId,
    SpreadKind, StyleId, StyleOverrides,
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
        spread: spread("sp_1"),
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
        spread: spread("sp_1"),
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
            spread: spread("sp_1"),
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
        spread: spread("sp_1"),
        frame: frame("fr_pic"),
        locked: true,
    })
    .unwrap();
    s.apply(Intent::DuplicateFrame {
        spread: spread("sp_1"),
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
        spread: spread("sp_1"),
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
        spread: spread("sp_1"),
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
            spread: spread("sp_1"),
            frame: frame("fr_text"),
            to: ReorderTo::Front
        })
        .unwrap_err(),
        locked()
    );
    assert_eq!(
        s.apply(Intent::MoveFrames {
            spread: spread("sp_1"),
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
fn a_copy_of_a_flowing_frame_is_not_another_slot_in_the_flow() {
    let mut s = session();
    s.apply(Intent::DuplicateFrame {
        spread: spread("sp_1"),
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
        spread: spread("sp_1"),
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
        spread: spread("sp_1"),
        index: 0,
        pos: Pt(120.0),
    })
    .unwrap();
    assert_eq!(guides(&s, 1)[0], (Axis::X, 120.0, false));
    s.undo().unwrap();
    assert_eq!(guides(&s, 1)[0], (Axis::X, 100.0, false));

    s.apply(Intent::RemoveGuide {
        spread: spread("sp_1"),
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
        spread: spread("sp_1"),
        axis: Axis::X,
        pos: Pt(10.0),
    })
    .unwrap();
    let lock = |locked| Intent::LockGuide {
        spread: spread("sp_1"),
        index: 0,
        locked,
    };
    s.apply(lock(true)).unwrap();
    assert_eq!(
        s.apply(Intent::MoveGuide {
            spread: spread("sp_1"),
            index: 0,
            pos: Pt(30.0),
        })
        .unwrap_err(),
        SessionError::GuideLocked(0)
    );
    assert_eq!(
        s.apply(Intent::RemoveGuide {
            spread: spread("sp_1"),
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
            spread: spread(sp),
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
        spread: spread("sp_1"),
        axis: Axis::X,
        pos: Pt(at),
    };
    assert_eq!(s.apply(bad(f64::NAN)).unwrap_err(), SessionError::BadGuide);
    assert_eq!(s.apply(bad(1.0e9)).unwrap_err(), SessionError::BadGuide);
    assert_eq!(
        s.apply(Intent::AddGuide {
            spread: spread("sp_nope"),
            axis: Axis::X,
            pos: Pt(1.0),
        })
        .unwrap_err(),
        SessionError::NoSuchSpread("sp_nope".into())
    );
    assert_eq!(
        s.apply(Intent::MoveGuide {
            spread: spread("sp_1"),
            index: 3,
            pos: Pt(1.0),
        })
        .unwrap_err(),
        SessionError::NoSuchGuide(3, "sp_1".into())
    );
    assert_eq!(s.revision(), 0);
}
