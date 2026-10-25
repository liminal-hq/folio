// A texture tile as a picture, at the resolution a printer wants
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use resvg::{tiny_skia, usvg};

/// How sharp a printed texture is, in pixels to the inch.
const PPI: f64 = 300.0;

/// One tile of texture `id` (tinted with `tint`) drawn at 300 pixels to the inch as a PNG, with the
/// tile's side in points. `None` for a texture Folio does not draw.
pub fn tile_png(id: &str, tint: Option<&str>) -> Option<(Vec<u8>, f64)> {
    let tile = folio_presets::textures::texture(id, tint)?;
    let tree = usvg::Tree::from_str(&tile.svg, &usvg::Options::default()).ok()?;
    let px = (tile.size * PPI / 72.0).round().max(1.0) as u32;
    let mut pixmap = tiny_skia::Pixmap::new(px, px)?;
    let scale = px as f32 / tile.size as f32;
    resvg::render(
        &tree,
        tiny_skia::Transform::from_scale(scale, scale),
        &mut pixmap.as_mut(),
    );
    Some((pixmap.encode_png().ok()?, tile.size))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_texture_draws_a_square_tile_at_300_ppi() {
        for id in folio_model::TEXTURE_IDS {
            let (png, size) = tile_png(id, None).unwrap_or_else(|| panic!("{id}"));
            let img = image::load_from_memory(&png).unwrap();
            let want = (size * 300.0 / 72.0).round() as u32;
            assert_eq!((img.width(), img.height()), (want, want), "{id}");
        }
        assert!(tile_png("marble", None).is_none());
    }

    #[test]
    fn a_tinted_linen_tile_is_that_colour_with_marks_over_it() {
        let (png, _) = tile_png("linen", Some("#336699")).unwrap();
        let img = image::load_from_memory(&png).unwrap().to_rgba8();
        // A corner away from the marks is the tint.
        let corner = img.get_pixel(1, 1);
        assert!(
            (i16::from(corner[0]) - 0x33).abs() < 12 && (i16::from(corner[2]) - 0x99).abs() < 12,
            "{corner:?}"
        );
        // The marks darken some pixels.
        assert!(img.pixels().any(|p| p[2] < 0x92));
    }

    #[test]
    fn the_dot_grid_is_transparent_apart_from_its_dot() {
        let (png, _) = tile_png("dot-grid", None).unwrap();
        let img = image::load_from_memory(&png).unwrap().to_rgba8();
        assert_eq!(img.get_pixel(0, 0)[3], 0);
        let middle = img.width() / 2;
        assert!(img.get_pixel(middle, middle)[3] > 30);
    }
}
