// Colour: RGB to CMYK, ICC profiles and soft proofs, behind a small API over Little CMS
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Folio's colours are sRGB `#rrggbb` (D-24). Print wants ink. This crate turns one into the
//! other: with a CMYK ICC profile when there is one, and by a plain separation when there is not,
//! always keeping the ink and any very dark neutral as black ink alone, so text never prints as a
//! four-colour mix whose plates drift apart.

use lcms2::{Flags, Intent as LcmsIntent, PixelFormat, Profile as LcmsProfile, Transform};
use thiserror::Error;

/// A colour as ink, each plate from 0 (none) to 1 (full).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cmyk {
    pub c: f32,
    pub m: f32,
    pub y: f32,
    pub k: f32,
}

impl Cmyk {
    /// The total of the four plates, which a printer limits (commonly to 300 or 340 percent).
    pub fn total_ink(&self) -> f32 {
        self.c + self.m + self.y + self.k
    }
}

#[derive(Debug, Error)]
pub enum ColourError {
    #[error("that is not a colour: `{0}`")]
    NotAColour(String),
    #[error("the profile could not be read: {0}")]
    BadProfile(String),
    #[error("the profile is {found}, not {wanted}")]
    WrongSpace {
        found: &'static str,
        wanted: &'static str,
    },
}

/// `#rrggbb` as bytes.
pub fn parse_hex(text: &str) -> Result<(u8, u8, u8), ColourError> {
    let hex = text.trim().trim_start_matches('#');
    if hex.len() != 6 {
        return Err(ColourError::NotAColour(text.into()));
    }
    let byte = |at: usize| {
        u8::from_str_radix(&hex[at..at + 2], 16).map_err(|_| ColourError::NotAColour(text.into()))
    };
    Ok((byte(0)?, byte(2)?, byte(4)?))
}

/// Whether a colour is dark and nearly neutral, so it is the page's ink and prints as black alone.
pub fn is_ink(rgb: (u8, u8, u8)) -> bool {
    let (r, g, b) = rgb;
    let (hi, lo) = (r.max(g).max(b), r.min(g).min(b));
    hi < 0x58 && hi - lo <= 0x18
}

/// A plain separation with no profile: black generated from the darkest channel (full grey
/// component replacement). A neutral or an ink comes out as black alone.
pub fn rgb_to_cmyk(rgb: (u8, u8, u8)) -> Cmyk {
    let (r, g, b) = (
        f32::from(rgb.0) / 255.0,
        f32::from(rgb.1) / 255.0,
        f32::from(rgb.2) / 255.0,
    );
    if is_ink(rgb) || (rgb.0 == rgb.1 && rgb.1 == rgb.2) {
        let luma = 0.2126 * r + 0.7152 * g + 0.0722 * b;
        return Cmyk {
            c: 0.0,
            m: 0.0,
            y: 0.0,
            k: 1.0 - luma,
        };
    }
    let k = 1.0 - r.max(g).max(b);
    if k >= 1.0 {
        return Cmyk {
            c: 0.0,
            m: 0.0,
            y: 0.0,
            k: 1.0,
        };
    }
    Cmyk {
        c: (1.0 - r - k) / (1.0 - k),
        m: (1.0 - g - k) / (1.0 - k),
        y: (1.0 - b - k) / (1.0 - k),
        k,
    }
}

/// What a profile describes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Space {
    Rgb,
    Cmyk,
    Gray,
    Other,
}

impl Space {
    fn name(self) -> &'static str {
        match self {
            Space::Rgb => "RGB",
            Space::Cmyk => "CMYK",
            Space::Gray => "grey",
            Space::Other => "another kind",
        }
    }
}

/// How a colour outside what a printer can do is brought inside.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Intent {
    /// Keeps the look of the whole picture, the usual choice for photographs.
    #[default]
    Perceptual,
    /// Keeps colours that fit exactly and clips the rest.
    RelativeColorimetric,
}

impl Intent {
    fn lcms(self) -> LcmsIntent {
        match self {
            Intent::Perceptual => LcmsIntent::Perceptual,
            Intent::RelativeColorimetric => LcmsIntent::RelativeColorimetric,
        }
    }
}

/// An ICC profile.
pub struct Profile {
    inner: LcmsProfile,
    bytes: Vec<u8>,
}

impl Profile {
    /// The standard sRGB profile, which every colour in a Folio book is in.
    pub fn srgb() -> Self {
        let inner = LcmsProfile::new_srgb();
        let bytes = inner.icc().unwrap_or_default();
        Self { inner, bytes }
    }

    /// A profile read from ICC bytes, such as one the author brought or Folio downloaded.
    pub fn from_icc(bytes: &[u8]) -> Result<Self, ColourError> {
        let inner =
            LcmsProfile::new_icc(bytes).map_err(|e| ColourError::BadProfile(e.to_string()))?;
        Ok(Self {
            inner,
            bytes: bytes.to_vec(),
        })
    }

    pub fn icc(&self) -> &[u8] {
        &self.bytes
    }

    pub fn space(&self) -> Space {
        use lcms2::ColorSpaceSignature as S;
        match self.inner.color_space() {
            S::RgbData => Space::Rgb,
            S::CmykData => Space::Cmyk,
            S::GrayData => Space::Gray,
            _ => Space::Other,
        }
    }
}

/// Turns sRGB colours into ink.
pub struct Separator {
    transform: Option<Transform<[u8; 3], [u8; 4]>>,
}

impl Separator {
    /// A plain separation, with no profile behind it.
    pub fn plain() -> Self {
        Self { transform: None }
    }

    /// Separation through a CMYK profile, with black point compensation.
    pub fn through(profile: &Profile, intent: Intent) -> Result<Self, ColourError> {
        if profile.space() != Space::Cmyk {
            return Err(ColourError::WrongSpace {
                found: profile.space().name(),
                wanted: "CMYK",
            });
        }
        let srgb = LcmsProfile::new_srgb();
        let transform = Transform::new_flags(
            &srgb,
            PixelFormat::RGB_8,
            &profile.inner,
            PixelFormat::CMYK_8,
            intent.lcms(),
            Flags::BLACKPOINT_COMPENSATION,
        )
        .map_err(|e| ColourError::BadProfile(e.to_string()))?;
        Ok(Self {
            transform: Some(transform),
        })
    }

    /// `rgb` as ink. The page's ink and very dark neutrals are black alone whatever the profile.
    pub fn cmyk(&self, rgb: (u8, u8, u8)) -> Cmyk {
        if is_ink(rgb) {
            return rgb_to_cmyk(rgb);
        }
        match &self.transform {
            None => rgb_to_cmyk(rgb),
            Some(transform) => {
                let mut out = [[0u8; 4]];
                transform.transform_pixels(&[[rgb.0, rgb.1, rgb.2]], &mut out);
                let [c, m, y, k] = out[0];
                Cmyk {
                    c: f32::from(c) / 255.0,
                    m: f32::from(m) / 255.0,
                    y: f32::from(y) / 255.0,
                    k: f32::from(k) / 255.0,
                }
            }
        }
    }
}

/// What a screen shows for a colour printed on a given press: the colour sent through the press's
/// profile and back to sRGB, and whether it was outside what the press can reproduce.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Proofed {
    pub rgb: (u8, u8, u8),
    pub out_of_gamut: bool,
}

/// How far, in 8-bit levels, a proofed colour may drift from the original before it counts as
/// one the press cannot reproduce.
const GAMUT_DRIFT: i16 = 24;

/// Soft proofing for one press: how an sRGB colour would look when printed there.
pub struct Proof {
    transform: Transform<[u8; 3], [u8; 3]>,
}

impl Proof {
    pub fn new(press: &Profile, intent: Intent) -> Result<Self, ColourError> {
        let srgb = LcmsProfile::new_srgb();
        let transform = Transform::new_proofing(
            &srgb,
            PixelFormat::RGB_8,
            &srgb,
            PixelFormat::RGB_8,
            &press.inner,
            intent.lcms(),
            LcmsIntent::RelativeColorimetric,
            Flags::SOFT_PROOFING | Flags::BLACKPOINT_COMPENSATION,
        )
        .map_err(|e| ColourError::BadProfile(e.to_string()))?;
        Ok(Self { transform })
    }

    /// Proofs a run of RGBA pixels in place, leaving alpha alone, and says how many of the opaque
    /// ones were outside what the press can reproduce.
    pub fn pixels(&self, rgba: &mut [u8]) -> usize {
        let mut outside = 0;
        let mut cache: Option<([u8; 3], Proofed)> = None;
        for px in rgba.as_chunks_mut::<4>().0 {
            let rgb = [px[0], px[1], px[2]];
            // Flat areas repeat the same colour, so the last answer is often the next one too.
            let proofed = match cache {
                Some((seen, proofed)) if seen == rgb => proofed,
                _ => {
                    let proofed = self.colour((rgb[0], rgb[1], rgb[2]));
                    cache = Some((rgb, proofed));
                    proofed
                }
            };
            if px[3] > 0 && proofed.out_of_gamut {
                outside += 1;
            }
            px[..3].copy_from_slice(&[proofed.rgb.0, proofed.rgb.1, proofed.rgb.2]);
        }
        outside
    }

    pub fn colour(&self, rgb: (u8, u8, u8)) -> Proofed {
        let mut out = [[0u8; 3]];
        self.transform
            .transform_pixels(&[[rgb.0, rgb.1, rgb.2]], &mut out);
        let shown = (out[0][0], out[0][1], out[0][2]);
        let drift = [(rgb.0, shown.0), (rgb.1, shown.1), (rgb.2, shown.2)]
            .iter()
            .map(|&(a, b)| (i16::from(a) - i16::from(b)).abs())
            .max()
            .unwrap_or(0);
        Proofed {
            rgb: shown,
            out_of_gamut: drift > GAMUT_DRIFT,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_colours_parse_and_other_text_is_refused() {
        assert_eq!(parse_hex("#1c1b20").unwrap(), (0x1c, 0x1b, 0x20));
        assert_eq!(parse_hex("FFFFFF").unwrap(), (255, 255, 255));
        assert!(parse_hex("#fff").is_err());
        assert!(parse_hex("#gg0000").is_err());
    }

    #[test]
    fn the_ink_and_black_and_greys_are_black_ink_alone() {
        for rgb in [
            (0x1c, 0x1b, 0x20),
            (0, 0, 0),
            (0x14, 0x11, 0x0f),
            (128, 128, 128),
        ] {
            let c = rgb_to_cmyk(rgb);
            assert_eq!((c.c, c.m, c.y), (0.0, 0.0, 0.0), "{rgb:?}");
            assert!(c.k > 0.0);
        }
        assert!((rgb_to_cmyk((0, 0, 0)).k - 1.0).abs() < 1e-6);
        assert!((rgb_to_cmyk((255, 255, 255)).total_ink()).abs() < 1e-6);
    }

    #[test]
    fn a_colour_is_separated_with_black_taken_from_the_darkest_channel() {
        let red = rgb_to_cmyk((255, 0, 0));
        assert_eq!((red.c, red.k), (0.0, 0.0));
        assert!((red.m - 1.0).abs() < 1e-6 && (red.y - 1.0).abs() < 1e-6);
        let teal = rgb_to_cmyk((0x1f, 0x4e, 0x5f));
        assert!(teal.k > 0.5 && teal.c > 0.0);
        assert!(teal.total_ink() <= 4.0);
    }

    #[test]
    fn only_dark_near_neutrals_count_as_ink() {
        assert!(is_ink((0x1c, 0x1b, 0x20)));
        assert!(!is_ink((0xff, 0, 0)));
        assert!(
            !is_ink((0x80, 0x80, 0x80)),
            "a mid grey is a colour, not the ink"
        );
        assert!(!is_ink((0x20, 0x10, 0x50)), "a dark blue keeps its colour");
    }

    #[test]
    fn the_srgb_profile_is_rgb_and_a_cmyk_profile_is_needed_to_separate() {
        let srgb = Profile::srgb();
        assert_eq!(srgb.space(), Space::Rgb);
        assert!(!srgb.icc().is_empty());
        assert!(matches!(
            Separator::through(&srgb, Intent::Perceptual),
            Err(ColourError::WrongSpace { .. })
        ));
        assert!(Profile::from_icc(b"not a profile").is_err());
    }

    /// Ghostscript ships a CMYK profile; where it is installed the profile path is exercised.
    fn press() -> Option<Profile> {
        let bytes = std::fs::read("/usr/share/ghostscript/iccprofiles/default_cmyk.icc").ok()?;
        Profile::from_icc(&bytes).ok()
    }

    #[test]
    fn through_a_profile_a_colour_is_separated_and_the_ink_stays_black_alone() {
        let Some(press) = press() else { return };
        assert_eq!(press.space(), Space::Cmyk);
        let separator = Separator::through(&press, Intent::Perceptual).unwrap();
        let ink = separator.cmyk((0x1c, 0x1b, 0x20));
        assert_eq!((ink.c, ink.m, ink.y), (0.0, 0.0, 0.0));
        let red = separator.cmyk((200, 30, 30));
        assert!(red.m > 0.5 && red.y > 0.5 && red.c < 0.3, "{red:?}");
        let white = separator.cmyk((255, 255, 255));
        assert!(white.total_ink() < 0.1, "{white:?}");
    }

    #[test]
    fn soft_proofing_through_a_press_changes_a_vivid_colour_and_leaves_white_alone() {
        let Some(press) = press() else { return };
        let proof = Proof::new(&press, Intent::RelativeColorimetric).unwrap();
        let vivid = proof.colour((0, 255, 0));
        assert_ne!(
            vivid.rgb,
            (0, 255, 0),
            "a press cannot print pure screen green"
        );
        let white = proof.colour((255, 255, 255));
        assert!(
            white.rgb.0 > 240 && white.rgb.1 > 240 && white.rgb.2 > 240,
            "{white:?}"
        );
    }
}
