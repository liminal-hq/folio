// Lays out the sample books of every kind and checks what comes out
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use folio_layout::wire::{PreflightKind, SpreadLayout};
use folio_layout::{FontBook, Layouter, StyleSheet};
use folio_model::{
    Book, BookKind, Fit, Frame, FrameBase, FrameId, ImageFrame, ImageRef, InlineContent, Pt, Rect,
    Run, SpreadKind, StyleId, TextFrame,
};
use folio_presets::{new_book, NewBookSpec, Words};

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
        words: Words::Samples,
        ..NewBookSpec::default()
    })
    .unwrap();
    let styles = StyleSheet::from_styles(&made.book.styles);
    (made.book, styles)
}

fn layout_all(book: &Book, styles: &StyleSheet) -> Vec<SpreadLayout> {
    let fonts = FontBook::bundled();
    let layouter = Layouter::new(book, styles, &fonts);
    book.spreads
        .iter()
        .map(|s| layouter.layout_spread(&s.id).unwrap().unwrap())
        .collect()
}

fn text_frame(id: &str, style: &str, x: f64, y: f64, w: f64, h: f64, text: &str) -> Frame {
    Frame::Text(TextFrame {
        base: FrameBase {
            id: FrameId(id.into()),
            rect: Rect {
                x: Pt(x),
                y: Pt(y),
                w: Pt(w),
                h: Pt(h),
            },
            rotation: 0.0,
            locked: false,
            from_master: None,
            description: None,
        },
        style: StyleId(style.into()),
        content: Some(InlineContent {
            inline: vec![Run {
                text: text.into(),
                style: None,
                bold: false,
                italic: false,
                font: None,
                colour: None,
            }],
        }),
        flow: None,
        columns: 1,
        baseline_lock: false,
        overrides: None,
    })
}

#[test]
fn every_kind_lays_out_with_text_on_its_pages() {
    for kind in KINDS {
        let (book, styles) = sample(kind);
        let layouts = layout_all(&book, &styles);
        assert_eq!(layouts.len(), book.spreads.len(), "{kind:?}");
        let lines: usize = layouts
            .iter()
            .flat_map(|l| &l.frames)
            .map(|f| f.lines.len())
            .sum();
        assert!(lines > 0, "{kind:?} has text to draw");
        for layout in &layouts {
            for frame in &layout.frames {
                for line in &frame.lines {
                    assert!(line.height > 0.0 && line.width >= 0.0);
                    assert!(line.baseline >= line.top && line.baseline <= line.top + line.height);
                    for run in &line.runs {
                        assert!(!run.family.is_empty() && run.size > 0.0, "{kind:?}");
                    }
                }
            }
        }
    }
}

#[test]
fn the_sample_text_comes_out_as_the_sample_text() {
    let (book, styles) = sample(BookKind::Picture);
    let layouts = layout_all(&book, &styles);
    let joined: String = layouts
        .iter()
        .flat_map(|l| &l.frames)
        .flat_map(|f| &f.lines)
        .flat_map(|l| l.runs.iter().filter(|r| !r.hyphen).map(|r| r.text.as_str()))
        .collect::<Vec<_>>()
        .join(" ");
    assert!(joined.contains("Otter"), "{joined}");
}

#[test]
fn lines_stay_inside_their_frames() {
    for kind in KINDS {
        let (book, styles) = sample(kind);
        let layouts = layout_all(&book, &styles);
        for (spread, layout) in book.spreads.iter().zip(&layouts) {
            for fl in &layout.frames {
                let frame = spread
                    .frames
                    .iter()
                    .find(|f| f.base().id == fl.frame)
                    .unwrap();
                let rect = frame.base().rect;
                for line in &fl.lines {
                    assert!(line.top >= -1e-6, "{kind:?}");
                    if !fl.overset {
                        assert!(line.top + line.height <= rect.h.0 + 1e-6, "{kind:?}");
                    }
                }
            }
        }
    }
}

#[test]
fn an_unknown_spread_is_none() {
    let (book, styles) = sample(BookKind::Novel);
    let fonts = FontBook::bundled();
    let layouter = Layouter::new(&book, &styles, &fonts);
    assert!(layouter
        .layout_spread(&folio_model::SpreadId("nope".into()))
        .unwrap()
        .is_none());
}

#[test]
fn text_too_long_for_its_frame_is_overset_and_flagged_by_preflight() {
    let (mut book, styles) = sample(BookKind::Novel);
    let long = "The otter counted every star in the sky and then began again. ".repeat(40);
    let frame = text_frame("long", "body", 60.0, 60.0, 200.0, 60.0, &long);
    book.spreads[1].frames.push(frame);
    let fonts = FontBook::bundled();
    let layouter = Layouter::new(&book, &styles, &fonts);
    let layout = layouter
        .layout_spread(&book.spreads[1].id)
        .unwrap()
        .unwrap();
    let laid = layout.frames.iter().find(|f| f.frame.0 == "long").unwrap();
    assert!(laid.overset);
    assert!(!laid.lines.is_empty());
    assert!(layout
        .preflight
        .iter()
        .any(|p| p.kind == PreflightKind::OversetText
            && p.frame.as_ref().is_some_and(|f| f.0 == "long")));
}

#[test]
fn text_outside_the_safe_margin_is_flagged() {
    let (mut book, styles) = sample(BookKind::Novel);
    let near_edge = text_frame(
        "edge",
        "body",
        4.0,
        4.0,
        100.0,
        40.0,
        "Too close to the edge",
    );
    book.spreads[1].frames.push(near_edge);
    let fonts = FontBook::bundled();
    let layout = Layouter::new(&book, &styles, &fonts)
        .layout_spread(&book.spreads[1].id)
        .unwrap()
        .unwrap();
    assert!(layout
        .preflight
        .iter()
        .any(|p| p.kind == PreflightKind::TextOutsideMargin
            && p.frame.as_ref().is_some_and(|f| f.0 == "edge")));
}

#[test]
fn a_frame_inside_the_margins_is_not_flagged() {
    let (mut book, styles) = sample(BookKind::Novel);
    let m = book.margins.inside.0.max(book.margins.outside.0) + book.margins.top.0 + 20.0;
    book.spreads[1].frames.clear();
    book.spreads[1].frames.push(text_frame(
        "fine",
        "body",
        m,
        m,
        150.0,
        60.0,
        "Comfortably inside",
    ));
    let fonts = FontBook::bundled();
    let layout = Layouter::new(&book, &styles, &fonts)
        .layout_spread(&book.spreads[1].id)
        .unwrap()
        .unwrap();
    assert!(layout.preflight.is_empty(), "{:?}", layout.preflight);
}

#[test]
fn a_missing_style_falls_back_to_defaults_rather_than_failing() {
    let (mut book, styles) = sample(BookKind::Novel);
    book.spreads[1].frames.push(text_frame(
        "odd",
        "no-such-style",
        60.0,
        60.0,
        200.0,
        80.0,
        "Words",
    ));
    let fonts = FontBook::bundled();
    let layout = Layouter::new(&book, &styles, &fonts)
        .layout_spread(&book.spreads[1].id)
        .unwrap()
        .unwrap();
    let odd = layout.frames.iter().find(|f| f.frame.0 == "odd").unwrap();
    assert_eq!(odd.lines.len(), 1);
    assert_eq!(odd.lines[0].runs[0].size, 12.0);
}

#[test]
fn line_breaks_in_a_text_frame_start_new_paragraphs() {
    let (mut book, styles) = sample(BookKind::Novel);
    book.spreads[1].frames.push(text_frame(
        "two",
        "body",
        60.0,
        60.0,
        300.0,
        200.0,
        "First paragraph\nSecond paragraph",
    ));
    let fonts = FontBook::bundled();
    let layout = Layouter::new(&book, &styles, &fonts)
        .layout_spread(&book.spreads[1].id)
        .unwrap()
        .unwrap();
    let two = layout.frames.iter().find(|f| f.frame.0 == "two").unwrap();
    assert_eq!(two.lines.len(), 2);
}

#[test]
fn poetry_keeps_its_line_breaks() {
    let (mut book, styles) = sample(BookKind::Poetry);
    book.spreads[1].frames.push(text_frame(
        "poem",
        "poem",
        60.0,
        60.0,
        300.0,
        300.0,
        "Slow river\nSleeping otter\nStars",
    ));
    let fonts = FontBook::bundled();
    let layout = Layouter::new(&book, &styles, &fonts)
        .layout_spread(&book.spreads[1].id)
        .unwrap()
        .unwrap();
    let poem = layout.frames.iter().find(|f| f.frame.0 == "poem").unwrap();
    assert_eq!(poem.lines.len(), 3);
}

#[test]
fn a_larger_override_makes_bigger_text() {
    let (mut book, styles) = sample(BookKind::Novel);
    let mut frame = text_frame("big", "body", 60.0, 60.0, 300.0, 200.0, "Bigger");
    if let Frame::Text(t) = &mut frame {
        t.overrides = Some(folio_model::StyleOverrides {
            size: Some(Pt(30.0)),
            ..Default::default()
        });
    }
    book.spreads[1].frames.push(frame);
    let fonts = FontBook::bundled();
    let layout = Layouter::new(&book, &styles, &fonts)
        .layout_spread(&book.spreads[1].id)
        .unwrap()
        .unwrap();
    let big = layout.frames.iter().find(|f| f.frame.0 == "big").unwrap();
    assert_eq!(big.lines[0].runs[0].size, 30.0);
}

#[test]
fn the_wire_types_serialise_with_camel_case_keys() {
    let (book, styles) = sample(BookKind::Picture);
    let layouts = layout_all(&book, &styles);
    let json = serde_json::to_value(&layouts[1]).unwrap();
    assert!(json.get("preflight").is_some());
    let frame = &json["frames"][0];
    assert!(frame["lines"][0]["runs"][0].get("family").is_some());
    assert!(frame.get("overset").is_some());
}

#[test]
fn a_story_flows_from_a_frame_on_one_spread_into_a_frame_on_the_next() {
    use folio_model::{Flow, FlowId, FlowSlot};
    let (mut book, styles) = sample(BookKind::Novel);
    let story =
        "The otter counted the stars over the river and then the pebbles under it. ".repeat(12);
    let mut first = text_frame("a", "body", 60.0, 60.0, 200.0, 80.0, "");
    let mut second = text_frame("b", "body", 60.0, 60.0, 200.0, 400.0, "");
    for (frame, index) in [(&mut first, 0), (&mut second, 1)] {
        if let Frame::Text(t) = frame {
            t.content = None;
            t.flow = Some(FlowSlot {
                id: FlowId("story".into()),
                index,
            });
        }
    }
    book.spreads[1].frames.push(first);
    book.spreads[2].frames.push(second);
    book.flows.push(Flow::from_text(
        FlowId("story".into()),
        &story,
        vec![FrameId("a".into()), FrameId("b".into())],
    ));
    let fonts = FontBook::bundled();
    let layouter = Layouter::new(&book, &styles, &fonts);
    let one = layouter
        .layout_spread(&book.spreads[1].id)
        .unwrap()
        .unwrap();
    let two = layouter
        .layout_spread(&book.spreads[2].id)
        .unwrap()
        .unwrap();
    let a = one.frames.iter().find(|f| f.frame.0 == "a").unwrap();
    let b = two.frames.iter().find(|f| f.frame.0 == "b").unwrap();
    assert!(!a.lines.is_empty() && !b.lines.is_empty());
    assert!(!a.overset && !b.overset);
    assert!(a.lines.last().unwrap().top + a.lines.last().unwrap().height <= 80.0 + 1e-6);
    assert!(
        a.lines[0].runs[0].size > 0.0,
        "flowed runs keep their style"
    );
}

fn lay(book: &Book, styles: &StyleSheet, spread: usize) -> SpreadLayout {
    let fonts = FontBook::bundled();
    Layouter::new(book, styles, &fonts)
        .layout_spread(&book.spreads[spread].id)
        .unwrap()
        .unwrap()
}

#[test]
fn a_run_names_the_font_that_was_measured_not_the_one_asked_for() {
    let (mut book, _) = sample(BookKind::Novel);
    let styles = StyleSheet::from_value(
        &serde_json::json!({ "fontFamily": "No Such Family", "styles": {} }),
    );
    book.spreads[1].frames.push(text_frame(
        "t",
        "body",
        60.0,
        60.0,
        300.0,
        200.0,
        "Slow river",
    ));
    let layout = lay(&book, &styles, 1);
    let frame = layout.frames.iter().find(|f| f.frame.0 == "t").unwrap();
    let run = &frame.lines[0].runs[0];
    assert!(
        ["Literata", "Inter", "Source Serif 4"].contains(&run.family.as_str()),
        "{}",
        run.family
    );
}

#[test]
fn a_poem_line_wider_than_its_frame_is_flagged() {
    let (mut book, styles) = sample(BookKind::Poetry);
    book.spreads[1].frames.push(text_frame(
        "poem",
        "poem",
        60.0,
        60.0,
        80.0,
        300.0,
        "A line of verse far too long to fit in so narrow a frame\nShort",
    ));
    let layout = lay(&book, &styles, 1);
    assert!(layout
        .preflight
        .iter()
        .any(|p| p.kind == PreflightKind::PoemLineTooLong
            && p.frame.as_ref().is_some_and(|f| f.0 == "poem")));
}

#[test]
fn a_frame_whose_flow_is_gone_shows_its_own_words() {
    let (mut book, styles) = sample(BookKind::Novel);
    let mut frame = text_frame("orphan", "body", 60.0, 60.0, 300.0, 200.0, "Slow river");
    if let Frame::Text(t) = &mut frame {
        t.flow = Some(folio_model::FlowSlot {
            id: folio_model::FlowId("gone".into()),
            index: 0,
        });
    }
    book.spreads[1].frames.push(frame);
    let layout = lay(&book, &styles, 1);
    let laid = layout
        .frames
        .iter()
        .find(|f| f.frame.0 == "orphan")
        .unwrap();
    assert!(!laid.lines.is_empty());
}

#[test]
fn a_picture_at_the_fold_of_an_opener_needs_no_bleed_there() {
    let (mut book, styles) = sample(BookKind::Novel);
    let (w, h, bleed) = (book.trim.w.0, book.trim.h.0, book.bleed.0);
    let spread = &mut book.spreads[1];
    spread.kind = SpreadKind::Opener;
    spread.pages.truncate(1);
    spread.frames.clear();
    spread.frames.push(Frame::Image(ImageFrame {
        base: FrameBase {
            id: FrameId("pic".into()),
            rect: Rect {
                x: Pt(w),
                y: Pt(-bleed),
                w: Pt(w + bleed),
                h: Pt(h + bleed * 2.0),
            },
            rotation: 0.0,
            locked: false,
            from_master: None,
            description: None,
        },
        image: ImageRef {
            asset: None,
            fit: Fit::Fill,
            focus: (0.5, 0.5),
            crop: None,
        },
        placeholder_label: None,
    }));
    let layout = lay(&book, &styles, 1);
    assert!(layout
        .preflight
        .iter()
        .all(|p| p.kind != PreflightKind::PictureShortOfBleed));
}

fn with_numbers(book: &mut Book) {
    let frame = text_frame(
        "fr_pn",
        "folio",
        100.0,
        500.0,
        60.0,
        18.0,
        "{page} · {book.title}",
    );
    book.master_pages.push(folio_model::Master {
        id: folio_model::MasterId("mst_a".into()),
        name: "A".into(),
        frames: vec![frame],
        guides: Vec::new(),
        bg: None,
    });
    for spread in book
        .spreads
        .iter_mut()
        .filter(|s| s.kind != SpreadKind::Cover)
    {
        for page in &mut spread.pages {
            page.master = Some(folio_model::MasterId("mst_a".into()));
        }
    }
}

fn said(layout: &SpreadLayout, frame: &str) -> String {
    layout
        .frames
        .iter()
        .find(|f| f.frame.0 == frame)
        .map(|f| {
            f.lines
                .iter()
                .flat_map(|l| l.runs.iter().map(|r| r.text.clone()))
                .collect::<Vec<_>>()
                .join(" ")
        })
        .unwrap_or_default()
}

#[test]
fn a_master_page_repeats_on_each_page_with_its_number_and_the_title_filled_in() {
    let (mut book, styles) = sample(BookKind::Novel);
    with_numbers(&mut book);
    let title = book.title.clone();
    let layout = lay(&book, &styles, 1);
    let first = layout
        .frames
        .iter()
        .find(|f| f.frame.0.starts_with("fr_pn@"))
        .expect("the master frame is laid out");
    let origin = first.origin.expect("a master frame says where it sits");
    assert_eq!(origin.y, 500.0);
    assert!(!first.lines.is_empty());
    let text = said(&layout, &first.frame.0);
    assert!(text.starts_with("1 "), "{text}");
    assert!(
        text.contains(&title.split(' ').next().unwrap().to_owned()),
        "{text}"
    );
}

#[test]
fn the_second_page_of_a_pair_sits_a_page_across_and_has_the_next_number() {
    let (mut book, styles) = sample(BookKind::Novel);
    with_numbers(&mut book);
    let spread = book
        .spreads
        .iter()
        .position(|s| s.pages.len() == 2)
        .expect("a spread of two pages");
    let before: usize = book.spreads[..spread]
        .iter()
        .filter(|s| s.kind != SpreadKind::Cover)
        .map(|s| s.pages.len())
        .sum();
    let width = book.trim.w.0;
    let layout = lay(&book, &styles, spread);
    let masters: Vec<_> = layout
        .frames
        .iter()
        .filter(|f| f.frame.0.starts_with("fr_pn@"))
        .collect();
    assert_eq!(masters.len(), 2);
    assert_eq!(
        masters[1].origin.unwrap().x - masters[0].origin.unwrap().x,
        width
    );
    assert!(said(&layout, &masters[0].frame.0).starts_with(&format!("{} ", before + 1)));
    assert!(said(&layout, &masters[1].frame.0).starts_with(&format!("{} ", before + 2)));
}

#[test]
fn a_hidden_master_item_is_left_off_that_page_only() {
    let (mut book, styles) = sample(BookKind::Novel);
    with_numbers(&mut book);
    let spread = book
        .spreads
        .iter()
        .position(|s| s.pages.len() == 2)
        .expect("a spread of two pages");
    let page = book.spreads[spread].pages[0].id.clone();
    book.spreads[spread]
        .master_overrides
        .push(folio_model::MasterOverride {
            page_id: page,
            hide: vec![FrameId("fr_pn".into())],
        });
    let layout = lay(&book, &styles, spread);
    assert_eq!(
        layout
            .frames
            .iter()
            .filter(|f| f.frame.0.starts_with("fr_pn@"))
            .count(),
        1
    );
}

#[test]
fn the_cover_and_pages_without_a_master_have_no_master_frames() {
    let (mut book, styles) = sample(BookKind::Novel);
    with_numbers(&mut book);
    let cover = lay(&book, &styles, 0);
    assert!(cover.frames.iter().all(|f| f.origin.is_none()));
}

fn shape_frame(id: &str, shape: folio_model::Shape, w: f64, h: f64, text: Option<&str>) -> Frame {
    Frame::Shape(folio_model::ShapeFrame {
        base: FrameBase {
            id: FrameId(id.into()),
            rect: Rect {
                x: Pt(100.0),
                y: Pt(50.0),
                w: Pt(w),
                h: Pt(h),
            },
            rotation: 0.0,
            locked: false,
            from_master: None,
            description: None,
        },
        shape,
        fill: None,
        style: None,
        stroke: None,
        stroke_width: None,
        radius: None,
        flip: false,
        content: text.map(|t| InlineContent {
            inline: vec![Run {
                text: t.into(),
                style: None,
                bold: false,
                italic: false,
                font: None,
                colour: None,
            }],
        }),
        overrides: None,
    })
}

/// The first picture-book spread after the cover, with `frame` added.
fn laid_with(frame: Frame) -> (Book, SpreadLayout) {
    let (mut book, styles) = sample(BookKind::Picture);
    book.spreads[1].frames.push(frame);
    let id = book.spreads[1].id.clone();
    let fonts = FontBook::bundled();
    let layout = Layouter::new(&book, &styles, &fonts)
        .layout_spread(&id)
        .unwrap()
        .unwrap();
    (book, layout)
}

#[test]
fn words_in_a_bubble_sit_in_its_body_and_are_centred_in_it() {
    let (_, layout) = laid_with(shape_frame(
        "fr_bubble",
        folio_model::Shape::Speech,
        200.0,
        100.0,
        Some("Hello there"),
    ));
    let laid = layout
        .frames
        .iter()
        .find(|f| f.frame.0 == "fr_bubble")
        .expect("a bubble with words has a layout");
    let body = laid.text_box.expect("and says where its words go");
    // The speech bubble keeps 8% on each side and 38% at the bottom for its tail.
    for (got, want) in [
        (body.x, 16.0),
        (body.y, 8.0),
        (body.w, 168.0),
        (body.h, 54.0),
    ] {
        assert!((got - want).abs() < 1e-6, "{got} is not {want}");
    }
    assert!(!laid.lines.is_empty() && !laid.overset);
    for line in &laid.lines {
        assert!(line.x >= body.x - 1e-6 && line.x + line.width <= body.x + body.w + 1e-6);
        assert!(line.top >= body.y - 1e-6 && line.top + line.height <= body.y + body.h + 1e-6);
    }
    let top = laid.lines.first().unwrap().top - body.y;
    let last = laid.lines.last().unwrap();
    let bottom = body.y + body.h - (last.top + last.height);
    assert!((top - bottom).abs() < 1e-6, "centred top to bottom");
}

#[test]
fn a_shape_with_no_words_still_says_where_they_would_go_and_a_line_has_no_room() {
    let (_, layout) = laid_with(shape_frame(
        "fr_empty",
        folio_model::Shape::Ellipse,
        100.0,
        100.0,
        None,
    ));
    let laid = layout
        .frames
        .iter()
        .find(|f| f.frame.0 == "fr_empty")
        .unwrap();
    assert!(laid.lines.is_empty());
    assert!((laid.text_box.unwrap().x - 15.0).abs() < 1e-6);

    let (_, layout) = laid_with(shape_frame(
        "fr_line",
        folio_model::Shape::Line,
        100.0,
        0.0,
        Some("never shown"),
    ));
    assert!(layout.frames.iter().all(|f| f.frame.0 != "fr_line"));
}

#[test]
fn too_many_words_for_a_shape_are_flagged_overset() {
    let long = "Words that go on and on and on, well past what a small shape holds. ".repeat(6);
    let (_, layout) = laid_with(shape_frame(
        "fr_small",
        folio_model::Shape::Shout,
        80.0,
        50.0,
        Some(&long),
    ));
    assert!(layout
        .frames
        .iter()
        .any(|f| f.frame.0 == "fr_small" && f.overset));
    assert!(layout
        .preflight
        .iter()
        .any(|p| p.kind == PreflightKind::OversetText
            && p.frame.as_ref().is_some_and(|f| f.0 == "fr_small")));
}

fn laid_with_fallback(frame: Frame) -> SpreadLayout {
    let (mut book, styles) = sample(BookKind::Picture);
    book.spreads[1].frames.push(frame);
    let id = book.spreads[1].id.clone();
    let styles = styles.with_fallback(&StyleSheet::from_styles(&folio_presets::preset_styles()));
    let fonts = FontBook::bundled();
    Layouter::new(&book, &styles, &fonts)
        .layout_spread(&id)
        .unwrap()
        .unwrap()
}

#[test]
fn a_book_without_the_bubble_style_gets_folios_and_a_book_that_has_its_own_keeps_it() {
    use folio_model::Align;
    // The picture book's styles have no `bubble`: without the fallback its words are set as the
    // default, left aligned.
    let (_, plain) = laid_with(shape_frame(
        "fr_b",
        folio_model::Shape::Speech,
        200.0,
        100.0,
        Some("Hello"),
    ));
    let plain = plain.frames.iter().find(|f| f.frame.0 == "fr_b").unwrap();
    assert_eq!(plain.text_align, Some(Align::Left));
    // With Folio's styles behind the book's, the bubble speaks in its own centred style.
    let with = laid_with_fallback(shape_frame(
        "fr_b",
        folio_model::Shape::Speech,
        200.0,
        100.0,
        Some("Hello"),
    ));
    let with = with.frames.iter().find(|f| f.frame.0 == "fr_b").unwrap();
    assert_eq!(with.text_align, Some(Align::Center));
    // A book that defines a style of that name keeps its own.
    let own = StyleSheet::from_value(&serde_json::json!({
        "styles": { "bubble": { "sizePt": 20, "align": "right" } }
    }))
    .with_fallback(&StyleSheet::from_styles(&folio_presets::preset_styles()));
    let resolved = own.resolve(&StyleId("bubble".into()), None);
    assert_eq!(resolved.text.size_pt, 20.0);
}

#[test]
fn words_in_a_master_shape_are_laid_out_on_the_page_and_in_the_master_view() {
    let (mut book, styles) = sample(BookKind::Picture);
    book.master_pages.push(folio_model::Master {
        id: folio_model::MasterId("mst_s".into()),
        name: "S".into(),
        frames: vec![shape_frame(
            "fr_m",
            folio_model::Shape::Ellipse,
            120.0,
            80.0,
            Some("Page {page}"),
        )],
        guides: Vec::new(),
        bg: None,
    });
    for page in &mut book.spreads[1].pages {
        page.master = Some(folio_model::MasterId("mst_s".into()));
    }
    let fonts = FontBook::bundled();
    let layouter = Layouter::new(&book, &styles, &fonts);
    let spread = layouter
        .layout_spread(&book.spreads[1].id)
        .unwrap()
        .unwrap();
    let on_page = spread
        .frames
        .iter()
        .find(|f| f.frame.0.starts_with("fr_m@"))
        .expect("the master shape's words are laid out on the page");
    assert!(on_page.origin.is_some() && on_page.text_box.is_some());
    let words: String = on_page
        .lines
        .iter()
        .flat_map(|l| l.runs.iter().map(|r| r.text.as_str()))
        .collect::<Vec<_>>()
        .join(" ");
    assert!(!words.contains('{'), "{words}");
    let in_view = layouter
        .layout_master(&folio_model::MasterId("mst_s".into()))
        .unwrap()
        .unwrap();
    assert!(in_view.frames.iter().any(|f| f.frame.0 == "fr_m"));
}

#[test]
fn the_page_number_and_title_are_filled_in_for_a_frame_the_person_made() {
    let (book, styles) = sample(BookKind::Picture);
    let mut book = book;
    // A frame over the right-hand page of the first spread after the cover.
    book.spreads[2].frames.push(text_frame(
        "fr_folio",
        "body",
        book.trim.w.0 + 20.0,
        20.0,
        200.0,
        40.0,
        "Page {page} of {book.title}",
    ));
    let fonts = FontBook::bundled();
    let id = book.spreads[2].id.clone();
    let layout = Layouter::new(&book, &styles, &fonts)
        .layout_spread(&id)
        .unwrap()
        .unwrap();
    let laid = layout
        .frames
        .iter()
        .find(|f| f.frame.0 == "fr_folio")
        .unwrap();
    let words: String = laid
        .lines
        .iter()
        .flat_map(|l| l.runs.iter().map(|r| r.text.as_str()))
        .collect::<Vec<_>>()
        .join(" ");
    assert!(!words.contains('{'), "{words}");
    // Cover, then pages 1-2 on the first spread, so the right page of the next is page 4.
    assert!(words.contains("4"), "{words}");
    assert!(
        words.contains(&book.title.split(' ').next().unwrap().to_string()),
        "{words}"
    );
}

#[test]
fn a_turned_frame_is_checked_by_the_box_around_its_turned_corners() {
    let (mut book, styles) = sample(BookKind::Novel);
    let (w, h) = (book.trim.w.0, book.trim.h.0);
    // Tall and narrow in the middle of the first page: well inside the margins upright, but on its
    // side it runs out past both edges of the page.
    let mut post = text_frame(
        "post",
        "body",
        w / 2.0 - 5.0,
        h * 0.15,
        10.0,
        h * 0.7,
        "Tall",
    );
    book.spreads[1].frames.push(post.clone());
    let fonts = FontBook::bundled();
    let flagged = |book: &folio_model::Book| {
        Layouter::new(book, &styles, &fonts)
            .layout_spread(&book.spreads[1].id)
            .unwrap()
            .unwrap()
            .preflight
            .iter()
            .any(|p| {
                p.kind == PreflightKind::TextOutsideMargin
                    && p.frame.as_ref().is_some_and(|f| f.0 == "post")
            })
    };
    assert!(!flagged(&book), "upright, it is inside the margins");
    book.spreads[1].frames.pop();
    post.base_mut().rotation = 90.0;
    book.spreads[1].frames.push(post);
    assert!(flagged(&book), "turned, it is not");
}

fn table_frame(id: &str, w: f64, h: f64, words: &[&[&str]]) -> Frame {
    let mut table = folio_model::TableFrame::blank(
        FrameBase {
            id: FrameId(id.into()),
            rect: Rect {
                x: Pt(100.0),
                y: Pt(50.0),
                w: Pt(w),
                h: Pt(h),
            },
            rotation: 0.0,
            locked: false,
            from_master: None,
            description: None,
        },
        words.len(),
        words[0].len(),
    );
    for (r, row) in words.iter().enumerate() {
        for (c, text) in row.iter().enumerate() {
            table.rows[r][c].content = (!text.is_empty()).then(|| InlineContent {
                inline: vec![Run {
                    text: (*text).into(),
                    style: None,
                    bold: false,
                    italic: false,
                    font: None,
                    colour: None,
                }],
            });
        }
    }
    table.cols = vec![0.25, 0.75].into_iter().take(words[0].len()).collect();
    if table.cols.len() == 1 {
        table.cols = vec![1.0];
    }
    Frame::Table(table)
}

#[test]
fn each_cell_of_a_table_is_laid_out_in_its_own_room_from_the_tables_top_left() {
    let (_, layout) = laid_with(table_frame(
        "fr_table",
        400.0,
        120.0,
        &[&["Name", "What it is"], &["Pip", ""]],
    ));
    let cells: Vec<_> = layout
        .frames
        .iter()
        .filter(|f| f.frame.0.starts_with("fr_table#"))
        .collect();
    assert_eq!(cells.len(), 4, "one layout for every cell, empty or not");
    let cell = |name: &str| cells.iter().find(|f| f.frame.0 == name).unwrap();
    let (a, b, c) = (
        cell("fr_table#0.0"),
        cell("fr_table#0.1"),
        cell("fr_table#1.0"),
    );
    let (ra, rb, rc) = (
        a.text_box.unwrap(),
        b.text_box.unwrap(),
        c.text_box.unwrap(),
    );
    // Columns take their shares of 400 pt; rows are 60 pt each.
    assert!((ra.w - (100.0 - 8.0)).abs() < 1e-6 && (rb.w - (300.0 - 8.0)).abs() < 1e-6);
    assert!(
        (rb.x - (100.0 + 4.0)).abs() < 1e-6,
        "the second column starts after the first"
    );
    assert!(
        (rc.y - (60.0 + 3.0)).abs() < 1e-6,
        "the second row starts after the first"
    );
    assert!(!a.lines.is_empty() && b.lines.len() == 1 && c.lines.len() == 1);
    assert!(cell("fr_table#1.1").lines.is_empty());
    for (laid, room) in [(a, ra), (b, rb), (c, rc)] {
        for line in &laid.lines {
            assert!(line.x >= room.x - 1e-6 && line.x + line.width <= room.x + room.w + 1e-6);
            assert!(line.top >= room.y - 1e-6);
        }
    }
    assert!(layout
        .preflight
        .iter()
        .all(|p| p.kind != PreflightKind::OversetText));
}

#[test]
fn a_table_with_too_much_in_a_cell_is_flagged_overset_once() {
    let long = "words ".repeat(200);
    let (_, layout) = laid_with(table_frame(
        "fr_table",
        200.0,
        30.0,
        &[&[long.as_str(), long.as_str()]],
    ));
    assert!(layout.frames.iter().any(|f| f.overset));
    let flagged = layout
        .preflight
        .iter()
        .filter(|p| {
            p.kind == PreflightKind::OversetText
                && p.frame.as_ref().is_some_and(|f| f.0 == "fr_table")
        })
        .count();
    assert_eq!(flagged, 1);
}

#[test]
fn header_rows_are_set_bold() {
    let (mut book, styles) = sample(BookKind::Picture);
    let Frame::Table(mut table) =
        table_frame("fr_table", 400.0, 120.0, &[&["Head", "Head"], &["a", "b"]])
    else {
        unreachable!()
    };
    table.head = 1;
    book.spreads[1].frames.push(Frame::Table(table));
    let fonts = FontBook::bundled();
    let layout = Layouter::new(&book, &styles, &fonts)
        .layout_spread(&book.spreads[1].id)
        .unwrap()
        .unwrap();
    let weight = |id: &str| {
        let f = layout.frames.iter().find(|f| f.frame.0 == id).unwrap();
        f.lines[0].runs[0].weight
    };
    assert!(weight("fr_table#0.0") > weight("fr_table#1.0"));
}

fn picture_frame(id: &str, asset: &str, w: f64, h: f64, fit: Fit) -> Frame {
    Frame::Image(ImageFrame {
        base: FrameBase {
            id: FrameId(id.into()),
            rect: Rect {
                x: Pt(50.0),
                y: Pt(50.0),
                w: Pt(w),
                h: Pt(h),
            },
            rotation: 0.0,
            locked: false,
            from_master: None,
            description: None,
        },
        image: ImageRef {
            asset: Some(asset.into()),
            fit,
            focus: (0.5, 0.5),
            crop: None,
        },
        placeholder_label: None,
    })
}

fn kinds_for(
    book: &Book,
    styles: &StyleSheet,
    facts: &folio_layout::AssetFacts,
    spread: usize,
) -> Vec<(PreflightKind, Option<String>, Option<String>)> {
    let fonts = FontBook::bundled();
    Layouter::new(book, styles, &fonts)
        .with_assets(facts)
        .layout_spread(&book.spreads[spread].id)
        .unwrap()
        .unwrap()
        .preflight
        .into_iter()
        .filter(|p| {
            matches!(
                p.kind,
                PreflightKind::PictureMissing
                    | PreflightKind::LowResolution
                    | PreflightKind::VeryLowResolution
                    | PreflightKind::BleedNeeded
            )
        })
        .map(|p| (p.kind, p.frame.map(|f| f.0), p.page.map(|f| f.0)))
        .collect()
}

fn facts(entries: &[(&str, folio_layout::AssetFact)]) -> folio_layout::AssetFacts {
    folio_layout::AssetFacts(entries.iter().map(|(k, v)| ((*k).to_owned(), *v)).collect())
}

#[test]
fn a_picture_frame_is_flagged_when_missing_or_too_coarse_for_print() {
    use folio_layout::AssetFact::{Raster, Vector};
    let (mut book, styles) = sample(BookKind::Novel);
    // Each box is 288 × 288 pt (4 in): 1200 px is 300 ppi, 900 is 225, 500 is 125.
    for (id, asset) in [
        ("sharp", "assets/images/a.png"),
        ("soft", "assets/images/b.png"),
        ("blurry", "assets/images/c.png"),
        ("gone", "assets/images/d.png"),
        ("drawn", "assets/images/e.svg"),
    ] {
        book.spreads[1]
            .frames
            .push(picture_frame(id, asset, 288.0, 288.0, Fit::Fill));
    }
    let facts = facts(&[
        ("assets/images/a.png", Raster { w: 1200, h: 1200 }),
        ("assets/images/b.png", Raster { w: 900, h: 900 }),
        ("assets/images/c.png", Raster { w: 500, h: 500 }),
        ("assets/images/e.svg", Vector),
    ]);
    let found = kinds_for(&book, &styles, &facts, 1);
    let of = |kind| {
        found
            .iter()
            .filter(|f| f.0 == kind)
            .filter_map(|f| f.1.clone())
            .collect::<Vec<_>>()
    };
    assert_eq!(of(PreflightKind::LowResolution), ["soft"]);
    assert_eq!(of(PreflightKind::VeryLowResolution), ["blurry"]);
    assert_eq!(of(PreflightKind::PictureMissing), ["gone"]);
}

#[test]
fn how_a_picture_sits_changes_how_sharp_it_is() {
    use folio_layout::AssetFact::Raster;
    let (mut book, styles) = sample(BookKind::Novel);
    // 1200 × 600 px in a 288 pt square: filled it is scaled by the short side, so 600 px over 4 in
    // is 150 ppi; fitted whole it is 1200 px over 4 in, 300 ppi.
    book.spreads[1].frames.push(picture_frame(
        "fill",
        "assets/images/a.png",
        288.0,
        288.0,
        Fit::Fill,
    ));
    book.spreads[1].frames.push(picture_frame(
        "fit",
        "assets/images/a.png",
        288.0,
        288.0,
        Fit::Fit,
    ));
    let facts = facts(&[("assets/images/a.png", Raster { w: 1200, h: 600 })]);
    let found = kinds_for(&book, &styles, &facts, 1);
    assert!(found.contains(&(PreflightKind::LowResolution, Some("fill".into()), None)));
    assert!(found.iter().all(|f| f.1.as_deref() != Some("fit")));
}

#[test]
fn a_picture_background_is_checked_at_the_page_with_its_bleed_and_covers_count() {
    use folio_layout::AssetFact::Raster;
    let (mut book, styles) = sample(BookKind::Picture);
    let page = book.spreads[0].pages[0].id.clone();
    book.spreads[0].pages[0].bg = Some(folio_model::Background::Picture {
        asset: "assets/images/bg.png".into(),
        fit: folio_model::BgFit::Fill,
        focus: (0.5, 0.5),
    });
    // The cover is trim plus bleed all round: a 2000 px picture over about 8.7 in is under 300 ppi.
    let facts = facts(&[("assets/images/bg.png", Raster { w: 2000, h: 2000 })]);
    let found = kinds_for(&book, &styles, &facts, 0);
    assert_eq!(
        found,
        [(PreflightKind::LowResolution, None, Some(page.0.clone()))]
    );
    // Gone from the book: missing, at the page.
    let none = folio_layout::AssetFacts::default();
    assert_eq!(
        kinds_for(&book, &styles, &none, 0),
        [(PreflightKind::PictureMissing, None, Some(page.0.clone()))]
    );
    // No bleed: any background is a warning, whatever it is.
    book.bleed = Pt(0.0);
    book.spreads[0].pages[0].bg = Some(folio_model::Background::Colour {
        colour: folio_model::Colour("#ffffff".into()),
    });
    assert_eq!(
        kinds_for(&book, &styles, &none, 0),
        [(PreflightKind::BleedNeeded, None, Some(page.0))]
    );
}

#[test]
fn pictures_are_not_checked_without_facts_about_them() {
    let (mut book, styles) = sample(BookKind::Novel);
    book.spreads[1].frames.push(picture_frame(
        "gone",
        "assets/images/d.png",
        288.0,
        288.0,
        Fit::Fill,
    ));
    let fonts = FontBook::bundled();
    let layout = Layouter::new(&book, &styles, &fonts)
        .layout_spread(&book.spreads[1].id)
        .unwrap()
        .unwrap();
    assert!(layout
        .preflight
        .iter()
        .all(|p| p.kind != PreflightKind::PictureMissing));
}

#[test]
fn page_and_section_tokens_follow_the_sections_of_the_book() {
    let (mut book, styles) = sample(BookKind::Novel);
    book.spreads[1].section = Some(folio_model::Section {
        name: "Front matter".into(),
        numbering: folio_model::Numbering::Roman,
        start: Some(3),
    });
    book.spreads[1].frames.push(text_frame(
        "tok",
        "body",
        20.0,
        20.0,
        200.0,
        30.0,
        "{page} in {section}",
    ));
    book.spreads[2].frames.push(text_frame(
        "later",
        "body",
        20.0,
        20.0,
        200.0,
        30.0,
        "{page} in [{section}]",
    ));
    let said = |spread: usize, frame: &str| -> String {
        let fonts = FontBook::bundled();
        let layout = Layouter::new(&book, &styles, &fonts)
            .layout_spread(&book.spreads[spread].id)
            .unwrap()
            .unwrap();
        layout
            .frames
            .iter()
            .find(|f| f.frame.0 == frame)
            .unwrap()
            .lines
            .iter()
            .flat_map(|l| l.runs.iter().map(|r| r.text.clone()))
            .collect::<Vec<_>>()
            .join("")
            .replace(' ', "")
    };
    // The first page after the cover is page 3 of the front matter, in Roman.
    assert!(
        said(1, "tok").contains("iiiinFrontmatter"),
        "{}",
        said(1, "tok")
    );
    // The next spread carries the section on; its page follows the count.
    let pages_before: usize = book.spreads[1].pages.len();
    let want = folio_model::roman(3 + pages_before as u32);
    let later = said(2, "later");
    assert!(
        later.contains(&format!("{want}in[Frontmatter]")),
        "{later} (wanted {want})"
    );
}

/// Marks every occurrence of the given words, as a speller would.
fn flag<'a>(wrong: &'a [&'a str]) -> impl Fn(&str) -> Vec<std::ops::Range<usize>> + 'a {
    move |text: &str| {
        let mut found = Vec::new();
        let mut at = 0;
        for word in text.split(' ') {
            if wrong.contains(&word) {
                found.push(at..at + word.len());
            }
            at += word.len() + 1;
        }
        found
    }
}

fn marked_runs(layout: &SpreadLayout, frame: &str) -> Vec<(String, Vec<(u32, u32)>)> {
    layout
        .frames
        .iter()
        .find(|f| f.frame.0 == frame)
        .unwrap()
        .lines
        .iter()
        .flat_map(|l| l.runs.iter())
        .filter(|r| !r.miss.is_empty())
        .map(|r| (r.text.clone(), r.miss.clone()))
        .collect()
}

#[test]
fn misspelt_words_are_marked_on_the_runs_that_show_them_wherever_the_lines_break() {
    let (mut book, styles) = sample(BookKind::Novel);
    let text =
        "A small ottr sat by the zorb and watched another ottr swim past the old stone bridge \
                until the light was gone and every last zorb had gone home to sleep";
    book.spreads[1]
        .frames
        .push(text_frame("tx", "body", 20.0, 20.0, 120.0, 200.0, text));
    let fonts = FontBook::bundled();
    let wrong = ["ottr", "zorb"];
    let finder = flag(&wrong);
    let layout = Layouter::new(&book, &styles, &fonts)
        .with_proofing(&finder)
        .layout_spread(&book.spreads[1].id)
        .unwrap()
        .unwrap();
    let marked = marked_runs(&layout, "tx");
    // Two of each, each marked across the whole of its run; none on a word that is right.
    assert_eq!(marked.len(), 4, "{marked:?}");
    for (run, miss) in &marked {
        assert!(run == "ottr" || run == "zorb", "{run}");
        assert_eq!(miss, &[(0, 4)]);
    }
    // Without a speller nothing is marked.
    let plain = Layouter::new(&book, &styles, &fonts)
        .layout_spread(&book.spreads[1].id)
        .unwrap()
        .unwrap();
    assert!(marked_runs(&plain, "tx").is_empty());
}

#[test]
fn the_words_in_shapes_and_table_cells_are_marked_too() {
    let (mut book, styles) = sample(BookKind::Picture);
    book.spreads[1].frames.push(shape_frame(
        "sh",
        folio_model::Shape::Speech,
        300.0,
        200.0,
        Some("what a zorb this is"),
    ));
    book.spreads[1].frames.push(table_frame(
        "tb",
        400.0,
        90.0,
        &[&["fine", "a zorb"], &["", "ottr"]],
    ));
    let fonts = FontBook::bundled();
    let wrong = ["zorb", "ottr"];
    let finder = flag(&wrong);
    let layout = Layouter::new(&book, &styles, &fonts)
        .with_proofing(&finder)
        .layout_spread(&book.spreads[1].id)
        .unwrap()
        .unwrap();
    assert_eq!(marked_runs(&layout, "sh").len(), 1);
    assert_eq!(marked_runs(&layout, "tb#0.1").len(), 1);
    assert_eq!(marked_runs(&layout, "tb#1.1").len(), 1);
    assert!(marked_runs(&layout, "tb#0.0").is_empty());
}

#[test]
fn a_run_can_have_its_own_typeface_and_colour_and_the_others_keep_the_styles() {
    let mut frame = text_frame("mixed", "body", 60.0, 60.0, 300.0, 80.0, "plain");
    let Frame::Text(text) = &mut frame else {
        unreachable!()
    };
    text.content.as_mut().unwrap().inline.push(Run {
        text: " fancy".into(),
        style: None,
        bold: false,
        italic: false,
        font: Some("Inter".into()),
        colour: Some(folio_model::Colour("#b91c1c".into())),
    });
    let (_, layout) = laid_with(frame);
    let runs: Vec<_> = layout
        .frames
        .iter()
        .find(|f| f.frame.0 == "mixed")
        .unwrap()
        .lines
        .iter()
        .flat_map(|l| &l.runs)
        .collect();
    let plain = runs.iter().find(|r| r.text.contains("plain")).unwrap();
    let fancy = runs.iter().find(|r| r.text.contains("fancy")).unwrap();
    assert_eq!(fancy.family, "Inter");
    assert_eq!(fancy.colour.as_ref().map(|c| c.0.as_str()), Some("#b91c1c"));
    assert_ne!(plain.family, "Inter");
    assert_eq!(plain.colour, None);
}

#[test]
fn the_families_a_book_asks_for_come_from_its_look_styles_overrides_and_runs() {
    use folio_layout::families::families_used;
    let (mut book, _) = sample(BookKind::Picture);
    let before = families_used(&book);
    assert!(before.contains(&book.styles.font_family));
    assert!(!before.iter().any(|f| f == "Zed Sans"));

    let mut frame = text_frame("fonts", "body", 60.0, 60.0, 300.0, 80.0, "words");
    let Frame::Text(text) = &mut frame else {
        unreachable!()
    };
    text.overrides = Some(folio_model::StyleOverrides {
        font: Some("Zed Sans".into()),
        ..Default::default()
    });
    text.content.as_mut().unwrap().inline.push(Run {
        text: " more".into(),
        style: None,
        bold: false,
        italic: false,
        font: Some("Quill Mono".into()),
        colour: None,
    });
    book.spreads[1].frames.push(frame);
    let after = families_used(&book);
    assert!(after.contains(&"Zed Sans".to_string()));
    assert!(after.contains(&"Quill Mono".to_string()));
    let mut sorted = after.clone();
    sorted.sort();
    sorted.dedup();
    assert_eq!(after, sorted, "sorted, one of each");
}

#[test]
fn a_frame_in_a_font_nobody_has_is_flagged_once_with_the_family_and_one_the_book_keeps_is_not() {
    use folio_layout::wire::PreflightKind;
    use folio_layout::{BookFace, FontBook};
    let (mut book, styles) = sample(BookKind::Picture);
    let mut frame = text_frame("odd", "body", 60.0, 60.0, 300.0, 80.0, "words");
    let Frame::Text(text) = &mut frame else {
        unreachable!()
    };
    text.overrides = Some(folio_model::StyleOverrides {
        font: Some("Zed Sans".into()),
        ..Default::default()
    });
    book.spreads[1].frames.push(frame);
    let id = book.spreads[1].id.clone();

    let base = FontBook::bundled();
    let missing = Layouter::new(&book, &styles, &base)
        .layout_spread(&id)
        .unwrap()
        .unwrap();
    let flagged: Vec<_> = missing
        .preflight
        .iter()
        .filter(|i| i.kind == PreflightKind::FontMissing)
        .collect();
    assert_eq!(flagged.len(), 1);
    assert_eq!(flagged[0].family.as_deref(), Some("Zed Sans"));
    assert_eq!(flagged[0].frame.as_ref().map(|f| f.0.as_str()), Some("odd"));

    // The same book with the font kept in it has nothing missing.
    let inter = base
        .resolve(&folio_layout::FontRequest::new("Inter", 400, false))
        .unwrap()
        .font;
    let kept = base.layered(vec![BookFace {
        family: "Zed Sans".into(),
        weight: 400,
        italic: false,
        index: 0,
        data: std::sync::Arc::clone(inter.data()),
    }]);
    let fine = Layouter::new(&book, &styles, &kept)
        .layout_spread(&id)
        .unwrap()
        .unwrap();
    assert!(fine
        .preflight
        .iter()
        .all(|i| i.kind != PreflightKind::FontMissing));
}

#[test]
fn the_look_and_bundled_families_are_never_called_missing() {
    use folio_layout::wire::PreflightKind;
    use folio_layout::FontBook;
    for kind in KINDS {
        let (book, styles) = sample(kind);
        let fonts = FontBook::bundled();
        for layout in layout_all_with(&book, &styles, &fonts) {
            assert!(
                layout
                    .preflight
                    .iter()
                    .all(|i| i.kind != PreflightKind::FontMissing),
                "{kind:?}"
            );
        }
    }
}

fn layout_all_with(
    book: &Book,
    styles: &StyleSheet,
    fonts: &folio_layout::FontBook,
) -> Vec<SpreadLayout> {
    let layouter = Layouter::new(book, styles, fonts);
    book.spreads
        .iter()
        .map(|s| layouter.layout_spread(&s.id).unwrap().unwrap())
        .collect()
}

#[test]
fn a_picture_frame_with_no_picture_is_a_placeholder_and_one_without_a_description_is_noted() {
    let (mut book, styles) = sample(BookKind::Picture);
    let mut empty = picture_frame("empty", "x", 100.0, 100.0, Fit::Fill);
    let mut described = picture_frame("described", "a.png", 100.0, 100.0, Fit::Fill);
    let bare = picture_frame("bare", "b.png", 100.0, 100.0, Fit::Fill);
    if let Frame::Image(i) = &mut empty {
        i.image.asset = None;
    }
    if let Frame::Image(i) = &mut described {
        i.base.description = Some("An otter on a lily pad".into());
    }
    book.spreads[2].frames.clear();
    book.spreads[2].frames.extend([empty, described, bare]);
    let layouts = layout_all(&book, &styles);
    let found: Vec<(PreflightKind, &str)> = layouts[2]
        .preflight
        .iter()
        .filter(|i| {
            matches!(
                i.kind,
                PreflightKind::PicturePlaceholder | PreflightKind::AltMissing
            )
        })
        .map(|i| (i.kind, i.frame.as_ref().unwrap().0.as_str()))
        .collect();
    assert!(found.contains(&(PreflightKind::PicturePlaceholder, "empty")));
    assert!(found.contains(&(PreflightKind::AltMissing, "bare")));
    assert!(!found.iter().any(|(_, frame)| *frame == "described"));
    assert_eq!(
        PreflightKind::PicturePlaceholder.severity(),
        folio_layout::wire::Severity::Error
    );
    assert_eq!(
        PreflightKind::AltMissing.severity(),
        folio_layout::wire::Severity::Note
    );
    assert!(layouts[2]
        .preflight
        .iter()
        .all(|i| i.severity == i.kind.severity()));
}

#[test]
fn a_word_in_a_script_the_font_lacks_is_set_in_a_fallback_and_what_nothing_covers_is_flagged() {
    use folio_layout::wire::PreflightKind;
    let (mut book, styles) = sample(BookKind::Picture);
    book.spreads[1].frames.push(text_frame(
        "cyr",
        "body",
        60.0,
        60.0,
        300.0,
        60.0,
        "Привет, Pip",
    ));
    book.spreads[1].frames.push(text_frame(
        "han",
        "body",
        60.0,
        140.0,
        300.0,
        60.0,
        "Pip 中 pip",
    ));
    let id = book.spreads[1].id.clone();
    let fonts = FontBook::bundled();
    let layout = Layouter::new(&book, &styles, &fonts)
        .layout_spread(&id)
        .unwrap()
        .unwrap();
    let cyr = layout.frames.iter().find(|f| f.frame.0 == "cyr").unwrap();
    let families: std::collections::BTreeSet<&str> = cyr
        .lines
        .iter()
        .flat_map(|l| &l.runs)
        .map(|r| r.family.as_str())
        .collect();
    assert!(
        families.len() >= 2,
        "the Cyrillic is in a fallback: {families:?}"
    );
    let flagged: Vec<_> = layout
        .preflight
        .iter()
        .filter(|i| i.kind == PreflightKind::MissingGlyphs)
        .collect();
    assert_eq!(flagged.len(), 1);
    assert_eq!(flagged[0].chars.as_deref(), Some("中"));
    assert_eq!(flagged[0].frame.as_ref().map(|f| f.0.as_str()), Some("han"));
}

#[test]
fn a_story_keeps_the_emphasis_and_paragraph_style_of_its_paragraphs() {
    use folio_model::{Flow, FlowId, FlowSlot, StoryPara, StyleId};
    let (mut book, styles) = sample(BookKind::Novel);
    let mut frame = text_frame("a", "body", 60.0, 60.0, 300.0, 400.0, "");
    if let Frame::Text(t) = &mut frame {
        t.content = None;
        t.flow = Some(FlowSlot {
            id: FlowId("story".into()),
            index: 0,
        });
    }
    book.spreads[1].frames.push(frame);
    let mut flow = Flow::from_text(
        FlowId("story".into()),
        "A chapter\nThe otter slept.",
        vec![FrameId("a".into())],
    );
    flow.paragraphs[0].style = Some(StyleId("h1".into()));
    let mut second = StoryPara::plain("The otter ");
    second.inline.push(folio_model::Run {
        text: "slept".into(),
        style: None,
        bold: true,
        italic: false,
        font: None,
        colour: None,
    });
    flow.paragraphs[1] = second;
    book.flows.push(flow);
    let laid = lay(&book, &styles, 1);
    let lines = &laid.frames.iter().find(|f| f.frame.0 == "a").unwrap().lines;
    // The chapter title is set larger than the body, and the bold word is bold.
    assert!(lines[0].runs[0].size > lines[1].runs[0].size, "{lines:?}");
    assert!(lines[1].runs.iter().any(|r| r.weight >= 700));
}

#[test]
fn a_story_says_how_many_more_frames_it_needs_and_needs_none_once_they_are_there() {
    use folio_model::{Flow, FlowId, FlowSlot};
    let (mut book, styles) = sample(BookKind::Novel);
    let story =
        "The otter counted the stars over the river and then the pebbles under it. ".repeat(20);
    let make = |name: &str| {
        let mut frame = text_frame(name, "body", 60.0, 60.0, 200.0, 120.0, "");
        if let Frame::Text(t) = &mut frame {
            t.content = None;
            t.flow = Some(FlowSlot {
                id: FlowId("story".into()),
                index: 0,
            });
        }
        frame
    };
    book.spreads[1].frames.push(make("a"));
    book.flows.push(Flow::from_text(
        FlowId("story".into()),
        &story,
        vec![FrameId("a".into())],
    ));
    let fonts = FontBook::bundled();
    let needed = Layouter::new(&book, &styles, &fonts)
        .frames_needed(&FlowId("story".into()))
        .unwrap();
    assert!(needed >= 2, "{needed}");
    // With exactly that many frames more, nothing is left over; with one fewer, something is.
    let mut more = book.clone();
    let names: Vec<String> = (0..needed).map(|i| format!("x{i}")).collect();
    for name in &names {
        more.spreads[2].frames.push(make(name));
        more.flows
            .last_mut()
            .unwrap()
            .frames
            .push(FrameId(name.clone()));
    }
    let laid = Layouter::new(&more, &styles, &fonts)
        .frames_needed(&FlowId("story".into()))
        .unwrap();
    assert_eq!(laid, 0);
    let mut fewer = book.clone();
    for name in &names[..needed - 1] {
        fewer.spreads[2].frames.push(make(name));
        fewer
            .flows
            .last_mut()
            .unwrap()
            .frames
            .push(FrameId(name.clone()));
    }
    let still = Layouter::new(&fewer, &styles, &fonts)
        .frames_needed(&FlowId("story".into()))
        .unwrap();
    assert_eq!(still, 1);
    // A story that fits needs none, and a flow that is not there needs none.
    *book.flows.last_mut().unwrap() =
        Flow::from_text(FlowId("story".into()), "Short.", vec![FrameId("a".into())]);
    let layouter = Layouter::new(&book, &styles, &fonts);
    assert_eq!(layouter.frames_needed(&FlowId("story".into())).unwrap(), 0);
    assert_eq!(layouter.frames_needed(&FlowId("nope".into())).unwrap(), 0);
}

#[test]
fn spreads_laid_out_with_a_shared_flow_cache_are_the_same_as_without_one() {
    use folio_layout::FlowCache;
    let (book, styles) = sample(BookKind::Novel);
    let fonts = FontBook::bundled();
    let cache = FlowCache::new();
    for spread in &book.spreads {
        let alone = Layouter::new(&book, &styles, &fonts)
            .layout_spread(&spread.id)
            .unwrap()
            .unwrap();
        let shared = Layouter::new(&book, &styles, &fonts)
            .with_flow_cache(&cache, 7)
            .layout_spread(&spread.id)
            .unwrap()
            .unwrap();
        assert_eq!(
            serde_json::to_value(&alone).unwrap(),
            serde_json::to_value(&shared).unwrap()
        );
    }
}

#[test]
fn a_swatch_that_is_missing_or_lays_down_too_much_ink_is_reported_once_for_the_spread() {
    use folio_layout::wire::PreflightKind;
    use folio_model::{Background, BookSwatch, Colour, Ink, SwatchId};
    let (mut book, styles) = sample(BookKind::Novel);
    book.styles.swatches = vec![
        BookSwatch {
            id: SwatchId("heavy".into()),
            name: "Heavy black".into(),
            display: Colour("#000000".into()),
            ink: Some(Ink {
                c: 100,
                m: 100,
                y: 100,
                k: 100,
            }),
        },
        BookSwatch {
            id: SwatchId("fine".into()),
            name: "Fine".into(),
            display: Colour("#336699".into()),
            ink: Some(Ink {
                c: 60,
                m: 40,
                y: 40,
                k: 100,
            }),
        },
    ];
    let reference = |id: &str| Background::Colour {
        colour: Colour(format!("swatch:{id}")),
    };
    book.spreads[1].pages[0].bg = Some(reference("heavy"));
    book.spreads[1].bg_span = Some(reference("heavy"));
    book.spreads[2].pages[0].bg = Some(reference("gone"));
    book.spreads[3].pages[0].bg = Some(reference("fine"));
    let kinds = |spread: usize| -> Vec<(PreflightKind, Option<String>)> {
        lay(&book, &styles, spread)
            .preflight
            .into_iter()
            .filter(|i| {
                matches!(
                    i.kind,
                    PreflightKind::InkOverLimit | PreflightKind::SwatchMissing
                )
            })
            .map(|i| (i.kind, i.family))
            .collect()
    };
    assert_eq!(
        kinds(1),
        vec![(PreflightKind::InkOverLimit, Some("Heavy black".into()))]
    );
    assert_eq!(
        kinds(2),
        vec![(PreflightKind::SwatchMissing, Some("gone".into()))]
    );
    // 240% in all is within what a press holds.
    assert!(kinds(3).is_empty());
}

#[test]
fn small_words_in_a_rich_black_are_warned_about_and_large_ones_are_not() {
    use folio_layout::wire::PreflightKind;
    use folio_model::{BookSwatch, Colour, Ink, StyleDef, SwatchId};
    let (mut book, styles) = sample(BookKind::Novel);
    book.styles.swatches = vec![BookSwatch {
        id: SwatchId("rich".into()),
        name: "Rich black".into(),
        display: Colour("#000000".into()),
        ink: Some(Ink {
            c: 60,
            m: 40,
            y: 40,
            k: 100,
        }),
    }];
    let small = text_frame("small", "body", 20.0, 20.0, 200.0, 40.0, "Fine print");
    let big = text_frame("big", "body", 20.0, 80.0, 200.0, 40.0, "Heading");
    let put = |frame: Frame, size: f64, book: &mut Book| {
        let Frame::Text(mut t) = frame else {
            unreachable!()
        };
        let style = format!("s{size}");
        t.style = StyleId(style.clone());
        let def = StyleDef {
            size_pt: Some(size),
            colour: Some(Colour("swatch:rich".into())),
            ..StyleDef::default()
        };
        book.styles.styles.insert(style, def);
        book.spreads[1].frames.push(Frame::Text(t));
    };
    put(small, 9.0, &mut book);
    put(big, 24.0, &mut book);
    let found: Vec<_> = lay(&book, &styles, 1)
        .preflight
        .into_iter()
        .filter(|i| i.kind == PreflightKind::RichBlackSmallText)
        .collect();
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].family.as_deref(), Some("Rich black"));
    assert_eq!(found[0].frame.as_ref().map(|f| f.0.as_str()), Some("small"));
}

#[test]
fn a_frame_in_a_story_says_which_paragraph_its_words_begin_with() {
    let (book, styles) = sample(BookKind::Novel);
    let mut firsts: Vec<(String, Option<u32>)> = Vec::new();
    for index in 0..book.spreads.len() {
        for frame in lay(&book, &styles, index).frames {
            firsts.push((frame.frame.0.clone(), frame.first_paragraph));
        }
    }
    let flow = &book.flows[0];
    let first = firsts
        .iter()
        .find(|(id, _)| flow.frames.first().is_some_and(|f| f.0 == *id))
        .expect("the story's first frame is laid out");
    assert_eq!(first.1, Some(0));
    // A frame with words of its own is not in a story, so it names no paragraph.
    let own = firsts
        .iter()
        .filter(|(id, _)| !flow.frames.iter().any(|f| f.0 == *id))
        .all(|(_, p)| p.is_none());
    assert!(own);
}
