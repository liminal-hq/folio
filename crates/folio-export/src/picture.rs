// Pictures for a PDF: each sized for its largest place on the page and written as it suits
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Folio prepares the pictures of a PDF itself. The book keeps every picture as it was brought in,
//! so a 2400 px painting that sits in a 6 inch frame would otherwise be 400 ppi in the file. Here
//! each picture is sized for the largest place it is drawn (a picture used twice is one object in
//! the file, sized for the bigger), photographs are written as JPEG, flat art and anything with
//! transparency stays lossless, and a JPEG that is already small enough passes through byte for
//! byte.

use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::io::Cursor;
use std::sync::{Arc, Mutex};

use folio_layout::scene::{focused_rect, PageScene, SceneItem, SceneKind, SceneRect};
use folio_model::{Background, BgFit, Fit};
use image::codecs::jpeg::JpegEncoder;
use image::imageops::FilterType;
use image::{DynamicImage, ExtendedColorType, ImageDecoder, ImageFormat, ImageReader};
use krilla::image::{BitsPerComponent, CustomImage, Image, ImageColorspace};
use krilla::Data;

use crate::pdf::Assets;

/// How a picture is written when it is not passed through.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Compression {
    /// Photographs as JPEG; flat art and anything with transparency lossless.
    #[default]
    Auto,
    /// Every picture as JPEG, except those with transparency, whose colours stay lossless.
    Jpeg,
    /// Nothing lossy: every picture is written as its pixels.
    Lossless,
}

/// What a PDF does to the pictures in it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ImagePolicy {
    /// The most pixels per inch a picture keeps at its largest place; `None` keeps them all.
    pub max_ppi: Option<f32>,
    /// A picture is only reduced when it has more than this many times `max_ppi`, so a picture just
    /// over the limit is not resampled for a small saving.
    pub downsample_above: f32,
    /// JPEG quality from 1 to 100 for the pictures written as JPEG.
    pub jpeg_quality: u8,
    pub compression: Compression,
    /// A JPEG is never touched, however large it is.
    pub keep_jpeg: bool,
}

impl ImagePolicy {
    /// For email and reading on a screen.
    pub fn small() -> Self {
        Self {
            max_ppi: Some(100.0),
            downsample_above: 1.5,
            jpeg_quality: 60,
            compression: Compression::Auto,
            keep_jpeg: false,
        }
    }

    /// The Sharing PDF's usual: sharp on a screen and a modest file.
    pub fn standard() -> Self {
        Self {
            max_ppi: Some(150.0),
            downsample_above: 1.5,
            jpeg_quality: 80,
            compression: Compression::Auto,
            keep_jpeg: false,
        }
    }

    /// For a press: 300 ppi, and only a picture above 450 ppi is reduced.
    pub fn high() -> Self {
        Self {
            max_ppi: Some(300.0),
            downsample_above: 1.5,
            jpeg_quality: 90,
            compression: Compression::Auto,
            keep_jpeg: false,
        }
    }

    /// Every picture exactly as it was brought in.
    pub fn original() -> Self {
        Self {
            max_ppi: None,
            downsample_above: 1.5,
            jpeg_quality: 95,
            compression: Compression::Lossless,
            keep_jpeg: true,
        }
    }

    /// What a PDF for `target` usually does: the Sharing PDF is for a screen, the rest are for a
    /// press.
    pub fn for_target(target: crate::pdf::PdfTarget) -> Self {
        match target {
            crate::pdf::PdfTarget::Sharing => Self::standard(),
            _ => Self::high(),
        }
    }

    /// A number that is the same for two policies that treat pictures the same.
    fn key(&self) -> u64 {
        let mut h = std::collections::hash_map::DefaultHasher::new();
        self.max_ppi.map(f32::to_bits).hash(&mut h);
        self.downsample_above.to_bits().hash(&mut h);
        self.jpeg_quality.hash(&mut h);
        self.compression.hash(&mut h);
        self.keep_jpeg.hash(&mut h);
        h.finish()
    }
}

/// The pixels of a picture, as the file will hold them after compression.
#[derive(Clone)]
pub struct RawPicture {
    colour: Arc<Vec<u8>>,
    alpha: Option<Arc<Vec<u8>>>,
    size: (u32, u32),
    space: ImageColorspace,
    /// Where the pixels came from, so two pictures are told apart without hashing them again.
    id: u64,
}

impl Hash for RawPicture {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self.size.hash(state);
    }
}

impl CustomImage for RawPicture {
    fn color_channel(&self) -> &[u8] {
        &self.colour
    }
    fn alpha_channel(&self) -> Option<&[u8]> {
        self.alpha.as_ref().map(|a| a.as_slice())
    }
    fn bits_per_component(&self) -> BitsPerComponent {
        BitsPerComponent::Eight
    }
    fn size(&self) -> (u32, u32) {
        self.size
    }
    fn icc_profile(&self) -> Option<&[u8]> {
        None
    }
    fn color_space(&self) -> ImageColorspace {
        self.space
    }
}

/// A picture ready to go in the file.
#[derive(Clone)]
pub enum Prepared {
    /// The file as it was brought in (PNG, JPEG, GIF or WebP), left to the writer.
    PassThrough(Arc<Vec<u8>>, ImageFormat),
    /// A JPEG made here, in grey or sRGB.
    Jpeg(Arc<Vec<u8>>),
    /// Pixels the writer compresses without loss.
    Raw(RawPicture),
}

impl Prepared {
    /// The picture as a writer's image, or `None` for a file it cannot read.
    pub fn image(&self) -> Option<Image> {
        match self {
            Self::PassThrough(bytes, format) => {
                let data: Data = Arc::new(bytes.as_ref().clone()).into();
                match format {
                    ImageFormat::Png => Image::from_png(data, true),
                    ImageFormat::Jpeg => Image::from_jpeg(data, true),
                    ImageFormat::Gif => Image::from_gif(data, true),
                    ImageFormat::WebP => Image::from_webp(data, true),
                    _ => return None,
                }
                .ok()
            }
            Self::Jpeg(bytes) => {
                Image::from_jpeg(Arc::new(bytes.as_ref().clone()).into(), true).ok()
            }
            Self::Raw(raw) => Image::from_custom(raw.clone(), true).ok(),
        }
    }
}

/// What was made of each picture, kept so a second export, or an estimate that is then exported,
/// does not make them again. Cheap to clone: the clones share one store.
#[derive(Clone, Default)]
pub struct PictureCache(Arc<Mutex<HashMap<CacheKey, Option<Prepared>>>>);

#[derive(Clone, PartialEq, Eq, Hash)]
struct CacheKey {
    path: String,
    /// The width the picture was reduced to, or 0 for as it was.
    width: u32,
    policy: u64,
}

/// The widest each picture is drawn, in points, over the whole book: what to size it for. The width
/// is that of the picture itself, not of its frame, so a picture cropped by its frame counts as the
/// size it is drawn at.
pub type Demand = HashMap<String, f64>;

/// Reads how many pixels a picture has across and down without decoding it.
pub fn dimensions(bytes: &[u8]) -> Option<(u32, u32)> {
    ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .ok()?
        .into_dimensions()
        .ok()
}

/// The widest place each picture of `pages` is drawn. `bleed` is the room a background has beyond
/// the trim.
pub fn demand(pages: &[PageScene], assets: &dyn Assets, bleed: f64) -> Demand {
    let mut wanted = Demand::new();
    let mut natural: HashMap<String, Option<(u32, u32)>> = HashMap::new();
    for page in pages {
        for item in &page.items {
            for (asset, rect, fit, focus) in placements(item, page, bleed) {
                let size = natural
                    .entry(asset.clone())
                    .or_insert_with(|| assets.bytes(&asset).and_then(|b| dimensions(&b)));
                let Some((nw, nh)) = *size else { continue };
                let at = focused_rect(rect, (f64::from(nw), f64::from(nh)), fit, focus);
                let entry = wanted.entry(asset).or_insert(0.0);
                *entry = entry.max(at.w);
            }
        }
    }
    wanted
}

/// The pictures one scene item draws: the asset, the frame, and how the picture sits in it.
fn placements(
    item: &SceneItem,
    page: &PageScene,
    bleed: f64,
) -> Vec<(String, SceneRect, Fit, (f64, f64))> {
    match &item.kind {
        SceneKind::Picture {
            rect,
            asset: Some(asset),
            fit,
            focus,
            ..
        } => vec![(asset.clone(), *rect, *fit, *focus)],
        SceneKind::Background {
            bg: Background::Picture { asset, fit, focus },
            across,
        } => {
            let whole = SceneRect {
                x: -bleed,
                y: -bleed,
                w: page.width + 2.0 * bleed,
                h: page.height + 2.0 * bleed,
            };
            let fit = match fit {
                BgFit::Fill => Fit::Fill,
                BgFit::Fit => Fit::Fit,
            };
            vec![(asset.clone(), across.unwrap_or(whole), fit, *focus)]
        }
        _ => Vec::new(),
    }
}

/// The pixels across a picture needs at `width` points to keep `policy`'s limit, or `None` when it
/// is to be left at its own size.
fn reduced_width(natural: u32, width: f64, policy: &ImagePolicy) -> Option<u32> {
    let ppi = policy.max_ppi?;
    if width <= 0.0 {
        return None;
    }
    let wanted = (width * f64::from(ppi) / 72.0).ceil();
    if f64::from(natural) > wanted * f64::from(policy.downsample_above) && wanted >= 1.0 {
        Some(wanted as u32)
    } else {
        None
    }
}

/// How many different colours a picture may have, in a sample of it, and still be flat art.
const FLAT_COLOURS: usize = 6000;

/// Whether the pixels of a sample are few enough colours to be drawn art rather than a photograph.
fn is_flat(rgb: &image::RgbImage) -> bool {
    let (w, h) = rgb.dimensions();
    let step = (((u64::from(w) * u64::from(h)) as f64 / 65_536.0)
        .sqrt()
        .ceil() as u32)
        .max(1);
    let mut seen = HashSet::new();
    let mut y = 0;
    while y < h {
        let mut x = 0;
        while x < w {
            let p = rgb.get_pixel(x, y).0;
            seen.insert(u32::from(p[0]) << 16 | u32::from(p[1]) << 8 | u32::from(p[2]));
            if seen.len() > FLAT_COLOURS {
                return false;
            }
            x += step;
        }
        y += step;
    }
    true
}

fn hash_bytes(bytes: &[u8]) -> u64 {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    bytes.hash(&mut h);
    h.finish()
}

/// Makes what goes in the file for the picture `bytes`, drawn at most `width` points across, or
/// `None` for a file that is not a picture.
#[allow(clippy::chunks_exact_to_as_chunks)]
pub fn prepare(bytes: Arc<Vec<u8>>, width: f64, policy: &ImagePolicy) -> Option<Prepared> {
    let format = image::guess_format(&bytes).ok()?;
    let (nw, nh) = dimensions(&bytes)?;
    if nw == 0 || nh == 0 {
        return None;
    }
    let target = reduced_width(nw, width, policy);
    let jpeg = format == ImageFormat::Jpeg;
    if jpeg && (policy.keep_jpeg || target.is_none()) {
        return Some(Prepared::PassThrough(bytes, format));
    }
    if target.is_none() && policy.compression == Compression::Lossless {
        return Some(Prepared::PassThrough(bytes, format));
    }

    let mut decoder = ImageReader::new(Cursor::new(&bytes[..]))
        .with_guessed_format()
        .ok()?
        .into_decoder()
        .ok()?;
    let icc = decoder.icc_profile().ok().flatten();
    let mut picture = DynamicImage::from_decoder(decoder).ok()?;
    if let Some(w) = target {
        let h = ((u64::from(nh) * u64::from(w) + u64::from(nw) / 2) / u64::from(nw)).max(1) as u32;
        picture = picture.resize_exact(w, h, FilterType::Lanczos3);
    }
    let size = (picture.width(), picture.height());
    let has_alpha = picture.color().has_alpha() && {
        let alpha = picture.to_rgba8();
        alpha.pixels().any(|p| p.0[3] != 255)
    };
    let alpha = has_alpha.then(|| {
        picture
            .to_rgba8()
            .pixels()
            .map(|p| p.0[3])
            .collect::<Vec<u8>>()
    });
    let mut rgb = picture.to_rgb8().into_raw();
    // A picture in another profile is brought into sRGB, since the file's pictures carry none.
    if let Some(icc) = icc {
        if let Ok(profile) = folio_colour::Profile::from_icc(&icc) {
            if let Ok(Some(convert)) = profile.to_srgb() {
                convert.pixels(&mut rgb);
            }
        }
    }
    let grey = rgb.chunks_exact(3).all(|p| p[0] == p[1] && p[1] == p[2]);
    let lossy = match policy.compression {
        Compression::Lossless => false,
        Compression::Jpeg => !has_alpha,
        Compression::Auto => {
            !has_alpha
                && image::RgbImage::from_raw(size.0, size.1, rgb.clone())
                    .is_some_and(|img| !is_flat(&img))
        }
    };
    if lossy {
        let mut out = Vec::new();
        let mut encoder =
            JpegEncoder::new_with_quality(&mut out, policy.jpeg_quality.clamp(1, 100));
        if grey {
            let luma: Vec<u8> = rgb.chunks_exact(3).map(|p| p[0]).collect();
            encoder
                .encode(&luma, size.0, size.1, ExtendedColorType::L8)
                .ok()?;
        } else {
            encoder
                .encode(&rgb, size.0, size.1, ExtendedColorType::Rgb8)
                .ok()?;
        }
        return Some(Prepared::Jpeg(Arc::new(out)));
    }
    let (colour, space) = if grey {
        (
            rgb.chunks_exact(3).map(|p| p[0]).collect::<Vec<u8>>(),
            ImageColorspace::Luma,
        )
    } else {
        (rgb, ImageColorspace::Rgb)
    };
    Some(Prepared::Raw(RawPicture {
        id: hash_bytes(&colour) ^ alpha.as_deref().map_or(0, hash_bytes),
        colour: Arc::new(colour),
        alpha: alpha.map(Arc::new),
        size,
        space,
    }))
}

impl PictureCache {
    /// The picture at `path` prepared for the widest place it is drawn, made once.
    pub fn get(
        &self,
        path: &str,
        assets: &dyn Assets,
        width: f64,
        policy: &ImagePolicy,
    ) -> Option<Prepared> {
        let bytes = Arc::new(assets.bytes(path)?);
        // Two widths that reduce a picture to the same pixels are one entry.
        let key = CacheKey {
            path: path.to_string(),
            width: dimensions(&bytes)
                .and_then(|(nw, _)| reduced_width(nw, width, policy))
                .unwrap_or(0),
            policy: policy.key(),
        };
        if let Some(found) = self.0.lock().ok()?.get(&key) {
            return found.clone();
        }
        let made = prepare(bytes, width, policy);
        self.0.lock().ok()?.insert(key, made.clone());
        made
    }
}
