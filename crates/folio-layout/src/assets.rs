// What layout knows about the pictures a book uses: whether they are there, and how big they are
//
// Reads only the header of a picture, never decodes it. The plugin reads the bytes once and hands
// layout these facts, so layout stays free of files.
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::collections::HashMap;

/// What is known of one picture in the book.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AssetFact {
    /// A picture of this many pixels across and down.
    Raster { w: u32, h: u32 },
    /// A drawing that is as sharp at any size.
    Vector,
    /// There, but not a picture whose size could be read.
    Unreadable,
}

/// The facts about the pictures a book uses, by package path. A path with no entry is not in the
/// book at all, so a picture frame or background that names it is missing its picture.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AssetFacts(pub HashMap<String, AssetFact>);

/// The least pixels per inch a picture can have before preflight says so as a warning, and as an
/// error (D-23).
pub const WARN_PPI: f64 = 300.0;
pub const ERROR_PPI: f64 = 150.0;

/// How sharp a picture of `natural` pixels is when it covers `drawn_w` × `drawn_h` points: the
/// pixels per inch along its worse axis.
pub fn effective_ppi(natural: (u32, u32), drawn_w: f64, drawn_h: f64) -> f64 {
    let across = f64::from(natural.0) / (drawn_w.max(1e-6) / 72.0);
    let down = f64::from(natural.1) / (drawn_h.max(1e-6) / 72.0);
    across.min(down)
}

/// What a file of `bytes` named `path` is, from its extension and the start of it.
pub fn sniff(path: &str, bytes: &[u8]) -> AssetFact {
    let extension = path.rsplit('.').next().unwrap_or("").to_ascii_lowercase();
    if extension == "svg" {
        return AssetFact::Vector;
    }
    let size = match extension.as_str() {
        "png" => png_size(bytes),
        "jpg" | "jpeg" => jpeg_size(bytes),
        "gif" => gif_size(bytes),
        "webp" => webp_size(bytes),
        _ => None,
    };
    match size {
        Some((w, h)) if w > 0 && h > 0 => AssetFact::Raster { w, h },
        _ => AssetFact::Unreadable,
    }
}

fn be32(b: &[u8], at: usize) -> Option<u32> {
    Some(u32::from_be_bytes(b.get(at..at + 4)?.try_into().ok()?))
}

fn le16(b: &[u8], at: usize) -> Option<u32> {
    Some(u32::from(u16::from_le_bytes(
        b.get(at..at + 2)?.try_into().ok()?,
    )))
}

fn png_size(b: &[u8]) -> Option<(u32, u32)> {
    // The signature, then the IHDR chunk: its length, its name, then the width and height.
    if b.get(..8)? != b"\x89PNG\r\n\x1a\n" || b.get(12..16)? != b"IHDR" {
        return None;
    }
    Some((be32(b, 16)?, be32(b, 20)?))
}

fn gif_size(b: &[u8]) -> Option<(u32, u32)> {
    if !(b.starts_with(b"GIF87a") || b.starts_with(b"GIF89a")) {
        return None;
    }
    Some((le16(b, 6)?, le16(b, 8)?))
}

fn jpeg_size(b: &[u8]) -> Option<(u32, u32)> {
    if b.get(..2)? != [0xFF, 0xD8] {
        return None;
    }
    let mut at = 2;
    while at + 4 <= b.len() {
        if b[at] != 0xFF {
            at += 1;
            continue;
        }
        let marker = b[at + 1];
        // Fill bytes, and markers that stand alone.
        if marker == 0xFF {
            at += 1;
            continue;
        }
        if marker == 0x01 || (0xD0..=0xD9).contains(&marker) {
            at += 2;
            continue;
        }
        let length = usize::from(u16::from_be_bytes([b[at + 2], *b.get(at + 3)?]));
        // The start-of-frame markers, apart from the three that are not about size.
        if (0xC0..=0xCF).contains(&marker) && ![0xC4, 0xC8, 0xCC].contains(&marker) {
            let h = u32::from(u16::from_be_bytes([*b.get(at + 5)?, *b.get(at + 6)?]));
            let w = u32::from(u16::from_be_bytes([*b.get(at + 7)?, *b.get(at + 8)?]));
            return Some((w, h));
        }
        at += 2 + length;
    }
    None
}

fn webp_size(b: &[u8]) -> Option<(u32, u32)> {
    if b.get(..4)? != b"RIFF" || b.get(8..12)? != b"WEBP" {
        return None;
    }
    let le24 = |at: usize| -> Option<u32> {
        let s = b.get(at..at + 3)?;
        Some(u32::from(s[0]) | u32::from(s[1]) << 8 | u32::from(s[2]) << 16)
    };
    match b.get(12..16)? {
        // Lossy: a frame header with its sync code, then 14 bits each.
        b"VP8 " => Some((le16(b, 26)? & 0x3FFF, le16(b, 28)? & 0x3FFF)),
        // Lossless: a signature byte, then 14 bits each, packed.
        b"VP8L" => {
            let bits = u32::from_le_bytes(b.get(21..25)?.try_into().ok()?);
            Some(((bits & 0x3FFF) + 1, ((bits >> 14) & 0x3FFF) + 1))
        }
        // Extended: the canvas size less one, in three bytes each.
        b"VP8X" => Some((le24(24)? + 1, le24(27)? + 1)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn png_size_is_read_from_the_header() {
        let mut png = b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR".to_vec();
        png.extend_from_slice(&640u32.to_be_bytes());
        png.extend_from_slice(&480u32.to_be_bytes());
        assert_eq!(
            sniff("assets/images/a.png", &png),
            AssetFact::Raster { w: 640, h: 480 }
        );
    }

    #[test]
    fn gif_size_is_read_from_the_header() {
        let mut gif = b"GIF89a".to_vec();
        gif.extend_from_slice(&300u16.to_le_bytes());
        gif.extend_from_slice(&200u16.to_le_bytes());
        assert_eq!(sniff("a.gif", &gif), AssetFact::Raster { w: 300, h: 200 });
    }

    #[test]
    fn jpeg_size_skips_other_segments_to_the_frame() {
        // SOI, an APP0 segment of 16 bytes, then a baseline frame of 1200 × 800.
        let mut jpeg = vec![0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10];
        jpeg.extend_from_slice(&[0; 14]);
        jpeg.extend_from_slice(&[0xFF, 0xC0, 0x00, 0x11, 0x08]);
        jpeg.extend_from_slice(&800u16.to_be_bytes());
        jpeg.extend_from_slice(&1200u16.to_be_bytes());
        jpeg.extend_from_slice(&[0; 10]);
        assert_eq!(sniff("a.JPG", &jpeg), AssetFact::Raster { w: 1200, h: 800 });
    }

    #[test]
    fn webp_size_is_read_for_each_of_its_three_forms() {
        let head = |kind: &[u8; 4]| {
            let mut v = b"RIFF\0\0\0\0WEBP".to_vec();
            v.extend_from_slice(kind);
            v.extend_from_slice(&[0; 4]);
            v
        };
        let mut lossy = head(b"VP8 ");
        lossy.extend_from_slice(&[0; 3]);
        lossy.extend_from_slice(&[0x9D, 0x01, 0x2A]);
        lossy.extend_from_slice(&640u16.to_le_bytes());
        lossy.extend_from_slice(&360u16.to_le_bytes());
        assert_eq!(
            sniff("a.webp", &lossy),
            AssetFact::Raster { w: 640, h: 360 }
        );
        let mut lossless = head(b"VP8L");
        lossless.push(0x2F);
        lossless.extend_from_slice(&((639u32) | (359u32 << 14)).to_le_bytes());
        assert_eq!(
            sniff("a.webp", &lossless),
            AssetFact::Raster { w: 640, h: 360 }
        );
        let mut extended = head(b"VP8X");
        extended.extend_from_slice(&[0; 4]);
        extended.extend_from_slice(&639u32.to_le_bytes()[..3]);
        extended.extend_from_slice(&359u32.to_le_bytes()[..3]);
        assert_eq!(
            sniff("a.webp", &extended),
            AssetFact::Raster { w: 640, h: 360 }
        );
    }

    #[test]
    fn a_drawing_is_vector_and_a_broken_file_is_unreadable() {
        assert_eq!(sniff("a.svg", b"<svg/>"), AssetFact::Vector);
        assert_eq!(sniff("a.png", b"not a png"), AssetFact::Unreadable);
        assert_eq!(sniff("a.jpg", &[0xFF, 0xD8]), AssetFact::Unreadable);
        assert_eq!(sniff("a.bin", b"x"), AssetFact::Unreadable);
    }

    #[test]
    fn ppi_is_the_worse_axis() {
        // 2400 px across 8 in is 300 ppi; 1200 px down 8 in is 150.
        assert!((effective_ppi((2400, 1200), 576.0, 576.0) - 150.0).abs() < 1e-9);
    }
}
