// Every place a book holds a colour, visited in one pass
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use crate::background::Background;
use crate::book::Book;
use crate::colour::{Colour, Swatches};
use crate::frame::{Frame, Run};
use crate::spread::Spread;

/// Calls `visit` on every colour in the book: styles, the words of frames, tables and flows, shape
/// fills and outlines, and every background (its colour, gradient stops and texture tint). Swatch
/// definitions are not visited: they are what the references point at.
pub fn visit_colours(book: &mut Book, visit: &mut dyn FnMut(&mut Colour)) {
    for style in book.styles.styles.values_mut() {
        if let Some(colour) = &mut style.colour {
            visit(colour);
        }
    }
    for spread in &mut book.spreads {
        visit_spread(spread, visit);
    }
    for flow in &mut book.flows {
        for para in &mut flow.paragraphs {
            visit_runs(&mut para.inline, visit);
        }
    }
    for master in &mut book.master_pages {
        visit_frames(&mut master.frames, visit);
        if let Some(bg) = &mut master.bg {
            visit_background(bg, visit);
        }
    }
}

/// Calls `visit` on every colour of one spread.
pub fn visit_spread(spread: &mut Spread, visit: &mut dyn FnMut(&mut Colour)) {
    visit_frames(&mut spread.frames, visit);
    if let Some(bg) = &mut spread.bg_span {
        visit_background(bg, visit);
    }
    for page in &mut spread.pages {
        if let Some(bg) = &mut page.bg {
            visit_background(bg, visit);
        }
    }
}

fn visit_runs(runs: &mut [Run], visit: &mut dyn FnMut(&mut Colour)) {
    for run in runs {
        if let Some(colour) = &mut run.colour {
            visit(colour);
        }
    }
}

fn visit_frames(frames: &mut [Frame], visit: &mut dyn FnMut(&mut Colour)) {
    for frame in frames {
        match frame {
            Frame::Text(t) => {
                if let Some(content) = &mut t.content {
                    visit_runs(&mut content.inline, visit);
                }
            }
            Frame::Shape(s) => {
                for colour in [&mut s.fill, &mut s.stroke].into_iter().flatten() {
                    visit(colour);
                }
                if let Some(content) = &mut s.content {
                    visit_runs(&mut content.inline, visit);
                }
            }
            Frame::Table(t) => {
                for cell in t.rows.iter_mut().flatten() {
                    if let Some(content) = &mut cell.content {
                        visit_runs(&mut content.inline, visit);
                    }
                }
            }
            Frame::Group { children, .. } => visit_frames(children, visit),
            Frame::Image(_) => {}
        }
    }
}

fn visit_background(bg: &mut Background, visit: &mut dyn FnMut(&mut Colour)) {
    match bg {
        Background::Colour { colour } => visit(colour),
        Background::Gradient { stops, .. } => {
            for stop in stops {
                visit(&mut stop.colour);
            }
        }
        Background::Texture { tint, .. } => {
            if let Some(tint) = tint {
                visit(tint);
            }
        }
        Background::Picture { .. } => {}
    }
}

/// The colours a background is made of, for checking what they refer to.
pub fn colours_of(bg: &Background) -> Vec<Colour> {
    let mut copy = bg.clone();
    let mut found = Vec::new();
    visit_background(&mut copy, &mut |c| found.push(c.clone()));
    found
}

/// Replaces every reference to a swatch with the colour the swatch shows as, for drawing a book on a
/// screen or writing it as RGB. A reference to a swatch the book lacks becomes mid-grey.
pub fn resolve_swatches(book: &mut Book) {
    let swatches = book.styles.swatches.clone();
    let all = Swatches(&swatches);
    visit_colours(book, &mut |colour| {
        if colour.swatch_id().is_some() {
            let [r, g, b] = all.resolve(colour).rgb;
            *colour = Colour(format!("#{r:02x}{g:02x}{b:02x}"));
        }
    });
}
