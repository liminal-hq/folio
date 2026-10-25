// Tests for the page scene: every page of every sample book, drawn bottom to top
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use folio_layout::scene::{shape_paths, SceneKind, SpreadScene};
use folio_layout::{FontBook, Layouter, StyleSheet};
use folio_model::{Book, BookKind, Frame, FrameId, Pt, Shape, SpreadKind};
use folio_presets::{new_book, NewBookSpec};

const KINDS: [BookKind; 6] = [
    BookKind::Picture,
    BookKind::Novel,
    BookKind::Textbook,
    BookKind::Illustrated,
    BookKind::Poetry,
    BookKind::Memoir,
];

fn sample(kind: BookKind) -> (Book, StyleSheet) {
    let made = new_book(&NewBookSpec {
        kind,
        ..NewBookSpec::default()
    })
    .unwrap();
    let styles = StyleSheet::from_styles(&made.book.styles);
    (made.book, styles)
}

fn scenes(book: &Book, styles: &StyleSheet) -> Vec<SpreadScene> {
    let fonts = FontBook::bundled();
    let layouter = Layouter::new(book, styles, &fonts);
    book.spreads
        .iter()
        .map(|s| layouter.scene_spread(&s.id).unwrap().unwrap())
        .collect()
}

#[test]
fn every_spread_of_every_kind_has_a_scene_with_a_page_for_each_page() {
    for kind in KINDS {
        let (book, styles) = sample(kind);
        let scenes = scenes(&book, &styles);
        assert_eq!(scenes.len(), book.spreads.len(), "{kind:?}");
        for (scene, spread) in scenes.iter().zip(&book.spreads) {
            assert_eq!(scene.pages.len(), spread.pages.len(), "{kind:?}");
            for page in &scene.pages {
                // The cover is the printed sheet: the back, the spine and the front.
                if spread.kind == folio_model::SpreadKind::Cover {
                    assert_eq!(page.width, folio_model::cover_width(&book).0);
                } else {
                    assert_eq!(page.width, book.trim.w.0);
                }
                assert_eq!(page.height, book.trim.h.0);
            }
        }
    }
}

#[test]
fn text_comes_with_the_glyphs_and_the_font_it_was_shaped_with() {
    for kind in KINDS {
        let (book, styles) = sample(kind);
        let mut runs = 0;
        for scene in scenes(&book, &styles) {
            for page in &scene.pages {
                for item in &page.items {
                    let SceneKind::Text { lines, .. } = &item.kind else {
                        continue;
                    };
                    for run in lines.iter().flat_map(|l| &l.runs) {
                        if run.text.trim().is_empty() {
                            continue;
                        }
                        runs += 1;
                        assert!(!run.face_key.is_empty(), "{kind:?} {:?}", run.text);
                        assert!(!run.glyphs.is_empty(), "{kind:?} {:?}", run.text);
                        let advance: f64 = run.glyphs.iter().map(|g| g.advance).sum();
                        assert!(
                            (advance - run.width).abs() < 0.5 + run.text.len() as f64 * 0.1,
                            "{kind:?} {:?}: glyphs {advance} vs width {}",
                            run.text,
                            run.width
                        );
                    }
                }
            }
        }
        assert!(runs > 0, "{kind:?} has words to draw");
    }
}

#[test]
fn a_picture_book_has_its_pictures_in_the_scene_and_the_cover_its_colour_block() {
    let (book, styles) = sample(BookKind::Picture);
    let all = scenes(&book, &styles);
    let pictures = all
        .iter()
        .flat_map(|s| &s.pages)
        .flat_map(|p| &p.items)
        .filter(|i| matches!(i.kind, SceneKind::Picture { .. }))
        .count();
    assert!(pictures > 0, "a picture book has pictures");
    let cover = &all[0].pages[0];
    assert!(
        cover.items.iter().any(|i| matches!(
            i.kind,
            SceneKind::Shape { .. } | SceneKind::Background { .. }
        )),
        "the cover has its colour"
    );
}

#[test]
fn a_frame_that_crosses_the_gutter_is_on_both_pages_and_cut_to_each() {
    use folio_model::{Frame, FrameBase, FrameId, Pt, Rect, ShapeFrame};
    let (mut book, styles) = sample(BookKind::Picture);
    let w = book.trim.w.0;
    let index = book
        .spreads
        .iter()
        .position(|s| s.pages.len() == 2)
        .expect("a two-page spread");
    book.spreads[index].frames.push(Frame::Shape(ShapeFrame {
        base: FrameBase {
            id: FrameId("across".into()),
            rect: Rect {
                x: Pt(w - 50.0),
                y: Pt(100.0),
                w: Pt(100.0),
                h: Pt(100.0),
            },
            rotation: 0.0,
            locked: false,
            from_master: None,
            description: None,
        },
        shape: Shape::Rect,
        fill: None,
        style: None,
        stroke: None,
        stroke_width: None,
        radius: None,
        flip: false,
        content: None,
        overrides: None,
    }));
    let scene = scenes(&book, &styles).remove(index);
    let on = |page: usize| {
        scene.pages[page]
            .items
            .iter()
            .find(|i| i.frame.0 == "across")
            .cloned()
    };
    let left = on(0).expect("on the left page");
    let right = on(1).expect("on the right page");
    assert!(left.clip.is_some() && right.clip.is_some());
    let SceneKind::Shape { rect: l, .. } = left.kind else {
        panic!("a shape")
    };
    let SceneKind::Shape { rect: r, .. } = right.kind else {
        panic!("a shape")
    };
    // The same box, seen from each page: where it starts moves by one page's width.
    assert_eq!(l.x - r.x, w);
    assert_eq!(l.x, w - 50.0);
}

#[test]
fn the_bubble_outlines_are_the_ones_the_page_draws() {
    let text = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../apps/folio/src/canvas/fixtures/shapePaths.json"),
    )
    .unwrap();
    let cases: serde_json::Value = serde_json::from_str(&text).unwrap();
    for (name, shape) in [
        ("speech", Shape::Speech),
        ("thought", Shape::Thought),
        ("shout", Shape::Shout),
    ] {
        let expected: Vec<String> = cases[name]
            .as_array()
            .unwrap()
            .iter()
            .map(|p| p.as_str().unwrap().to_string())
            .collect();
        assert_eq!(shape_paths(shape), expected, "{name}");
    }
    assert!(shape_paths(Shape::Rect).is_empty());
}

#[test]
fn a_frame_on_an_opener_is_placed_from_the_pages_own_left_edge() {
    let (mut book, styles) = sample(BookKind::Picture);
    let w = book.trim.w.0;
    // The cover's title, copied onto the opener (a single page, with nothing on it to start).
    let title = book.spreads[0]
        .frames
        .iter()
        .find(|f| matches!(f, Frame::Text(_)))
        .expect("the cover has a title")
        .clone();
    // The page after the cover, made a single right-hand page with nothing on it.
    let opener = 1;
    book.spreads[opener].kind = SpreadKind::Opener;
    book.spreads[opener].pages.truncate(1);
    let place = |book: &mut Book, x: f64| {
        let mut frame = title.clone();
        frame.base_mut().id = FrameId("on_opener".into());
        frame.base_mut().rect.x = Pt(x);
        book.spreads[opener].frames = vec![frame];
    };
    let words_on_page = |book: &Book| {
        scenes(book, &styles)[opener].pages[0]
            .items
            .iter()
            .any(|i| matches!(&i.kind, SceneKind::Text { lines, .. } if !lines.is_empty()))
    };

    // Counted from the page's own left edge, as the canvas, the masters and the guides count.
    place(&mut book, 36.0);
    assert!(words_on_page(&book), "a frame at 36 is on the page");
    // One page further on is off it: there is no left page, and nothing is drawn there.
    place(&mut book, w + 36.0);
    assert!(!words_on_page(&book), "a frame a page-width on is not");
}
