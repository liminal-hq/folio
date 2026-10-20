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
    paperback_cover, Align, Background, BarcodeChoice, Book, Colour, Frame, FrameBase, FrameId,
    InlineContent, Page, PageId, Paper, Pt, Rect, Run, Shape, ShapeFrame, Size, Spread, SpreadId,
    SpreadKind, StyleId, StyleOverrides, TextFrame,
};

pub use folio_model::{spine_pages, SPINE_TEXT_FROM};

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

/// A bar of the barcode as a black rectangle frame.
fn bar(name: &str, rect: Rect) -> Frame {
    Frame::Shape(ShapeFrame {
        base: base(name, rect, 0.0),
        shape: Shape::Rect,
        fill: Some(Colour("#000000".into())),
        style: None,
        stroke: None,
        stroke_width: None,
        radius: None,
        flip: false,
        content: None,
        overrides: None,
    })
}

/// A line of small centred words in the body face.
fn small_text(name: &str, rect: Rect, text: &str) -> Frame {
    let mut frame = text_frame(name, "body", rect, 0.0, text);
    if let Frame::Text(f) = &mut frame {
        f.overrides = Some(StyleOverrides {
            size: Some(Pt(7.0)),
            align: Some(Align::Center),
            bold: None,
            italic: None,
            font: None,
        });
    }
    frame
}

/// The frames that draw an EAN-13 for `isbn` inside the barcode box: the bars at the standard width
/// of a module, the ISBN above them and its digits below, all on the white of the box, and the five
/// digit price `addon` beside them when there is one. Nothing when the ISBN is not a good one, which
/// leaves KDP to add its own.
fn barcode_frames(isbn: &str, addon: Option<&str>, maths: &folio_model::CoverMaths) -> Vec<Frame> {
    let Some(modules) = crate::barcode::ean13(isbn) else {
        return Vec::new();
    };
    let addon_modules = addon.and_then(crate::barcode::ean5);
    // The nominal module is 0.33 mm.
    let module = 0.33 / 25.4 * 72.0;
    let box_ = Rect {
        x: Pt(maths.barcode.x.0 - maths.edge.0),
        y: Pt(maths.barcode.y.0 - maths.edge.0),
        w: maths.barcode.w,
        h: maths.barcode.h,
    };
    // The add-on stands nine modules clear of the barcode; together they are centred in the box.
    let across = module
        * if addon_modules.is_some() {
            95.0 + 9.0 + 47.0
        } else {
            95.0
        };
    let left = box_.x.0 + (box_.w.0 - across) / 2.0;
    let (label, bars_top, bars_height) = (10.0, box_.y.0 + 14.0, 50.0);
    let mut frames: Vec<Frame> = crate::barcode::bars(&modules)
        .into_iter()
        .map(|(first, run)| {
            bar(
                &format!("bar_{first}"),
                Rect {
                    x: Pt(left + first as f64 * module),
                    y: Pt(bars_top),
                    w: Pt(run as f64 * module),
                    h: Pt(bars_height),
                },
            )
        })
        .collect();
    let digits: String = isbn.chars().filter(|c| c.is_ascii_digit()).collect();
    let main_width = Pt(95.0 * module);
    for (name, text, y) in [
        ("isbn_label", format!("ISBN {isbn}"), box_.y.0 + 2.0),
        ("isbn_digits", digits, bars_top + bars_height + 2.0),
    ] {
        frames.push(small_text(
            name,
            Rect {
                x: Pt(left),
                y: Pt(y),
                w: main_width,
                h: Pt(label + 2.0),
            },
            &text,
        ));
    }
    if let (Some(add), Some(digits)) = (addon_modules, addon) {
        let add_left = left + (95.0 + 9.0) * module;
        // The add-on's bars are shorter, and its digits stand over them.
        for (first, run) in crate::barcode::bars(&add) {
            frames.push(bar(
                &format!("addon_{first}"),
                Rect {
                    x: Pt(add_left + first as f64 * module),
                    y: Pt(bars_top + 8.0),
                    w: Pt(run as f64 * module),
                    h: Pt(bars_height - 8.0),
                },
            ));
        }
        frames.push(small_text(
            "addon_digits",
            Rect {
                x: Pt(add_left),
                y: Pt(bars_top - 2.0),
                w: Pt(47.0 * module),
                h: Pt(label),
            },
            digits,
        ));
    }
    frames
}

/// Points inside the trimmed edge of a cover that text and pictures that need to stay keep to.
const SAFE: f64 = 18.0;
/// The same beside each fold of the spine.
const SPINE_SAFE: f64 = 4.5;

/// The lines the sheet canvas snaps to: the trimmed edges and folds, and the safe areas inside them.
fn fold_guides(w: f64, h: f64, spine: f64) -> Vec<folio_model::Guide> {
    use folio_model::{Axis, Guide};
    let mut xs = vec![
        0.0,
        w,
        w + spine,
        2.0 * w + spine,
        SAFE,
        w - SAFE,
        w + spine + SAFE,
        2.0 * w + spine - SAFE,
    ];
    if spine > 2.0 * SPINE_SAFE {
        xs.push(w + SPINE_SAFE);
        xs.push(w + spine - SPINE_SAFE);
    }
    let mut guides: Vec<Guide> = xs
        .into_iter()
        .map(|x| Guide {
            axis: Axis::X,
            pos: Pt(x),
            locked: true,
        })
        .collect();
    guides.extend([0.0, SAFE, h - SAFE, h].map(|y| Guide {
        axis: Axis::Y,
        pos: Pt(y),
        locked: true,
    }));
    guides
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
    // The book's own barcode, drawn in the box when its paperback ISBN is a good one and the book
    // does not leave it to KDP.
    if let Some(isbn) = book.metadata.isbn.paperback.as_deref() {
        let addon = match &book.barcode {
            BarcodeChoice::Kdp => None,
            BarcodeChoice::Auto => Some(None),
            BarcodeChoice::Own { addon } => Some(addon.as_deref()),
        };
        if let Some(addon) = addon {
            frames.extend(barcode_frames(isbn, addon, &maths));
        }
    }
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

    // What the person put on the sheet, from the back cover's top-left: words, pictures and shapes
    // that may sit on the back, across the spine or on the front, over everything else.
    for note in &book.back_notes {
        if let Some(shape) = note.shape {
            frames.push(Frame::Shape(ShapeFrame {
                base: base(
                    &format!("note_{}", note.id),
                    note.rect,
                    note.rotation.unwrap_or(0.0),
                ),
                shape,
                fill: note.fill.clone().or(Some(Colour("#ffffff".into()))),
                style: None,
                stroke: None,
                stroke_width: None,
                radius: None,
                flip: false,
                content: None,
                overrides: None,
            }));
            continue;
        }
        if let Some(asset) = &note.picture {
            frames.push(Frame::Image(folio_model::ImageFrame {
                base: base(
                    &format!("note_{}", note.id),
                    note.rect,
                    note.rotation.unwrap_or(0.0),
                ),
                image: folio_model::ImageRef {
                    asset: Some(asset.clone()),
                    fit: folio_model::Fit::Fit,
                    focus: (0.5, 0.5),
                    crop: None,
                },
                placeholder_label: None,
            }));
            continue;
        }
        let mut frame = text_frame(
            &format!("note_{}", note.id),
            "body",
            note.rect,
            note.rotation.unwrap_or(0.0),
            &note.text,
        );
        if let Frame::Text(f) = &mut frame {
            f.overrides = Some(StyleOverrides {
                size: Some(Pt(note.size)),
                align: None,
                bold: None,
                italic: None,
                font: None,
            });
        }
        frames.push(frame);
    }

    // What Folio draws itself (the blurb, the barcode, the spine) is not the person's to move on the
    // canvas: its words are Book details and its place is the maths of the cover.
    for frame in &mut frames {
        let base = match frame {
            Frame::Text(f) => &mut f.base,
            Frame::Shape(f) => &mut f.base,
            Frame::Image(f) => &mut f.base,
            _ => continue,
        };
        if base.id.0.starts_with("cover_") && !base.id.0.starts_with("cover_note_") {
            base.locked = true;
        }
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
        guides: fold_guides(w, h, spine),
        master_overrides: Vec::new(),
        bg_span: None,
        section: None,
    }];
    Some(sheet)
}
