// Writes the KDP paperback cover of a sample book to a PDF, to look at: `cargo run -p folio-export --example sample_cover -- out.pdf [pages]`
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use folio_export::job::{export_target, Target};
use folio_export::pdf::NoAssets;
use folio_layout::{FontBook, StyleSheet};
use folio_model::{BookKind, FrameId, PageId, Paper, SpreadId};
use folio_presets::{new_book, NewBookSpec};

fn main() {
    let mut args = std::env::args().skip(1);
    let out = args.next().unwrap_or_else(|| "cover.pdf".into());
    let pages: usize = args.next().and_then(|p| p.parse().ok()).unwrap_or(32);
    let mut made = new_book(&NewBookSpec {
        kind: BookKind::Novel,
        ..NewBookSpec::default()
    })
    .expect("a sample book");
    made.book.metadata.isbn.paperback = Some("978-0-306-40615-7".into());
    made.book.metadata.description =
        "Mara has kept the light at Gannet Point for eleven winters. When a letter arrives with no stamp, the light is not all she keeps.".into();
    // Copies of the last spread until the book is as long as asked.
    let template = made.book.spreads.last().expect("a spread").clone();
    let mut n = 0;
    while folio_export::cover::spine_pages(&made.book) < pages as u32 {
        n += 1;
        let mut spread = template.clone();
        spread.id = SpreadId(format!("sp_x{n}"));
        for page in &mut spread.pages {
            page.id = PageId(format!("pg_x{n}_{}", page.id.0));
        }
        for frame in &mut spread.frames {
            if let folio_model::Frame::Text(f) = frame {
                f.base.id = FrameId(format!("fr_x{n}_{}", f.base.id.0));
            }
        }
        made.book.spreads.push(spread);
    }
    let styles = StyleSheet::from_styles(&made.book.styles);
    let files = export_target(
        Target::KdpCover,
        Paper::White,
        &made.book,
        &styles,
        &FontBook::bundled(),
        &NoAssets,
        None,
    )
    .expect("the cover exports");
    std::fs::write(&out, &files[0].bytes).expect("the file is written");
    println!("wrote {out}");
}
