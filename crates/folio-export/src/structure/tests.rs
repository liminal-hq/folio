// Tests for the structure plan, on page scenes made by hand
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use super::*;
use folio_layout::scene::{SceneItem, SceneRect};
use folio_model::{
    BookKind, Fit, FlowId, FlowSlot, FrameBase, FrameId, ImageFrame, ImageRef, InlineContent,
    MasterId, PageId, Pt, Rect, Run, ShapeFrame, SpreadKind, StyleId, TableCell, TableFrame,
    TableLook, TextFrame,
};
use folio_presets::{new_book, NewBookSpec, Words};

fn base(id: &str, x: f64, y: f64, w: f64, h: f64) -> FrameBase {
    FrameBase {
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
        decorative: false,
    }
}

fn words(id: &str, style: &str, x: f64, y: f64, w: f64, h: f64, text: &str) -> Frame {
    Frame::Text(TextFrame {
        base: base(id, x, y, w, h),
        style: StyleId(style.into()),
        content: Some(InlineContent::of(vec![Run {
            text: text.into(),
            ..Default::default()
        }])),
        flow: None,
        columns: 1,
        baseline_lock: false,
        overrides: None,
    })
}

fn picture(id: &str, x: f64, y: f64, w: f64, h: f64, alt: Option<&str>) -> Frame {
    let mut base = base(id, x, y, w, h);
    base.description = alt.map(str::to_owned);
    Frame::Image(ImageFrame {
        base,
        image: ImageRef {
            asset: Some("a.png".into()),
            fit: Fit::Fill,
            focus: (0.5, 0.5),
            crop: None,
        },
        placeholder_label: None,
    })
}

/// A square book of two-page spreads `sp_0` … with `frames(i)` on spread `i`.
fn book_with(n: usize, frames: impl Fn(usize) -> Vec<Frame>) -> Book {
    let made = new_book(&NewBookSpec {
        kind: BookKind::Picture,
        words: Words::Samples,
        ..NewBookSpec::default()
    })
    .unwrap();
    let mut book = made.book;
    book.trim.w = Pt(612.0);
    book.trim.h = Pt(612.0);
    book.margins.top = Pt(36.0);
    book.margins.bottom = Pt(36.0);
    let template = book
        .spreads
        .iter()
        .find(|s| s.kind == SpreadKind::Spread)
        .unwrap()
        .clone();
    book.spreads.clear();
    for i in 0..n {
        let mut spread = template.clone();
        spread.id = folio_model::SpreadId(format!("sp_{i}"));
        for (p, page) in spread.pages.iter_mut().enumerate() {
            page.id = PageId(format!("pg_{i}_{p}"));
            page.master = None;
        }
        spread.frames = frames(i);
        book.spreads.push(spread);
    }
    book
}

fn run(text: &str, x: f64, width: f64) -> LayoutRun {
    LayoutRun {
        text: text.into(),
        x,
        width,
        family: "Serif".into(),
        size: 12.0,
        weight: 400,
        italic: false,
        tracking: 0.0,
        colour: None,
        underline: None,
        strike: None,
        lang: None,
        href: None,
        link: None,
        hyphen: false,
        miss: Vec::new(),
        start: 0,
        stops: Vec::new(),
        face_key: String::new(),
        glyphs: Vec::new(),
    }
}

fn line(paragraph: u32, runs: Vec<LayoutRun>) -> LayoutLine {
    LayoutLine {
        x: 0.0,
        top: 0.0,
        baseline: 10.0,
        width: 100.0,
        height: 12.0,
        runs,
        paragraph: Some(paragraph),
        start: 0,
        end: 0,
    }
}

fn say(frame: &str, paragraph: u32, text: &str) -> SceneItem {
    let mut x = 0.0;
    let mut runs = Vec::new();
    for word in text.split(' ') {
        runs.push(run(word, x, 30.0));
        x += 36.0;
    }
    SceneItem {
        frame: FrameId(frame.into()),
        clip: None,
        kind: SceneKind::Text {
            at: (0.0, 0.0),
            size: (100.0, 100.0),
            rotation: 0.0,
            lines: vec![line(paragraph, runs)],
        },
    }
}

fn pic(frame: &str, asset: Option<&str>) -> SceneItem {
    SceneItem {
        frame: FrameId(frame.into()),
        clip: None,
        kind: SceneKind::Picture {
            rect: SceneRect {
                x: 0.0,
                y: 0.0,
                w: 10.0,
                h: 10.0,
            },
            rotation: 0.0,
            asset: asset.map(str::to_owned),
            fit: Fit::Fill,
            focus: (0.5, 0.5),
            crop: None,
        },
    }
}

fn page(book: &Book, spread: usize, index: usize, items: Vec<SceneItem>) -> PageScene {
    PageScene {
        page: book.spreads[spread].pages[index].id.clone(),
        index,
        width: 612.0,
        height: 612.0,
        recto: index == 1,
        items,
    }
}

fn kinds(nodes: &[Node]) -> Vec<String> {
    nodes
        .iter()
        .map(|n| match n {
            Node::Leaf(_) => "leaf".to_string(),
            Node::Group(g) => match &g.kind {
                Kind::Heading { level, title } => format!("H{level}({title})"),
                Kind::Figure { alt } => format!("Figure({})", alt.clone().unwrap_or_default()),
                other => format!("{other:?}"),
            },
        })
        .collect()
}

/// Every mark is in the tree exactly once, which the PDF library insists on.
fn assert_each_mark_once(plan: &Plan) {
    let leaves = plan.leaves_in_order();
    let unique: BTreeSet<usize> = leaves.iter().map(|l| l.0).collect();
    assert_eq!(unique.len(), leaves.len(), "a mark is in the tree twice");
    assert_eq!(leaves.len(), plan.sites.len(), "a mark is not in the tree");
}

fn item_plan(plan: &Plan, sheet: usize, member: usize, item: usize) -> &ItemPlan {
    plan.items
        .get(&ItemAt {
            sheet,
            member,
            item,
        })
        .expect("planned")
}

#[test]
fn a_page_is_read_in_rows_from_the_top_and_left_to_right() {
    let book = book_with(1, |_| {
        vec![
            words("low", "body", 66.0, 400.0, 480.0, 100.0, "Last"),
            words("right", "body", 320.0, 100.0, 230.0, 100.0, "Second"),
            words("left", "body", 66.0, 105.0, 230.0, 100.0, "First"),
            words("top", "display", 66.0, 20.0, 480.0, 40.0, "Title here"),
        ]
    });
    let scene = page(
        &book,
        0,
        0,
        vec![
            say("low", 0, "Last"),
            say("right", 0, "Second"),
            say("left", 0, "First"),
            say("top", 0, "Title here"),
        ],
    );
    let p = plan(&book, &[vec![&scene]]);
    assert_eq!(
        kinds(&p.tree),
        vec!["H1(Title here)", "Paragraph", "Paragraph", "Paragraph"]
    );
    // First, then Second, then Last: the leaves of the paragraphs in order.
    let order: Vec<usize> = p
        .leaves_in_order()
        .iter()
        .map(|l| p.sites[l.0].at.item)
        .collect();
    assert_eq!(order, vec![3, 2, 1, 0]);
    assert_each_mark_once(&p);
}

#[test]
fn page_numbers_and_the_pumpkins_behind_them_are_artifacts_and_the_story_is_not() {
    let book = book_with(3, |i| {
        vec![
            picture(
                &format!("art{i}"),
                36.0,
                30.0,
                540.0,
                485.0,
                Some("A kitchen"),
            ),
            picture(&format!("pk{i}"), 276.0, 526.0, 60.0, 48.0, None),
            words(
                &format!("pn{i}"),
                "body",
                276.0,
                541.0,
                60.0,
                22.0,
                &format!("{i}"),
            ),
            words(
                &format!("tx{i}"),
                "body",
                678.0,
                36.0,
                456.0,
                480.0,
                "Once upon a time",
            ),
        ]
    });
    let left = page(
        &book,
        1,
        0,
        vec![
            pic("art1", Some("a.png")),
            pic("pk1", Some("a.png")),
            say("pn1", 0, "1"),
        ],
    );
    let right = page(&book, 1, 1, vec![say("tx1", 0, "Once upon a time")]);
    let p = plan(&book, &[vec![&left], vec![&right]]);
    assert_eq!(kinds(&p.tree), vec!["Figure(A kitchen)", "Paragraph"]);
    assert_eq!(
        item_plan(&p, 0, 0, 1),
        &ItemPlan::Artifact(ArtifactKind::Layout)
    );
    match item_plan(&p, 0, 0, 2) {
        ItemPlan::Text(lines) => {
            assert_eq!(lines, &vec![LinePlan::Artifact(ArtifactKind::PageNumber)])
        }
        other => panic!("{other:?}"),
    }
    assert_each_mark_once(&p);
}

#[test]
fn a_frame_marked_decorative_and_an_empty_picture_are_artifacts_but_an_unmarked_one_is_a_figure() {
    let mut book = book_with(1, |_| {
        vec![
            picture("flourish", 10.0, 10.0, 40.0, 40.0, None),
            picture("empty", 100.0, 10.0, 40.0, 40.0, None),
            picture("plain", 200.0, 10.0, 40.0, 40.0, None),
        ]
    });
    book.spreads[0].frames[0].base_mut().decorative = true;
    let scene = page(
        &book,
        0,
        0,
        vec![
            pic("flourish", Some("a.png")),
            pic("empty", None),
            pic("plain", Some("a.png")),
        ],
    );
    let p = plan(&book, &[vec![&scene]]);
    assert_eq!(
        item_plan(&p, 0, 0, 0),
        &ItemPlan::Artifact(ArtifactKind::Layout)
    );
    assert_eq!(
        item_plan(&p, 0, 0, 1),
        &ItemPlan::Artifact(ArtifactKind::Layout)
    );
    // An empty description is not decoration: it is a figure that preflight asks to describe.
    assert!(matches!(item_plan(&p, 0, 0, 2), ItemPlan::Figure(_)));
    assert_eq!(kinds(&p.tree), vec!["Figure()"]);
    assert_each_mark_once(&p);
}

#[test]
fn backgrounds_and_everything_from_a_master_are_artifacts() {
    let mut book = book_with(1, |_| {
        vec![words("tx", "body", 66.0, 36.0, 480.0, 400.0, "Story")]
    });
    let master = folio_model::Master {
        id: MasterId("ma".into()),
        name: "A".into(),
        frames: vec![
            words("fr_pn_a", "folio", 276.0, 541.0, 60.0, 22.0, "{page}"),
            words("fr_rh", "rh", 36.0, 10.0, 200.0, 18.0, "Head"),
            words("fr_x", "body", 36.0, 300.0, 100.0, 18.0, "Furniture"),
        ],
        guides: Vec::new(),
        bg: None,
    };
    book.master_pages = vec![master];
    book.spreads[0].pages[0].master = Some(MasterId("ma".into()));
    let pg = book.spreads[0].pages[0].id.0.clone();
    let scene = page(
        &book,
        0,
        0,
        vec![
            SceneItem {
                frame: FrameId(format!("bg@{pg}")),
                clip: None,
                kind: SceneKind::Background {
                    bg: folio_model::Background::Colour {
                        colour: folio_model::Colour("#ffffff".into()),
                    },
                    across: None,
                },
            },
            say(&format!("fr_pn_a@{pg}"), 0, "7"),
            say(&format!("fr_rh@{pg}"), 0, "Head"),
            say(&format!("fr_x@{pg}"), 0, "Furniture"),
            say("tx", 0, "Story"),
        ],
    );
    let p = plan(&book, &[vec![&scene]]);
    assert_eq!(
        item_plan(&p, 0, 0, 0),
        &ItemPlan::Artifact(ArtifactKind::Background)
    );
    let artifact = |item: usize| match item_plan(&p, 0, 0, item) {
        ItemPlan::Text(lines) => match &lines[0] {
            LinePlan::Artifact(kind) => *kind,
            other => panic!("{other:?}"),
        },
        other => panic!("{other:?}"),
    };
    assert_eq!(artifact(1), ArtifactKind::PageNumber);
    assert_eq!(artifact(2), ArtifactKind::Header);
    assert_eq!(artifact(3), ArtifactKind::Layout);
    assert_eq!(kinds(&p.tree), vec!["Paragraph"]);
    assert_each_mark_once(&p);
}

#[test]
fn a_frame_across_the_gutter_is_read_once_on_the_page_that_holds_its_middle() {
    // A picture 500 points wide from x = 400: its middle (650) is on the right page.
    let book = book_with(1, |_| {
        vec![picture(
            "wide",
            400.0,
            30.0,
            500.0,
            300.0,
            Some("A long beach"),
        )]
    });
    let left = page(&book, 0, 0, vec![pic("wide", Some("a.png"))]);
    let right = page(&book, 0, 1, vec![pic("wide", Some("a.png"))]);
    // Two pages on their own sheets, then both on one sheet.
    for sheets in [vec![vec![&left], vec![&right]], vec![vec![&left, &right]]] {
        let p = plan(&book, &sheets);
        assert_eq!(kinds(&p.tree), vec!["Figure(A long beach)"]);
        let (left_sheet, left_member) = (0, 0);
        assert_eq!(
            item_plan(&p, left_sheet, left_member, 0),
            &ItemPlan::Artifact(ArtifactKind::Layout)
        );
        assert!(matches!(
            item_plan(&p, sheets.len() - 1, sheets.last().unwrap().len() - 1, 0),
            ItemPlan::Figure(_)
        ));
        assert_each_mark_once(&p);
    }
    // With only the left page in the file, that page keeps the picture as content.
    let p = plan(&book, &[vec![&left]]);
    assert!(matches!(item_plan(&p, 0, 0, 0), ItemPlan::Figure(_)));
}

#[test]
fn a_paragraph_that_runs_over_frames_and_pages_is_one_element_in_the_order_of_its_frames() {
    let mut book = book_with(1, |_| {
        let mut b = words("second", "body", 66.0, 36.0, 480.0, 200.0, "");
        let mut a = words("first", "body", 66.0, 300.0, 480.0, 200.0, "");
        if let Frame::Text(t) = &mut a {
            t.flow = Some(FlowSlot {
                id: FlowId("fl".into()),
                index: 0,
            });
        }
        if let Frame::Text(t) = &mut b {
            t.flow = Some(FlowSlot {
                id: FlowId("fl".into()),
                index: 1,
            });
        }
        vec![a, b]
    });
    book.flows.push(folio_model::Flow::from_text(
        FlowId("fl".into()),
        "One\nTwo",
        vec![FrameId("first".into()), FrameId("second".into())],
    ));
    // The story's frames are on one page: slot 1 is higher up, but the story is read in slot order.
    let scene = page(
        &book,
        0,
        0,
        vec![
            say("first", 0, "start of one"),
            say("second", 0, "end of one"),
        ],
    );
    let p = plan(&book, &[vec![&scene]]);
    // One paragraph (key (flow, 0)), both frames' lines in it.
    assert_eq!(kinds(&p.tree), vec!["Paragraph"]);
    let items: Vec<usize> = p
        .leaves_in_order()
        .iter()
        .map(|l| p.sites[l.0].at.item)
        .collect();
    assert_eq!(items[0], 0);
    // A line is one piece when its words are alike.
    assert_eq!(items.len(), 2);
    assert_each_mark_once(&p);
}

#[test]
fn a_paragraph_continues_on_the_next_page_and_a_caption_after_a_figure_is_a_caption() {
    let mut book = book_with(2, |i| {
        vec![
            picture(
                &format!("pic{i}"),
                66.0,
                20.0,
                480.0,
                300.0,
                Some("An otter"),
            ),
            words(
                &format!("cap{i}"),
                "caption",
                66.0,
                330.0,
                480.0,
                40.0,
                "A caption",
            ),
            words(&format!("tx{i}"), "body", 66.0, 400.0, 480.0, 100.0, ""),
        ]
    });
    for i in 0..2 {
        if let Frame::Text(t) = &mut book.spreads[i].frames[2] {
            t.flow = Some(FlowSlot {
                id: FlowId("fl".into()),
                index: i as u32,
            });
        }
    }
    book.flows.push(folio_model::Flow::from_text(
        FlowId("fl".into()),
        "One long paragraph",
        vec![FrameId("tx0".into()), FrameId("tx1".into())],
    ));
    let a = page(
        &book,
        0,
        0,
        vec![
            pic("pic0", Some("a.png")),
            say("cap0", 0, "A caption"),
            say("tx0", 0, "One long"),
        ],
    );
    let b = page(&book, 1, 0, vec![say("tx1", 0, "paragraph")]);
    let p = plan(&book, &[vec![&a], vec![&b]]);
    assert_eq!(
        kinds(&p.tree),
        vec!["Figure(An otter)", "Caption", "Paragraph"]
    );
    // The paragraph holds the leaves from both pages.
    let Node::Group(paragraph) = &p.tree[2] else {
        panic!()
    };
    let sheets: Vec<usize> = paragraph
        .children
        .iter()
        .map(|n| match n {
            Node::Leaf(l) => p.sites[l.0].at.sheet,
            _ => panic!(),
        })
        .collect();
    assert_eq!(sheets, vec![0, 1]);
    assert_each_mark_once(&p);
}

#[test]
fn a_title_in_a_few_paragraphs_is_one_heading_with_its_words_and_other_styles_have_their_roles() {
    let book = book_with(1, |_| {
        vec![
            words(
                "title",
                "display",
                66.0,
                20.0,
                480.0,
                100.0,
                "A Cat\nand Mouse",
            ),
            words("sub", "numeral", 66.0, 130.0, 480.0, 40.0, "One"),
            words("quote", "epigraph", 66.0, 200.0, 480.0, 40.0, "Said"),
            words("note", "callout", 66.0, 260.0, 480.0, 40.0, "Note"),
            words("bub", "bubble", 66.0, 320.0, 480.0, 40.0, "Boo"),
            words("fol", "folio", 66.0, 560.0, 60.0, 20.0, "9"),
        ]
    });
    let two = SceneItem {
        frame: FrameId("title".into()),
        clip: None,
        kind: SceneKind::Text {
            at: (0.0, 0.0),
            size: (1.0, 1.0),
            rotation: 0.0,
            lines: vec![
                line(0, vec![run("A", 0.0, 8.0), run("Cat", 12.0, 20.0)]),
                line(1, vec![run("and", 0.0, 20.0), run("Mouse", 24.0, 30.0)]),
            ],
        },
    };
    let scene = page(
        &book,
        0,
        0,
        vec![
            two,
            say("sub", 0, "One"),
            say("quote", 0, "Said"),
            say("note", 0, "Note"),
            say("bub", 0, "Boo"),
            say("fol", 0, "9"),
        ],
    );
    let p = plan(&book, &[vec![&scene]]);
    assert_eq!(
        kinds(&p.tree),
        vec![
            "H1(A Cat and Mouse)",
            "H2(One)",
            "BlockQuote",
            "Note",
            "Paragraph"
        ]
    );
    match item_plan(&p, 0, 0, 5) {
        ItemPlan::Text(lines) => {
            assert_eq!(lines, &vec![LinePlan::Artifact(ArtifactKind::PageNumber)])
        }
        other => panic!("{other:?}"),
    }
    assert_each_mark_once(&p);
}

#[test]
fn words_that_link_are_a_link_element_across_the_lines_they_run_over() {
    let book = book_with(1, |_| {
        vec![words("tx", "body", 66.0, 36.0, 480.0, 100.0, "x")]
    });
    let mut linked = run("click", 40.0, 30.0);
    linked.href = Some("https://example.com".into());
    let mut linked2 = run("here", 0.0, 30.0);
    linked2.href = Some("https://example.com".into());
    let mut french = run("bonjour", 90.0, 30.0);
    french.lang = Some("fr-CA".into());
    let item = SceneItem {
        frame: FrameId("tx".into()),
        clip: None,
        kind: SceneKind::Text {
            at: (0.0, 0.0),
            size: (1.0, 1.0),
            rotation: 0.0,
            lines: vec![
                line(0, vec![run("Go", 0.0, 20.0), linked]),
                line(0, vec![linked2, run("now", 40.0, 20.0), french]),
            ],
        },
    };
    let scene = page(&book, 0, 0, vec![item]);
    let p = plan(&book, &[vec![&scene]]);
    let Node::Group(paragraph) = &p.tree[0] else {
        panic!()
    };
    let shape: Vec<String> = paragraph
        .children
        .iter()
        .map(|n| match n {
            Node::Leaf(_) => "text".to_string(),
            Node::Group(g) => match &g.kind {
                Kind::Link { href, .. } => format!("link {href} x{}", g.children.len()),
                other => format!("{other:?}"),
            },
        })
        .collect();
    // "Go", the link over "click" and "here" (the line break between them keeps one link), "now",
    // then a piece in French.
    assert_eq!(
        shape,
        vec!["text", "link https://example.com x2", "text", "text"]
    );
    match item_plan(&p, 0, 0, 0) {
        ItemPlan::Text(lines) => {
            let LinePlan::Pieces(first) = &lines[0] else {
                panic!()
            };
            assert_eq!(first.len(), 2);
            let LinePlan::Pieces(second) = &lines[1] else {
                panic!()
            };
            assert_eq!(second.len(), 3);
            assert_eq!(second[2].lang.as_deref(), Some("fr-CA"));
        }
        other => panic!("{other:?}"),
    }
    assert_each_mark_once(&p);
}

#[test]
fn a_table_has_head_rows_and_body_rows_of_cells() {
    let cell = |t: &str| TableCell {
        content: Some(InlineContent::of(vec![Run {
            text: t.into(),
            ..Default::default()
        }])),
    };
    let mut book = book_with(1, |_| Vec::new());
    book.spreads[0].frames.push(Frame::Table(TableFrame {
        base: base("tbl", 66.0, 36.0, 480.0, 200.0),
        cols: vec![0.5, 0.5],
        rows: vec![
            vec![cell("Name"), cell("Age")],
            vec![cell("Ann"), cell("7")],
        ],
        head: 1,
        look: TableLook::Lined,
        style: None,
        overrides: None,
    }));
    let table_art = SceneItem {
        frame: FrameId("tbl".into()),
        clip: None,
        kind: SceneKind::Table {
            rotation: 0.0,
            tints: Vec::new(),
            rules: Vec::new(),
        },
    };
    let scene = page(
        &book,
        0,
        0,
        vec![
            table_art,
            say("tbl#0.0", 0, "Name"),
            say("tbl#0.1", 0, "Age"),
            say("tbl#1.0", 0, "Ann"),
            say("tbl#1.1", 0, "7"),
        ],
    );
    let p = plan(&book, &[vec![&scene]]);
    assert_eq!(kinds(&p.tree), vec!["Table"]);
    let Node::Group(table) = &p.tree[0] else {
        panic!()
    };
    assert_eq!(kinds(&table.children), vec!["TableHead", "TableBody"]);
    let Node::Group(head) = &table.children[0] else {
        panic!()
    };
    let Node::Group(row) = &head.children[0] else {
        panic!()
    };
    assert_eq!(kinds(&row.children), vec!["HeaderCell", "HeaderCell"]);
    assert_eq!(
        item_plan(&p, 0, 0, 0),
        &ItemPlan::Artifact(ArtifactKind::Layout)
    );
    assert_each_mark_once(&p);
}

#[test]
fn a_shape_is_a_figure_only_when_it_is_described_and_its_words_are_read_after_it() {
    let mut book = book_with(1, |_| Vec::new());
    for (id, alt) in [("described", Some("A speech bubble")), ("bare", None)] {
        let mut b = base(
            id,
            66.0,
            if alt.is_some() { 36.0 } else { 300.0 },
            200.0,
            100.0,
        );
        b.description = alt.map(str::to_owned);
        book.spreads[0].frames.push(Frame::Shape(ShapeFrame {
            base: b,
            shape: folio_model::Shape::Speech,
            fill: None,
            style: None,
            stroke: None,
            stroke_width: None,
            radius: None,
            flip: false,
            content: None,
            overrides: None,
        }));
    }
    let art = |id: &str| SceneItem {
        frame: FrameId(id.into()),
        clip: None,
        kind: SceneKind::Shape {
            rect: SceneRect {
                x: 0.0,
                y: 0.0,
                w: 1.0,
                h: 1.0,
            },
            rotation: 0.0,
            shape: folio_model::Shape::Speech,
            fill: None,
            stroke: None,
            stroke_width: None,
            radius: None,
            flip: false,
            paths: Vec::new(),
        },
    };
    let scene = page(
        &book,
        0,
        0,
        vec![
            art("described"),
            say("described", 0, "Hello"),
            art("bare"),
            say("bare", 0, "Hi"),
        ],
    );
    let p = plan(&book, &[vec![&scene]]);
    assert_eq!(
        kinds(&p.tree),
        vec!["Figure(A speech bubble)", "Paragraph", "Paragraph"]
    );
    assert_eq!(
        item_plan(&p, 0, 0, 2),
        &ItemPlan::Artifact(ArtifactKind::Layout)
    );
    assert_each_mark_once(&p);
}
