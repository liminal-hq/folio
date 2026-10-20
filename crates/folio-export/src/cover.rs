// The paperback cover sheet: back, spine and front on one sheet, built from the book's cover
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! KDP wants the cover as one PDF: the bleed, the back cover, the spine, the front cover and the
//! bleed again, as wide as the pages make the spine. Folio keeps a cover as one page, so the sheet
//! is made here by laying that page on the right of a longer one: the front is the cover as it is,
//! the back carries the book's description and a clear box where the barcode goes, and the spine
//! carries the title and author when the book is thick enough for them. The sheet is a book of its
//! own with one page, so the same exporter draws it.

use folio_model::{
    paperback_cover, Background, Book, Colour, Frame, FrameBase, FrameId, InlineContent, Page,
    PageId, Paper, Pt, Rect, Run, Shape, ShapeFrame, Size, Spread, SpreadId, SpreadKind, StyleId,
    TextFrame,
};

/// KDP puts text on the spine of a book of this many pages or more. Its pages disagree between 79
/// and 80, so Folio takes the stricter (D-34).
pub const SPINE_TEXT_FROM: u32 = 80;

/// How many pages KDP counts for the spine: the interior, in twos, and at least 24.
pub fn spine_pages(book: &Book) -> u32 {
    let interior: u32 = book
        .spreads
        .iter()
        .skip(1)
        .map(|s| s.pages.len() as u32)
        .sum();
    (interior + interior % 2).max(24)
}

fn id(prefix: &str) -> FrameId {
    FrameId(format!("cover_{prefix}"))
}

fn base(name: &str, rect: Rect, rotation: f64) -> FrameBase {
    FrameBase {
        id: id(name),
        rect,
        rotation,
        locked: false,
        from_master: None,
        description: None,
    }
}

fn words(text: &str) -> InlineContent {
    InlineContent {
        inline: vec![Run {
            text: text.to_string(),
            style: None,
            bold: false,
            italic: false,
            font: None,
            colour: None,
        }],
    }
}

fn text_frame(name: &str, style: &str, rect: Rect, rotation: f64, text: &str) -> Frame {
    Frame::Text(TextFrame {
        base: base(name, rect, rotation),
        style: StyleId(style.to_string()),
        content: Some(words(text)),
        flow: None,
        columns: 1,
        baseline_lock: false,
        overrides: None,
    })
}

/// The cover sheet of `book` as a one-page book, or `None` when the book has no cover.
pub fn cover_sheet(book: &Book, paper: Paper) -> Option<Book> {
    let cover = book
        .spreads
        .first()
        .filter(|s| s.kind == SpreadKind::Cover)?;
    let pages = spine_pages(book);
    let maths = paperback_cover(book.trim, pages, paper);
    let (w, h, spine) = (book.trim.w.0, book.trim.h.0, maths.spine.0);
    let sheet_w = 2.0 * w + spine;
    let margin = Pt::from_in(0.5).0;

    // What fills the sheet: the cover's own background, which now runs across all three panels.
    let backgrounds = folio_model::resolve(&book.master_pages, cover);
    let bg: Option<Background> = backgrounds
        .into_iter()
        .next()
        .flatten()
        .map(|resolved| resolved.bg);

    let mut frames: Vec<Frame> = Vec::new();
    // The back: the book's description.
    if !book.metadata.description.trim().is_empty() {
        frames.push(text_frame(
            "blurb",
            "body",
            Rect {
                x: Pt(margin),
                y: Pt(margin * 1.5),
                w: Pt(w - 2.0 * margin),
                h: Pt(h - margin * 4.0),
            },
            0.0,
            book.metadata.description.trim(),
        ));
    }
    // The barcode area, kept clear: KDP adds the barcode there when the book has none of its own.
    frames.push(Frame::Shape(ShapeFrame {
        base: base(
            "barcode",
            Rect {
                x: Pt(maths.barcode.x.0 - maths.edge.0),
                y: Pt(maths.barcode.y.0 - maths.edge.0),
                w: maths.barcode.w,
                h: maths.barcode.h,
            },
            0.0,
        ),
        shape: Shape::Rect,
        fill: Some(Colour("#ffffff".into())),
        style: None,
        stroke: None,
        stroke_width: None,
        radius: None,
        flip: false,
        content: None,
        overrides: None,
    }));
    // The front: the cover as it is, moved along past the back and the spine.
    for frame in &cover.frames {
        let mut moved = frame.clone();
        match &mut moved {
            Frame::Text(f) => f.base.rect.x.0 += w + spine,
            Frame::Image(f) => f.base.rect.x.0 += w + spine,
            Frame::Shape(f) => f.base.rect.x.0 += w + spine,
            Frame::Table(f) => f.base.rect.x.0 += w + spine,
            _ => continue,
        }
        frames.push(moved);
    }
    // The spine: the title and author, turned to read down it, when the book is thick enough.
    if pages >= SPINE_TEXT_FROM {
        let author = book
            .authors
            .first()
            .map(|a| format!(" · {}", a.name))
            .unwrap_or_default();
        let length = h - 2.0 * Pt::from_in(1.0).0;
        let thickness = (spine - 2.0).clamp(4.0, 24.0);
        let (cx, cy) = (w + spine / 2.0, h / 2.0);
        frames.push(text_frame(
            "spine",
            "spine",
            Rect {
                x: Pt(cx - length / 2.0),
                y: Pt(cy - thickness / 2.0),
                w: Pt(length),
                h: Pt(thickness),
            },
            90.0,
            &format!("{}{author}", book.title),
        ));
    }

    let mut sheet = book.clone();
    sheet.trim = Size {
        w: Pt(sheet_w),
        h: Pt(h),
    };
    sheet.bleed = maths.edge;
    sheet.master_pages.clear();
    sheet.flows.clear();
    sheet.spreads = vec![Spread {
        id: SpreadId("sp_cover_sheet".into()),
        kind: SpreadKind::Single,
        pages: vec![Page {
            id: PageId("pg_cover_sheet".into()),
            master: None,
            bg,
        }],
        frames,
        guides: Vec::new(),
        master_overrides: Vec::new(),
        bg_span: None,
        section: None,
    }];
    Some(sheet)
}
