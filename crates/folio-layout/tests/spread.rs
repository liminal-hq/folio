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
    (made.book, StyleSheet::from_value(&made.styles))
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
        .any(|p| p.kind == PreflightKind::OversetText && p.frame.0 == "long"));
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
        .any(|p| p.kind == PreflightKind::TextOutsideMargin && p.frame.0 == "edge"));
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
    book.flows.push(Flow {
        id: FlowId("story".into()),
        source: story,
        frames: vec![FrameId("a".into()), FrameId("b".into())],
        auto_add_pages: true,
    });
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
        .any(|p| p.kind == PreflightKind::PoemLineTooLong && p.frame.0 == "poem"));
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
