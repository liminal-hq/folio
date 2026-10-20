// The font book: the bundled families and the system's, resolved by family, weight and slant
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use rustybuzz::{Face, Variation};
use sha2::{Digest, Sha256};
use ttf_parser::Tag;

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
    rights: EmbeddingRights,
    face_key: String,
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
        let rights = embedding_rights_of(&face);
        let face_key = face_key_of(&data, index);
        let units_per_em = f64::from(face.units_per_em());
        Ok(Self {
            family: family.to_string(),
            data,
            index,
            italic,
            weight,
            wght,
            opsz,
            rights,
            face_key,
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

    /// What the font's licence (`OS/2.fsType`) lets Folio do with the file.
    pub fn embedding_rights(&self) -> EmbeddingRights {
        self.rights
    }

    /// Whether the licence allows embedding the font in a PDF at all.
    pub fn embedding_allowed(&self) -> bool {
        self.rights.may_embed_in_pdf()
    }

    /// A name for this exact font file and face that does not change between runs or machines: the
    /// start of the file's SHA-256 and the face's index. Two files with the same family name have
    /// different keys, so a book can say which one it was laid out with.
    pub fn face_key(&self) -> &str {
        &self.face_key
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

/// Where a font comes from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontSource {
    /// Ships with Folio, so every machine has it.
    Bundled,
    /// Kept inside the book, so it travels with the file.
    Book,
    /// Installed on this computer.
    System,
}

/// A font file a book keeps with it: what it is called and where its bytes are.
#[derive(Debug, Clone)]
pub struct BookFace {
    pub family: String,
    pub weight: u16,
    pub italic: bool,
    /// Which face of a collection file; 0 for an ordinary font.
    pub index: u32,
    pub data: Arc<[u8]>,
}

/// One face a book could be drawn in, without loading its file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FaceInfo {
    pub family: String,
    pub weight: u16,
    pub italic: bool,
    pub source: FontSource,
}

/// The bundled families, plus the system's once asked for. Cheap to share: a font is loaded the
/// first time it is asked for and kept.
pub struct FontBook {
    system: Option<Arc<fontdb::Database>>,
    /// Fonts the book carries, looked at after the bundled ones and before the system's.
    book: Vec<BookFace>,
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
            book: Vec::new(),
            cache: Mutex::new(HashMap::new()),
        }
    }

    /// The bundled families and every font installed on the system.
    pub fn with_system_fonts() -> Self {
        let mut db = fontdb::Database::new();
        db.load_system_fonts();
        Self {
            system: Some(Arc::new(db)),
            book: Vec::new(),
            cache: Mutex::new(HashMap::new()),
        }
    }

    /// This book of fonts with a book's own fonts added. The system's fonts are shared, not read
    /// again, so making one per book is cheap. A font in the book wins over an installed one of the
    /// same family, so the book is laid out the same on a machine that has the font and on one
    /// that does not.
    pub fn layered(&self, faces: Vec<BookFace>) -> Self {
        Self {
            system: self.system.clone(),
            book: faces,
            cache: Mutex::new(HashMap::new()),
        }
    }

    /// The names of the bundled families.
    pub fn bundled_families() -> Vec<&'static str> {
        let mut names: Vec<&str> = BUNDLED.iter().map(|b| b.family).collect();
        names.dedup();
        names
    }

    /// Reads the system's fonts again, for fonts installed or removed since the book was made, and
    /// forgets the fonts loaded so far so they are read afresh.
    pub fn refresh_system_fonts(&mut self) {
        if self.system.is_some() {
            let mut db = fontdb::Database::new();
            db.load_system_fonts();
            self.system = Some(Arc::new(db));
        }
        self.cache.lock().expect("the font cache lock").clear();
    }

    /// Every face the book can draw, the bundled ones first, then the system's by family.
    pub fn faces(&self) -> Vec<FaceInfo> {
        let mut faces: Vec<FaceInfo> = BUNDLED
            .iter()
            .map(|b| FaceInfo {
                family: b.family.to_string(),
                weight: b.weight,
                italic: b.italic,
                source: FontSource::Bundled,
            })
            .collect();
        faces.extend(self.book.iter().map(|b| FaceInfo {
            family: b.family.clone(),
            weight: b.weight,
            italic: b.italic,
            source: FontSource::Book,
        }));
        if let Some(db) = &self.system {
            let mut system: Vec<FaceInfo> = db
                .faces()
                .filter_map(|f| {
                    let family = f.families.first().map(|(name, _)| name.clone())?;
                    Some(FaceInfo {
                        weight: f.weight.0,
                        italic: f.style != fontdb::Style::Normal,
                        source: FontSource::System,
                        family,
                    })
                })
                .filter(|f| {
                    !BUNDLED
                        .iter()
                        .any(|b| b.family.eq_ignore_ascii_case(&f.family))
                        && !self
                            .book
                            .iter()
                            .any(|b| b.family.eq_ignore_ascii_case(&f.family))
                })
                .collect();
            system.sort_by(|a, b| {
                (a.family.to_lowercase(), a.weight, a.italic).cmp(&(
                    b.family.to_lowercase(),
                    b.weight,
                    b.italic,
                ))
            });
            system.dedup();
            faces.extend(system);
        }
        faces
    }

    /// Every family the book can draw, bundled first.
    pub fn families(&self) -> Vec<String> {
        let mut names: Vec<String> = Self::bundled_families()
            .into_iter()
            .map(String::from)
            .collect();
        for face in &self.book {
            if !names.iter().any(|n| n.eq_ignore_ascii_case(&face.family)) {
                names.push(face.family.clone());
            }
        }
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
        if let Some(found) = self.find_book(request)? {
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

    fn find_book(&self, request: &FontRequest<'_>) -> Result<Option<Resolved>, LayoutError> {
        let of_family: Vec<&BookFace> = self
            .book
            .iter()
            .filter(|b| b.family.eq_ignore_ascii_case(request.family))
            .collect();
        if of_family.is_empty() {
            return Ok(None);
        }
        let italic_exists = of_family.iter().any(|b| b.italic);
        let want_italic = request.italic && italic_exists;
        let Some(pick) = of_family
            .iter()
            .filter(|b| b.italic == want_italic)
            .min_by_key(|b| (i32::from(b.weight) - i32::from(request.weight)).abs())
        else {
            return Ok(None);
        };
        let key = (pick.family.clone(), pick.italic, pick.weight);
        let font = self.cached(key, || {
            Font::parse(
                &pick.family,
                Arc::clone(&pick.data),
                pick.index,
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

/// What a font's licence says about embedding it, from `OS/2.fsType`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Embedding {
    /// No restriction: the font may be embedded and kept with a book.
    Installable,
    /// May be embedded and the document edited: the same for a book as installable.
    Editable,
    /// May be embedded in a document to be viewed and printed, but not kept as a file or edited.
    PreviewPrint,
    /// May not be embedded at all.
    Restricted,
}

/// The embedding rules of one font file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EmbeddingRights {
    pub level: Embedding,
    /// Only the whole font may be embedded, never a subset of the glyphs a book uses.
    pub no_subsetting: bool,
    /// Only bitmaps may be embedded, so the outlines may not be.
    pub bitmap_only: bool,
}

impl EmbeddingRights {
    /// No restriction at all, which is what a font with no `OS/2` table says.
    pub const FREE: Self = Self {
        level: Embedding::Installable,
        no_subsetting: false,
        bitmap_only: false,
    };

    /// Reads the `fsType` field: bit 1 restricts, bit 2 is preview and print, bit 3 editable, bit
    /// 8 forbids subsetting and bit 9 allows only bitmaps. When more than one of bits 1 to 3 is
    /// set the most restrictive wins.
    pub fn from_fs_type(fs_type: u16) -> Self {
        let level = if fs_type & 0x0002 != 0 {
            Embedding::Restricted
        } else if fs_type & 0x0004 != 0 {
            Embedding::PreviewPrint
        } else if fs_type & 0x0008 != 0 {
            Embedding::Editable
        } else {
            Embedding::Installable
        };
        Self {
            level,
            no_subsetting: fs_type & 0x0100 != 0,
            bitmap_only: fs_type & 0x0200 != 0,
        }
    }

    /// Whether the font may be embedded in a PDF (whole or as a subset, see `no_subsetting`).
    pub fn may_embed_in_pdf(&self) -> bool {
        self.level != Embedding::Restricted && !self.bitmap_only
    }

    /// Whether the font file may be kept inside a `.folio` or put in an EPUB, which anyone who has
    /// the file can then extract: only an installable or editable font may.
    pub fn may_keep_with_book(&self) -> bool {
        matches!(self.level, Embedding::Installable | Embedding::Editable) && !self.bitmap_only
    }

    /// Whether an export has to embed the whole font rather than the glyphs it uses.
    pub fn must_embed_whole(&self) -> bool {
        self.no_subsetting
    }
}

fn embedding_rights_of(face: &ttf_parser::Face<'_>) -> EmbeddingRights {
    let Some(os2) = face.raw_face().table(Tag::from_bytes(b"OS/2")) else {
        return EmbeddingRights::FREE;
    };
    let Some(bytes) = os2.get(8..10) else {
        return EmbeddingRights::FREE;
    };
    EmbeddingRights::from_fs_type(u16::from_be_bytes([bytes[0], bytes[1]]))
}

/// The start of the file's SHA-256, then the face's index in it.
fn face_key_of(data: &[u8], index: u32) -> String {
    let digest = Sha256::digest(data);
    let hex: String = digest.iter().take(8).map(|b| format!("{b:02x}")).collect();
    format!("{hex}-{index}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fs_type_zero_is_free_and_the_flags_read_as_the_spec_says() {
        assert_eq!(EmbeddingRights::from_fs_type(0), EmbeddingRights::FREE);
        let restricted = EmbeddingRights::from_fs_type(0x0002);
        assert_eq!(restricted.level, Embedding::Restricted);
        assert!(!restricted.may_embed_in_pdf() && !restricted.may_keep_with_book());
        let preview = EmbeddingRights::from_fs_type(0x0004);
        assert_eq!(preview.level, Embedding::PreviewPrint);
        assert!(preview.may_embed_in_pdf() && !preview.may_keep_with_book());
        let editable = EmbeddingRights::from_fs_type(0x0008);
        assert_eq!(editable.level, Embedding::Editable);
        assert!(editable.may_embed_in_pdf() && editable.may_keep_with_book());
    }

    #[test]
    fn no_subsetting_means_embed_it_whole_not_that_it_cannot_be_embedded() {
        let whole = EmbeddingRights::from_fs_type(0x0100);
        assert!(whole.must_embed_whole());
        assert!(whole.may_embed_in_pdf() && whole.may_keep_with_book());
        assert!(!EmbeddingRights::FREE.must_embed_whole());
    }

    #[test]
    fn bitmap_only_cannot_embed_the_outlines_and_the_most_restrictive_level_wins() {
        let bitmaps = EmbeddingRights::from_fs_type(0x0200);
        assert!(!bitmaps.may_embed_in_pdf() && !bitmaps.may_keep_with_book());
        assert_eq!(
            EmbeddingRights::from_fs_type(0x000e).level,
            Embedding::Restricted
        );
        assert_eq!(
            EmbeddingRights::from_fs_type(0x000c).level,
            Embedding::PreviewPrint
        );
    }

    #[test]
    fn the_faces_start_with_the_bundled_ones_and_say_where_each_comes_from() {
        let faces = FontBook::bundled().faces();
        assert!(faces.iter().all(|f| f.source == FontSource::Bundled));
        assert!(faces.iter().any(|f| f.family == FALLBACK_SERIF));
        assert!(faces.iter().any(|f| f.family == FALLBACK_SANS));
    }

    #[test]
    fn a_font_the_book_keeps_is_found_by_its_family_and_not_called_a_substitute() {
        let base = FontBook::bundled();
        let inter = base
            .resolve(&FontRequest::new(FALLBACK_SANS, 400, false))
            .unwrap()
            .font;
        let book = base.layered(vec![BookFace {
            family: "Zed Sans".into(),
            weight: 400,
            italic: false,
            index: 0,
            data: Arc::clone(inter.data()),
        }]);
        let found = book
            .resolve(&FontRequest::new("zed sans", 400, false))
            .unwrap();
        assert!(!found.family_substituted);
        assert_eq!(found.font.family(), "Zed Sans");
        assert_eq!(found.font.face_key(), inter.face_key());
        // The same family asked of a book without it falls back, as before.
        assert!(
            base.resolve(&FontRequest::new("Zed Sans", 400, false))
                .unwrap()
                .family_substituted
        );
        assert!(book.families().iter().any(|f| f == "Zed Sans"));
        assert!(book
            .faces()
            .iter()
            .any(|f| f.family == "Zed Sans" && f.source == FontSource::Book));
    }

    #[test]
    fn refreshing_a_bundled_only_book_changes_nothing() {
        let mut book = FontBook::bundled();
        let before = book.faces();
        book.refresh_system_fonts();
        assert_eq!(book.faces(), before);
    }

    #[test]
    fn the_bundled_fonts_are_free_and_each_file_has_its_own_stable_key() {
        let book = FontBook::bundled();
        let serif = book
            .resolve(&FontRequest::new(FALLBACK_SERIF, 400, false))
            .unwrap();
        let sans = book
            .resolve(&FontRequest::new(FALLBACK_SANS, 400, false))
            .unwrap();
        assert_eq!(serif.font.embedding_rights(), EmbeddingRights::FREE);
        assert_ne!(serif.font.face_key(), sans.font.face_key());
        let again = FontBook::bundled()
            .resolve(&FontRequest::new(FALLBACK_SERIF, 400, false))
            .unwrap();
        assert_eq!(serif.font.face_key(), again.font.face_key());
        assert!(serif.font.face_key().ends_with("-0"));
        assert_eq!(serif.font.face_key().len(), 16 + 2);
    }
}
