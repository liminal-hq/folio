// Reads the `book.json` shown in spec/03-project-format.md and checks the model keeps it
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use folio_model::{Book, BookKind, Frame, LayoutMode, Pt, SpreadKind};
use serde_json::{json, Value};

fn sample() -> Value {
    json!({
        "schema": 1,
        "id": "bk_01J9",
        "title": "The Otter Who Couldn't Sleep",
        "authors": [{ "name": "Mom", "role": "author" }],
        "language": "en-CA",
        "kind": "picture",
        "layout": "fixed",
        "trim": { "w": 612, "h": 612 },
        "bleed": 9,
        "margins": { "top": 36, "bottom": 36, "inside": 36, "outside": 36 },
        "grid": { "columns": 6, "gutter": 12, "baseline": null },
        "spreads": [{
            "id": "sp_cover",
            "kind": "cover",
            "pages": [{ "id": "pg_c", "master": null }],
            "frames": [
                {
                    "id": "fr_c1",
                    "type": "image",
                    "box": { "x": -9, "y": -9, "w": 630, "h": 630 },
                    "image": { "asset": "assets/images/9f2c.jpg", "fit": "fill", "focus": [0.5, 0.4], "crop": null }
                },
                {
                    "id": "fr_t1",
                    "type": "text",
                    "box": { "x": 54, "y": 61.2, "w": 504, "h": 144 },
                    "style": "display",
                    "content": { "inline": [{ "text": "The Otter Who Couldn't Sleep" }] }
                }
            ]
        }],
        "flows": [{ "id": "fl_main", "source": "text/flow-main.md", "frames": ["fr_b1", "fr_b2"] }],
        "metadata": { "isbn": { "paperback": null, "hardcover": null, "ebook": null }, "description": "" }
    })
}

#[test]
fn the_spec_example_loads_into_the_model() {
    let book: Book = serde_json::from_value(sample()).expect("the example is a book");
    assert_eq!(book.title, "The Otter Who Couldn't Sleep");
    assert_eq!(book.kind, BookKind::Picture);
    assert_eq!(book.layout, LayoutMode::Fixed);
    assert_eq!(book.trim.w, Pt(612.0));
    assert_eq!(book.bleed, Pt(9.0));
    assert_eq!(book.spreads[0].kind, SpreadKind::Cover);
    assert!(matches!(book.spreads[0].frames[0], Frame::Image(_)));
    assert!(matches!(book.spreads[0].frames[1], Frame::Text(_)));
}

#[test]
fn a_frame_is_told_apart_by_its_type_and_keeps_its_box() {
    let book: Book = serde_json::from_value(sample()).unwrap();
    let image = book.spreads[0].frames[0].base();
    assert_eq!(image.id.0, "fr_c1");
    assert_eq!(image.rect.x, Pt(-9.0));
    let written = serde_json::to_value(&book.spreads[0].frames[0]).unwrap();
    assert_eq!(written["type"], "image");
    assert_eq!(written["box"]["w"], 630.0);
}

#[test]
fn defaults_fill_what_a_file_leaves_out_and_the_result_is_stable() {
    let book: Book = serde_json::from_value(sample()).unwrap();
    // A text frame with no `columns` has one; a flow with no `autoAddPages` adds pages.
    let Frame::Text(text) = &book.spreads[0].frames[1] else {
        panic!("the second frame is text");
    };
    assert_eq!(text.columns, 1);
    assert!(book.flows[0].auto_add_pages);
    // Reading what was written gives the same document: nothing is lost or invented on a re-save.
    let once = serde_json::to_value(&book).unwrap();
    let again =
        serde_json::to_value(serde_json::from_value::<Book>(once.clone()).unwrap()).unwrap();
    assert_eq!(once, again);
}

#[test]
fn a_key_from_a_newer_version_is_kept_and_written_back() {
    let mut value = sample();
    value["futureThing"] = json!({ "answer": 42 });
    let book: Book = serde_json::from_value(value).unwrap();
    assert_eq!(book.extensions["futureThing"]["answer"], 42);
    let written = serde_json::to_value(&book).unwrap();
    assert_eq!(written["futureThing"]["answer"], 42);
    // The keys the model knows are not duplicated into the leftovers.
    assert!(!book.extensions.contains_key("title"));
}

#[test]
fn a_group_nests_frames() {
    let group = json!({
        "type": "group", "id": "g1", "box": { "x": 0, "y": 0, "w": 10, "h": 10 },
        "children": [{
            "type": "shape", "id": "s1", "box": { "x": 1, "y": 1, "w": 2, "h": 2 },
            "shape": "round", "fill": "#fff", "style": null
        }]
    });
    let frame: Frame = serde_json::from_value(group).unwrap();
    let Frame::Group { children, base } = &frame else {
        panic!("a group");
    };
    assert_eq!(base.id.0, "g1");
    assert!(matches!(children[0], Frame::Shape(_)));
}

#[test]
fn a_shape_is_written_with_only_the_fields_it_has() {
    let shape = json!({
        "type": "shape", "id": "s1", "box": { "x": 1, "y": 1, "w": 2, "h": 2 },
        "shape": "round", "fill": "#ffffff", "style": null
    });
    let frame: Frame = serde_json::from_value(shape).unwrap();
    let Frame::Shape(found) = &frame else {
        panic!("a shape");
    };
    assert_eq!(found.shape, folio_model::Shape::Round);
    let written = serde_json::to_value(&frame).unwrap();
    assert_eq!(written["shape"], "round");
    // Nothing is written for an outline, a radius or a flip the shape does not have.
    for key in ["stroke", "strokeWidth", "radius", "flip"] {
        assert!(written.get(key).is_none(), "{key}");
    }
    // The old name is not read: no book was ever made with it.
    let old = json!({
        "type": "shape", "id": "s2", "box": { "x": 1, "y": 1, "w": 2, "h": 2 },
        "shape": "rounded", "fill": null, "style": null
    });
    assert!(serde_json::from_value::<Frame>(old).is_err());
}

#[test]
fn a_line_with_an_outline_keeps_its_fields_under_their_json_names() {
    let line = json!({
        "type": "shape", "id": "l1", "box": { "x": 0, "y": 0, "w": 100, "h": 0 },
        "shape": "line", "fill": null, "style": null,
        "stroke": "#14110f", "strokeWidth": 2.0, "flip": true
    });
    let frame: Frame = serde_json::from_value(line.clone()).unwrap();
    let written = serde_json::to_value(&frame).unwrap();
    assert_eq!(written["stroke"], "#14110f");
    assert_eq!(written["strokeWidth"], 2.0);
    assert_eq!(written["flip"], true);
}

#[test]
fn the_ignored_words_are_kept_and_an_empty_list_is_left_out() {
    let mut book: Book = serde_json::from_value(sample()).unwrap();
    assert!(book.proofing.ignore.is_empty());
    assert!(serde_json::to_value(&book)
        .unwrap()
        .get("proofing")
        .is_none());
    book.proofing.ignore = vec!["Zorblat".into()];
    let written = serde_json::to_value(&book).unwrap();
    assert_eq!(written["proofing"]["ignore"][0], "Zorblat");
    let back: Book = serde_json::from_value(written).unwrap();
    assert_eq!(back.proofing.ignore, ["Zorblat"]);
}
