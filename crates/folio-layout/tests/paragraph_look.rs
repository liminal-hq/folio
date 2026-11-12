// Checks that what a paragraph sets for itself (spacing, line spacing, indents, alignment) reaches the layout
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use folio_layout::wire::FrameLayout;
use folio_layout::{FontBook, Layouter, StyleSheet};
use folio_model::{
    Align, Book, BookKind, Frame, FrameBase, FrameId, InlineContent, ParaLook, Pt, Rect, Run,
    StyleId, TextFrame,
};
use folio_presets::{new_book, NewBookSpec, Words};

const WIDTH: f64 = 300.0;

fn lay(text: &str, looks: Vec<ParaLook>) -> FrameLayout {
    let made = new_book(&NewBookSpec {
        kind: BookKind::Poetry,
        words: Words::Samples,
        ..NewBookSpec::default()
    })
    .unwrap();
    let mut book: Book = made.book;
    let styles = StyleSheet::from_styles(&book.styles);
    book.spreads[1].frames.push(Frame::Text(TextFrame {
        base: FrameBase {
            id: FrameId("tx".into()),
            rect: Rect {
                x: Pt(20.0),
                y: Pt(20.0),
                w: Pt(WIDTH),
                h: Pt(600.0),
            },
            rotation: 0.0,
            locked: false,
            from_master: None,
            description: None,
            decorative: false,
        },
        style: StyleId("body".into()),
        content: Some(InlineContent {
            inline: vec![Run {
                text: text.into(),
                style: None,
                bold: false,
                italic: false,
                font: None,
                colour: None,
                ..Default::default()
            }],
            looks: Some(looks),
        }),
        flow: None,
        columns: 1,
        baseline_lock: false,
        overrides: None,
    }));
    let fonts = FontBook::bundled();
    let layouter = Layouter::new(&book, &styles, &fonts);
    let id = book.spreads[1].id.clone();
    let spread = layouter.layout_spread(&id).unwrap().unwrap();
    spread
        .frames
        .into_iter()
        .find(|f| f.frame.0 == "tx")
        .expect("the frame is laid out")
}

fn look() -> ParaLook {
    ParaLook::default()
}

/// The tops of the first line of each of the three paragraphs "a\nb\nc".
fn tops(looks: Vec<ParaLook>) -> Vec<f64> {
    let laid = lay("a\nb\nc", looks);
    (0..3)
        .map(|p| {
            laid.lines
                .iter()
                .find(|l| l.paragraph == Some(p))
                .expect("a line")
                .top
        })
        .collect()
}

#[test]
fn space_after_one_paragraph_moves_the_next_down_by_that_much() {
    let plain = tops(vec![]);
    let spaced = tops(vec![
        ParaLook {
            space_after: Some(10.0),
            ..look()
        },
        look(),
    ]);
    assert_eq!(spaced[0], plain[0]);
    assert!((spaced[1] - plain[1] - 10.0).abs() < 1e-6);
    // Only the paragraph that set it: the third keeps the gap it had after the second.
    assert!((spaced[2] - spaced[1] - (plain[2] - plain[1])).abs() < 1e-6);
}

#[test]
fn space_before_and_space_after_add_together() {
    let plain = tops(vec![]);
    let both = tops(vec![
        ParaLook {
            space_after: Some(10.0),
            ..look()
        },
        ParaLook {
            space_before: Some(6.0),
            ..look()
        },
    ]);
    assert!((both[1] - plain[1] - 16.0).abs() < 1e-6, "{both:?}");
}

#[test]
fn space_before_the_first_paragraph_of_a_column_is_left_off() {
    let plain = tops(vec![]);
    let first = tops(vec![ParaLook {
        space_before: Some(30.0),
        ..look()
    }]);
    assert_eq!(first[0], plain[0]);
    assert_eq!(first[1], plain[1]);
}

#[test]
fn line_spacing_of_one_paragraph_makes_its_lines_taller() {
    let plain = lay("a", vec![]).lines[0].height;
    let loose = lay(
        "a",
        vec![ParaLook {
            line_height: Some(2.0),
            ..look()
        }],
    )
    .lines[0]
        .height;
    assert!(loose > plain * 1.3, "{plain} {loose}");
}

#[test]
fn indents_move_the_lines_in_and_alignment_is_per_paragraph() {
    let laid = lay(
        "aaa\nbbb\nccc",
        vec![
            ParaLook {
                indent: Some(40.0),
                ..look()
            },
            ParaLook {
                align: Some(Align::Right),
                indent_right: Some(10.0),
                ..look()
            },
            look(),
        ],
    );
    let line = |p| laid.lines.iter().find(|l| l.paragraph == Some(p)).unwrap();
    let plain = line(2).x;
    assert!((line(0).x - plain - 40.0).abs() < 1e-6);
    // Right-aligned inside the box less its right indent, so the line ends at that edge.
    let end = line(1).x + line(1).width;
    assert!(end > line(2).x + line(2).width + 20.0, "{end}");
    assert!(end <= WIDTH, "{end}");
}

#[test]
fn a_first_line_indent_is_this_paragraphs_alone() {
    let laid = lay(
        "aaa\nbbb",
        vec![ParaLook {
            first_indent: Some(24.0),
            ..look()
        }],
    );
    let line = |p| laid.lines.iter().find(|l| l.paragraph == Some(p)).unwrap();
    assert!((line(0).x - line(1).x - 24.0).abs() < 1e-6);
}

#[test]
fn a_flows_paragraphs_take_their_own_looks() {
    use folio_model::{Flow, FlowId, FlowSlot, StoryPara};
    let made = new_book(&NewBookSpec {
        kind: BookKind::Poetry,
        words: Words::Samples,
        ..NewBookSpec::default()
    })
    .unwrap();
    let mut book: Book = made.book;
    let styles = StyleSheet::from_styles(&book.styles);
    let frame = FrameId("fx".into());
    book.spreads[1].frames.push(Frame::Text(TextFrame {
        base: FrameBase {
            id: frame.clone(),
            rect: Rect {
                x: Pt(20.0),
                y: Pt(20.0),
                w: Pt(WIDTH),
                h: Pt(600.0),
            },
            rotation: 0.0,
            locked: false,
            from_master: None,
            description: None,
            decorative: false,
        },
        style: StyleId("body".into()),
        content: None,
        flow: Some(FlowSlot {
            id: FlowId("fl_t".into()),
            index: 0,
        }),
        columns: 1,
        baseline_lock: false,
        overrides: None,
    }));
    let mut flow = Flow::from_text(FlowId("fl_t".into()), "a\nb\nc", vec![frame.clone()]);
    let gap = |book: &Book, styles: &StyleSheet| {
        let fonts = FontBook::bundled();
        let id = book.spreads[1].id.clone();
        let laid = Layouter::new(book, styles, &fonts)
            .layout_spread(&id)
            .unwrap()
            .unwrap();
        let f = laid
            .frames
            .into_iter()
            .find(|f| f.frame == FrameId("fx".into()))
            .unwrap();
        let top = |p| f.lines.iter().find(|l| l.paragraph == Some(p)).unwrap().top;
        (top(1) - top(0), top(2) - top(1))
    };
    book.flows.push(flow.clone());
    let (plain_a, plain_b) = gap(&book, &styles);
    flow.paragraphs[0] = StoryPara {
        look: Some(ParaLook {
            space_after: Some(14.0),
            ..look()
        }),
        ..flow.paragraphs[0].clone()
    };
    book.flows[0] = flow;
    let (a, b) = gap(&book, &styles);
    assert!((a - plain_a - 14.0).abs() < 1e-6);
    assert!((b - plain_b).abs() < 1e-6);
}
