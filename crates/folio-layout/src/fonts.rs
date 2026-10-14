// The font book: the bundled families and the system's, resolved by family, weight and slant
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use rustybuzz::{Face, Variation};
use ttf_parser::{Permissions, Tag};

use crate::error::LayoutError;

/// The family used when the one asked for is missing, for serif and for sans text.
pub const FALLBACK_SERIF: &str = "Literata";
pub const FALLBACK_SANS: &str = "Inter";

const WGHT: Tag = Tag::from_bytes(b"wght");
const OPSZ: Tag = Tag::from_bytes(b"opsz");

/// Which font to find. Weights follow CSS: 400 regular, 700 bold.
#[derive(Debug, Clone, PartialEq)]
pub struct FontRequest<'a> {
    pub family: &'a str,
    pub weight: u16,
    pub italic: bool,
}

impl<'a> FontRequest<'a> {
    pub fn new(family: &'a str, weight: u16, italic: bool) -> Self {
        Self {
            family,
            weight,
            italic,
        }
    }
}

/// How far a font may vary along an axis.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AxisRange {
    pub min: f32,
    pub max: f32,
}

impl AxisRange {
    fn clamp(self, value: f32) -> f32 {
        value.clamp(self.min, self.max)
    }
}

/// Distances in a font at one size, in points. `descent` is positive, downwards.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Metrics {
    pub ascent: f64,
    pub descent: f64,
    pub line_gap: f64,
    pub x_height: f64,
    pub cap_height: f64,
}

/// One font file, ready to shape with.
#[derive(Debug)]
pub struct Font {
    family: String,
    data: Arc<[u8]>,
    index: u32,
    italic: bool,
    /// For a static font, the weight it has; for a variable one, unused.
    weight: u16,
    wght: Option<AxisRange>,
    opsz: Option<AxisRange>,
    embeddable: bool,
    units_per_em: f64,
}

impl Font {
    fn parse(
        family: &str,
        data: Arc<[u8]>,
        index: u32,
        italic: bool,
        weight: u16,
    ) -> Result<Self, LayoutError> {
        let bad = || LayoutError::BadFont {
            family: family.to_string(),
        };
        let face = ttf_parser::Face::parse(&data, index).map_err(|_| bad())?;
        let axis = |tag: Tag| {
            face.variation_axes().into_iter().find_map(|a| {
                (a.tag == tag).then_some(AxisRange {
                    min: a.min_value,
                    max: a.max_value,
                })
            })
        };
        let (wght, opsz) = (axis(WGHT), axis(OPSZ));
        let embeddable = !matches!(face.permissions(), Some(Permissions::Restricted));
        let units_per_em = f64::from(face.units_per_em());
        Ok(Self {
            family: family.to_string(),
            data,
            index,
            italic,
            weight,
            wght,
            opsz,
            embeddable,
            units_per_em,
        })
    }

    pub fn family(&self) -> &str {
        &self.family
    }

    pub fn italic(&self) -> bool {
        self.italic
    }

    /// The raw font file, for embedding in exports.
    pub fn data(&self) -> &Arc<[u8]> {
        &self.data
    }

    /// Whether the font's licence (`OS/2.fsType`) allows embedding it in a PDF or EPUB.
    pub fn embedding_allowed(&self) -> bool {
        self.embeddable
    }

    /// Whether the font has a weight axis, so any weight can be drawn from the one file.
    pub fn is_variable(&self) -> bool {
        self.wght.is_some()
    }

    /// The weight a request for `weight` ends up drawn at: the nearest the font can do.
    pub fn weight_for(&self, weight: u16) -> u16 {
        match self.wght {
            Some(range) => range.clamp(f32::from(weight)).round() as u16,
            None => self.weight,
        }
    }

    /// A face for shaping at `size_pt` and `weight`: the weight axis set, and the optical size
    /// where the font has one.
    pub(crate) fn face(&self, size_pt: f64, weight: u16) -> Face<'_> {
        let mut face =
            Face::from_slice(&self.data, self.index).expect("the font parsed when it was loaded");
        let mut variations = Vec::with_capacity(2);
        if let Some(range) = self.wght {
            variations.push(Variation {
                tag: WGHT,
                value: range.clamp(f32::from(weight)),
            });
        }
        if let Some(range) = self.opsz {
            variations.push(Variation {
                tag: OPSZ,
                value: range.clamp(size_pt as f32),
            });
        }
        face.set_variations(&variations);
        face
    }

    /// The font's vertical metrics at `size_pt`.
    pub fn metrics(&self, size_pt: f64) -> Metrics {
        let face = ttf_parser::Face::parse(&self.data, self.index)
            .expect("the font parsed when it was loaded");
        let scale = size_pt / self.units_per_em;
        let height = |value: Option<i16>, fallback: f64| value.map_or(fallback, f64::from) * scale;
        Metrics {
            ascent: f64::from(face.ascender()) * scale,
            descent: -f64::from(face.descender()) * scale,
            line_gap: f64::from(face.line_gap()) * scale,
            x_height: height(face.x_height(), self.units_per_em * 0.5),
            cap_height: height(face.capital_height(), self.units_per_em * 0.7),
        }
    }
}

/// What `resolve` found, and whether it is what was asked for.
#[derive(Debug, Clone)]
pub struct Resolved {
    pub font: Arc<Font>,
    /// A different family stood in because the one asked for is not installed.
    pub family_substituted: bool,
    /// The family has no italic, so upright text stands in for it.
    pub italic_missing: bool,
}

struct Bundled {
    family: &'static str,
    italic: bool,
    weight: u16,
    data: &'static [u8],
}

macro_rules! bundled {
    ($family:literal, $italic:literal, $weight:literal, $file:literal) => {
        Bundled {
            family: $family,
            italic: $italic,
            weight: $weight,
            data: include_bytes!(concat!("../fonts/", $file)),
        }
    };
}

/// The families that ship with Folio (all under the SIL Open Font Licence; see `fonts/OFL-*.txt`).
static BUNDLED: &[Bundled] = &[
    bundled!("Literata", false, 400, "Literata.ttf"),
    bundled!("Literata", true, 400, "Literata-Italic.ttf"),
    bundled!("Source Serif 4", false, 400, "SourceSerif4.ttf"),
    bundled!("Source Serif 4", true, 400, "SourceSerif4-Italic.ttf"),
    bundled!("Inter", false, 400, "Inter.ttf"),
    bundled!(
        "Atkinson Hyperlegible",
        false,
        400,
        "AtkinsonHyperlegible-Regular.ttf"
    ),
    bundled!(
        "Atkinson Hyperlegible",
        false,
        700,
        "AtkinsonHyperlegible-Bold.ttf"
    ),
    bundled!(
        "Atkinson Hyperlegible",
        true,
        400,
        "AtkinsonHyperlegible-Italic.ttf"
    ),
    bundled!(
        "Atkinson Hyperlegible",
        true,
        700,
        "AtkinsonHyperlegible-BoldItalic.ttf"
    ),
];

type CacheKey = (String, bool, u16);

/// The bundled families, plus the system's once asked for. Cheap to share: a font is loaded the
/// first time it is asked for and kept.
pub struct FontBook {
    system: Option<fontdb::Database>,
    cache: Mutex<HashMap<CacheKey, Arc<Font>>>,
}

impl Default for FontBook {
    fn default() -> Self {
        Self::bundled()
    }
}

impl FontBook {
    /// Only the families that ship with Folio. Layout is the same on every machine.
    pub fn bundled() -> Self {
        Self {
            system: None,
            cache: Mutex::new(HashMap::new()),
        }
    }

    /// The bundled families and every font installed on the system.
    pub fn with_system_fonts() -> Self {
        let mut db = fontdb::Database::new();
        db.load_system_fonts();
        Self {
            system: Some(db),
            cache: Mutex::new(HashMap::new()),
        }
    }

    /// The names of the bundled families.
    pub fn bundled_families() -> Vec<&'static str> {
        let mut names: Vec<&str> = BUNDLED.iter().map(|b| b.family).collect();
        names.dedup();
        names
    }

    /// Every family the book can draw, bundled first.
    pub fn families(&self) -> Vec<String> {
        let mut names: Vec<String> = Self::bundled_families()
            .into_iter()
            .map(String::from)
            .collect();
        if let Some(db) = &self.system {
            let mut system: Vec<String> = db
                .faces()
                .flat_map(|f| f.families.iter().map(|(name, _)| name.clone()))
                .filter(|n| !names.iter().any(|b| b.eq_ignore_ascii_case(n)))
                .collect();
            system.sort();
            system.dedup();
            names.extend(system);
        }
        names
    }

    /// The font for `request`. A family that is not installed is replaced by the serif or sans
    /// fallback (sans when the name looks like one) and the result says so.
    pub fn resolve(&self, request: &FontRequest<'_>) -> Result<Resolved, LayoutError> {
        if let Some(found) = self.find(request)? {
            return Ok(found);
        }
        let fallback = if request.family.to_ascii_lowercase().contains("sans") {
            FALLBACK_SANS
        } else {
            FALLBACK_SERIF
        };
        let mut found = self
            .find(&FontRequest::new(fallback, request.weight, request.italic))?
            .ok_or_else(|| LayoutError::NoFont {
                family: request.family.to_string(),
            })?;
        found.family_substituted = true;
        Ok(found)
    }

    fn find(&self, request: &FontRequest<'_>) -> Result<Option<Resolved>, LayoutError> {
        if let Some(found) = self.find_bundled(request)? {
            return Ok(Some(found));
        }
        self.find_system(request)
    }

    fn find_bundled(&self, request: &FontRequest<'_>) -> Result<Option<Resolved>, LayoutError> {
        let of_family: Vec<&Bundled> = BUNDLED
            .iter()
            .filter(|b| b.family.eq_ignore_ascii_case(request.family))
            .collect();
        if of_family.is_empty() {
            return Ok(None);
        }
        let italic_exists = of_family.iter().any(|b| b.italic);
        let want_italic = request.italic && italic_exists;
        let pick = of_family
            .iter()
            .filter(|b| b.italic == want_italic)
            .min_by_key(|b| (i32::from(b.weight) - i32::from(request.weight)).abs())
            .expect("a family has an upright face");
        let key = (pick.family.to_string(), pick.italic, pick.weight);
        let font = self.cached(key, || {
            Font::parse(
                pick.family,
                Arc::from(pick.data),
                0,
                pick.italic,
                pick.weight,
            )
        })?;
        Ok(Some(Resolved {
            font,
            family_substituted: false,
            italic_missing: request.italic && !italic_exists,
        }))
    }

    fn find_system(&self, request: &FontRequest<'_>) -> Result<Option<Resolved>, LayoutError> {
        let Some(db) = &self.system else {
            return Ok(None);
        };
        let families = [fontdb::Family::Name(request.family)];
        let query = |italic: bool| fontdb::Query {
            families: &families,
            weight: fontdb::Weight(request.weight),
            stretch: fontdb::Stretch::Normal,
            style: if italic {
                fontdb::Style::Italic
            } else {
                fontdb::Style::Normal
            },
        };
        let Some(id) = db.query(&query(request.italic)) else {
            return Ok(None);
        };
        let face = db.face(id).expect("the id came from the database");
        let (family, weight) = (
            face.families
                .first()
                .map_or(request.family.to_string(), |(n, _)| n.clone()),
            face.weight.0,
        );
        let italic = face.style != fontdb::Style::Normal;
        let key = (family.clone(), italic, weight);
        let font = self.cached(key, || {
            let (data, index) = db
                .with_face_data(id, |data, index| (Arc::<[u8]>::from(data), index))
                .ok_or_else(|| LayoutError::BadFont {
                    family: family.clone(),
                })?;
            Font::parse(&family, data, index, italic, weight)
        })?;
        Ok(Some(Resolved {
            font,
            family_substituted: false,
            italic_missing: request.italic && !italic,
        }))
    }

    fn cached(
        &self,
        key: CacheKey,
        load: impl FnOnce() -> Result<Font, LayoutError>,
    ) -> Result<Arc<Font>, LayoutError> {
        let mut cache = self.cache.lock().expect("the font cache lock");
        if let Some(font) = cache.get(&key) {
            return Ok(Arc::clone(font));
        }
        let font = Arc::new(load()?);
        cache.insert(key, Arc::clone(&font));
        Ok(font)
    }
}
