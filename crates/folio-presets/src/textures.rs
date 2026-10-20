// The textures a page background can have, as SVG tiles
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! The page draws each texture in TypeScript (`textures.tsx`) as an SVG pattern of a fixed size.
//! An exporter needs the same tile without a webview, so the drawings are written here as SVG
//! documents too, the same size, base colour and marks, and checked against a shared fixture of
//! sizes and base colours (`textures.json`) that the TypeScript side is tested against as well.

/// One tile of a texture.
#[derive(Debug, Clone, PartialEq)]
pub struct Texture {
    /// The side of the tile in points, so a texture is the same size on every book and page.
    pub size: f64,
    /// The tile as a complete SVG document.
    pub svg: String,
}

/// The size and untinted base colour of every texture, as the fixture records them.
pub const FACTS: [(&str, f64, &str); 5] = [
    ("linen", 12.0, "#ece5d3"),
    ("laid-paper", 36.0, "#f2eee3"),
    ("kraft", 120.0, "#c8a27a"),
    ("watercolour", 200.0, "#e3eef2"),
    ("dot-grid", 14.0, "none"),
];

/// The tile for texture `id`, with `tint` (a `#rrggbb`) in place of its base colour, or `None` for
/// an id Folio does not draw.
pub fn texture(id: &str, tint: Option<&str>) -> Option<Texture> {
    let &(_, size, base) = FACTS.iter().find(|(name, _, _)| *name == id)?;
    let base = tint.filter(|t| is_hex(t)).unwrap_or(base);
    let s = size;
    let ground = if base == "none" {
        String::new()
    } else {
        format!("<rect width=\"{s}\" height=\"{s}\" fill=\"{base}\"/>")
    };
    let marks = match id {
        "linen" => format!(
            "<path d=\"M0 {a}H{s}M0 {b}H{s}\" stroke=\"#000\" stroke-opacity=\"0.1\" stroke-width=\"0.5\"/><path d=\"M{a} 0V{s}M{b} 0V{s}\" stroke=\"#000\" stroke-opacity=\"0.08\" stroke-width=\"0.5\"/>",
            a = s / 4.0,
            b = s * 3.0 / 4.0
        ),
        "laid-paper" => {
            let lines: String = (0..12).map(|i| format!("M0 {}H{s}", i * 3 + 1)).collect();
            format!(
                "<path d=\"{lines}\" stroke=\"#000\" stroke-opacity=\"0.07\" stroke-width=\"0.4\"/><path d=\"M0 0V{s}\" stroke=\"#000\" stroke-opacity=\"0.12\" stroke-width=\"0.8\"/>"
            )
        }
        "kraft" => format!(
            "<filter id=\"noise\" x=\"0\" y=\"0\" width=\"100%\" height=\"100%\"><feTurbulence type=\"fractalNoise\" baseFrequency=\"0.9\" numOctaves=\"2\" seed=\"3\" stitchTiles=\"stitch\"/><feColorMatrix type=\"saturate\" values=\"0\"/></filter><rect width=\"{s}\" height=\"{s}\" filter=\"url(#noise)\" opacity=\"0.4\"/>"
        ),
        "watercolour" => format!(
            "<filter id=\"blur\" x=\"-30%\" y=\"-30%\" width=\"160%\" height=\"160%\"><feGaussianBlur stdDeviation=\"{}\"/></filter><g filter=\"url(#blur)\"><ellipse cx=\"{}\" cy=\"{}\" rx=\"{}\" ry=\"{}\" fill=\"#8fc0d2\" opacity=\"0.55\"/><ellipse cx=\"{}\" cy=\"{}\" rx=\"{}\" ry=\"{}\" fill=\"#efb9c4\" opacity=\"0.55\"/><ellipse cx=\"{}\" cy=\"{}\" rx=\"{}\" ry=\"{}\" fill=\"#bfdcaa\" opacity=\"0.55\"/></g>",
            s / 25.0,
            s * 0.34, s * 0.34, s * 0.14, s * 0.11,
            s * 0.64, s * 0.5, s * 0.15, s * 0.12,
            s * 0.42, s * 0.68, s * 0.16, s * 0.1
        ),
        "dot-grid" => format!(
            "<circle cx=\"{}\" cy=\"{}\" r=\"0.9\" fill=\"#000\" fill-opacity=\"0.28\"/>",
            s / 2.0,
            s / 2.0
        ),
        _ => return None,
    };
    Some(Texture {
        size,
        svg: format!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{s}\" height=\"{s}\" viewBox=\"0 0 {s} {s}\">{ground}{marks}</svg>"
        ),
    })
}

fn is_hex(text: &str) -> bool {
    text.len() == 7 && text.starts_with('#') && text[1..].chars().all(|c| c.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_texture_folio_names_has_a_tile_and_an_unknown_one_has_none() {
        for id in folio_model::TEXTURE_IDS {
            let tile = texture(id, None).unwrap_or_else(|| panic!("{id}"));
            assert!(tile.svg.starts_with("<svg") && tile.svg.ends_with("</svg>"));
        }
        assert!(texture("marble", None).is_none());
        assert_eq!(FACTS.len(), folio_model::TEXTURE_IDS.len());
    }

    #[test]
    fn a_tint_replaces_the_base_and_a_bad_one_is_ignored() {
        let tinted = texture("linen", Some("#112233")).unwrap();
        assert!(tinted.svg.contains("fill=\"#112233\"") && !tinted.svg.contains("#ece5d3"));
        let plain = texture("linen", Some("not a colour")).unwrap();
        assert!(plain.svg.contains("#ece5d3"));
        // The dot grid has no base to replace: a tint gives it one.
        assert!(!texture("dot-grid", None).unwrap().svg.contains("<rect"));
        assert!(texture("dot-grid", Some("#ffffff"))
            .unwrap()
            .svg
            .contains("<rect"));
    }

    #[test]
    fn the_sizes_and_bases_are_the_ones_the_page_draws_with() {
        let text = std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../apps/folio/src/canvas/fixtures/textures.json"),
        )
        .unwrap();
        let fixture: serde_json::Value = serde_json::from_str(&text).unwrap();
        for (id, size, base) in FACTS {
            assert_eq!(fixture[id]["size"].as_f64(), Some(size), "{id}");
            assert_eq!(fixture[id]["base"].as_str(), Some(base), "{id}");
        }
    }
}
