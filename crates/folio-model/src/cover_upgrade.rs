// Moves the cover of a book saved before schema 3 onto the printed sheet
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Before schema 3 the cover spread was the front cover alone, and the back cover was made at export
//! from the book's description and its `backNotes`. From schema 3 the cover spread is the whole
//! sheet, so the front is shifted along past the back and the spine, and the description and the
//! notes become frames of the cover like any others.

use crate::{
    front_offset, Book, Colour, Fit, Frame, FrameBase, FrameId, ImageFrame, ImageRef,
    InlineContent, Pt, Rect, Run, ShapeFrame, SpreadKind, StyleId, StyleOverrides, TextFrame,
    SCHEMA_VERSION,
};

/// The id of the frame that carries the description on the back cover.
pub const BLURB_FRAME: &str = "blurb";

fn base(id: &str, rect: Rect, rotation: f64) -> FrameBase {
    FrameBase {
        id: FrameId(id.to_string()),
        rect,
        rotation,
        locked: false,
        from_master: None,
        description: None,
    }
}

/// A text frame holding `text` in the body style.
pub fn words_frame(id: &str, rect: Rect, rotation: f64, text: &str) -> Frame {
    Frame::Text(TextFrame {
        base: base(id, rect, rotation),
        style: StyleId("body".to_string()),
        content: Some(InlineContent {
            inline: vec![Run {
                text: text.to_string(),
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

/// Where the description goes on the back cover, from the back's top-left.
pub fn blurb_rect(book: &Book) -> Rect {
    let margin = Pt::from_in(0.5).0;
    Rect {
        x: Pt(margin),
        y: Pt(margin * 1.5),
        w: Pt(book.trim.w.0 - 2.0 * margin),
        h: Pt(book.trim.h.0 - margin * 4.0),
    }
}

/// Brings the cover of a book older than schema 3 onto the sheet. `paper` is the paper its settings
/// name, which decides how thick the spine is for a book made before the paper was part of it.
pub fn upgrade_cover(book: &mut Book, paper: Option<crate::Paper>) {
    if book.schema >= SCHEMA_VERSION {
        return;
    }
    if let Some(paper) = paper {
        if book.paper == crate::Paper::default() {
            book.paper = paper;
        }
    }
    let offset = front_offset(book).0;
    let blurb = book.metadata.description.trim().to_string();
    let rect = blurb_rect(book);
    let notes = std::mem::take(&mut book.back_notes);
    if let Some(cover) = book
        .spreads
        .first_mut()
        .filter(|s| s.kind == SpreadKind::Cover)
    {
        for frame in &mut cover.frames {
            frame.base_mut().rect.x.0 += offset;
        }
        if !blurb.is_empty() {
            cover
                .frames
                .push(words_frame(BLURB_FRAME, rect, 0.0, &blurb));
        }
        for note in notes {
            let id = format!("note_{}", note.id);
            let turn = note.rotation.unwrap_or(0.0);
            if let Some(shape) = note.shape {
                cover.frames.push(Frame::Shape(ShapeFrame {
                    base: base(&id, note.rect, turn),
                    shape,
                    fill: note.fill.or(Some(Colour("#ffffff".into()))),
                    style: None,
                    stroke: None,
                    stroke_width: None,
                    radius: None,
                    flip: false,
                    content: None,
                    overrides: None,
                }));
            } else if let Some(asset) = note.picture {
                cover.frames.push(Frame::Image(ImageFrame {
                    base: base(&id, note.rect, turn),
                    image: ImageRef {
                        asset: Some(asset),
                        fit: Fit::Fit,
                        focus: (0.5, 0.5),
                        crop: None,
                    },
                    placeholder_label: None,
                }));
            } else {
                let mut frame = words_frame(&id, note.rect, turn, &note.text);
                if let Frame::Text(f) = &mut frame {
                    f.overrides = Some(StyleOverrides {
                        size: Some(Pt(note.size)),
                        align: None,
                        bold: None,
                        italic: None,
                        font: None,
                    });
                }
                cover.frames.push(frame);
            }
        }
    }
    book.schema = SCHEMA_VERSION;
}

/// Keeps the back cover's description frame in step with the book's description while the words in
/// it are still the last description put there. Once the person has changed the words in the frame
/// they are theirs, and a new description leaves them alone. A description typed where there was
/// none, on a cover with no such frame, makes the frame.
pub fn sync_blurb(book: &mut Book, was: &str, now: &str) {
    let (was, now) = (was.trim(), now.trim());
    if was == now {
        return;
    }
    let rect = blurb_rect(book);
    let Some(cover) = book
        .spreads
        .first_mut()
        .filter(|s| s.kind == SpreadKind::Cover)
    else {
        return;
    };
    let found = cover.frames.iter_mut().find_map(|f| match f {
        Frame::Text(t) if t.base.id.0 == BLURB_FRAME => Some(t),
        _ => None,
    });
    match found {
        Some(frame) => {
            let held: String = frame
                .content
                .as_ref()
                .map(|c| c.inline.iter().map(|r| r.text.as_str()).collect())
                .unwrap_or_default();
            if held.trim() == was {
                if let Frame::Text(fresh) = words_frame(BLURB_FRAME, rect, 0.0, now) {
                    frame.content = fresh.content;
                }
            }
        }
        None if was.is_empty() => cover.frames.push(words_frame(BLURB_FRAME, rect, 0.0, now)),
        None => {}
    }
}
