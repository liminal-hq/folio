// Turns a choice into a book: geometry, styles, a cover and the sample spreads
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::collections::{BTreeMap, BTreeSet};

use folio_model::{
    Background, Book, BookId, BookKind, BookMetadata, Colour, Contributor, ContributorRole, Fit,
    Frame, FrameBase, FrameId, Grid, ImageFrame, ImageRef, InlineContent, Isbns, LayoutMode,
    Margins, Page, PageId, Pt, Rect, Run, Size, Spread, SpreadId, SpreadKind, StyleId, Styles,
    TextFrame, SCHEMA_VERSION,
};
use serde_json::{json, Value};

use crate::data::{kind_data, parse_trim, style_table, KindData, SampleFrame, SampleSpread};
use crate::isbn::is_valid_isbn13;
use crate::spec::{Look, NewBookSpec, Paper, PresetError, Words};

/// A new book and the files that travel with it.
#[derive(Debug, Clone)]
pub struct NewBook {
    /// The book, with the look and the named styles it uses in `book.styles`.
    pub book: Book,
    /// `settings.json`: per-book choices that are not part of the layout, such as the paper.
    pub settings: Value,
}

fn random_id(prefix: &str) -> String {
    let id = uuid::Uuid::new_v4().simple().to_string();
    format!("{prefix}_{}", &id[..12])
}

/// Makes a book from the choices in `spec`.
pub fn new_book(spec: &NewBookSpec) -> Result<NewBook, PresetError> {
    new_book_with_ids(spec, &mut random_id)
}

/// Makes a book, naming its parts with `ids` (a counter in tests, for example).
pub fn new_book_with_ids(
    spec: &NewBookSpec,
    ids: &mut dyn FnMut(&str) -> String,
) -> Result<NewBook, PresetError> {
    let data = kind_data(spec.kind);

    let sizes: Vec<(f64, f64)> = data.trims.iter().filter_map(|t| parse_trim(t)).collect();
    let (base_w, base_h) = *sizes.first().ok_or(PresetError::NoSuchTrim(0))?;
    let (width, height) = *sizes
        .get(spec.trim_index as usize)
        .ok_or(PresetError::NoSuchTrim(spec.trim_index))?;
    let isbn = spec.isbn.trim();
    if !isbn.is_empty() && !is_valid_isbn13(isbn) {
        return Err(PresetError::BadIsbn(isbn.to_owned()));
    }
    let cover_colour = Colour::parse(&spec.cover_colour)
        .map_err(|_| PresetError::BadColour(spec.cover_colour.clone()))?;

    let samples = spec.words == Words::Samples;
    let title = match spec.title.trim() {
        "" if samples => data.sample_title.clone(),
        "" => "Untitled".to_owned(),
        given => given.to_owned(),
    };
    let author = spec.author.trim().to_owned();

    // The samples are drawn for the kind's first size; other sizes scale them to fit.
    let scale = Scale {
        x: width / base_w,
        y: height / base_h,
    };
    let bleed = data.bleed_in;
    let mut builder = Builder {
        ids,
        data,
        scale,
        title: &title,
        author: &author,
        used_styles: BTreeSet::new(),
    };

    let mut spreads = vec![builder.cover(spec, &cover_colour, width, height)];
    if samples {
        let skip = usize::from(is_sample_cover(&data.spreads));
        for sample in data.spreads.iter().skip(skip) {
            spreads.push(builder.spread(sample));
        }
    } else {
        spreads.push(builder.empty_spread());
    }
    let used = builder.used_styles;

    let book = Book {
        schema: SCHEMA_VERSION,
        id: BookId(ids("bk")),
        title: title.clone(),
        subtitle: Some(spec.subtitle.trim().to_owned()).filter(|s| !s.is_empty()),
        authors: if author.is_empty() {
            Vec::new()
        } else {
            vec![Contributor {
                name: author,
                role: ContributorRole::Author,
            }]
        },
        language: "en-CA".into(),
        kind: spec.kind,
        layout: if data.flowing {
            LayoutMode::Flowing
        } else {
            LayoutMode::Fixed
        },
        trim: Size {
            w: Pt::from_in(width),
            h: Pt::from_in(height),
        },
        bleed: Pt::from_in(bleed),
        margins: Margins {
            top: Pt::from_in(data.margin_in),
            bottom: Pt::from_in(data.margin_in),
            inside: Pt::from_in(data.margin_in),
            outside: Pt::from_in(data.margin_in),
        },
        adjust_gutter_for_page_count: false,
        grid: Grid {
            columns: data.columns,
            gutter: Pt(12.0),
            baseline: None,
        },
        spreads,
        flows: Vec::new(),
        masters: Vec::new(),
        master_pages: Vec::new(),
        page_numbers: folio_model::PageNumbers::None,
        fonts: Vec::new(),
        preflight_ignore: Vec::new(),
        proofing: folio_model::Proofing::default(),
        styles: style_file(spec.look, data, &used),
        metadata: BookMetadata {
            isbn: Isbns {
                paperback: Some(isbn.to_owned()).filter(|s| !s.is_empty()),
                hardcover: None,
                ebook: None,
            },
            description: String::new(),
            keywords: Vec::new(),
        },
        extensions: Default::default(),
    };

    Ok(NewBook {
        book,
        settings: json!({
            "paper": paper_name(spec.paper),
            "plannedPages": data.default_pages,
            "coverColour": cover_colour.0,
        }),
    })
}

fn paper_name(paper: Paper) -> &'static str {
    match paper {
        Paper::White => "white",
        Paper::Cream => "cream",
        Paper::Colour => "colour",
    }
}

/// Whether the first sample spread is a cover the book can use as its own.
fn is_sample_cover(spreads: &[SampleSpread]) -> bool {
    spreads
        .first()
        .is_some_and(|s| s.single && s.name.as_deref() == Some("Cover"))
}

/// The book's styles: the look, and each named style a frame uses or the kind lists.
fn style_file(look: Look, data: &KindData, used: &BTreeSet<String>) -> Styles {
    let table = style_table();
    let mut styles = BTreeMap::new();
    for id in data.styles.iter().chain(used.iter()) {
        if let Some(style) = table.get(id) {
            styles.insert(id.clone(), style.clone());
        }
    }
    let (name, family) = match look {
        Look::Friendly => ("friendly", "Atkinson Hyperlegible"),
        Look::Modern => ("modern", "Inter"),
        Look::Classic => ("classic", "Literata"),
    };
    Styles {
        look: name.into(),
        font_family: family.into(),
        styles,
        extra: BTreeMap::new(),
        swatches: Vec::new(),
    }
}

struct Scale {
    x: f64,
    y: f64,
}

struct Builder<'a> {
    ids: &'a mut dyn FnMut(&str) -> String,
    data: &'a KindData,
    scale: Scale,
    title: &'a str,
    author: &'a str,
    used_styles: BTreeSet<String>,
}

impl Builder<'_> {
    fn base(&mut self, x: f64, y: f64, w: f64, h: f64) -> FrameBase {
        FrameBase {
            id: FrameId((self.ids)("fr")),
            rect: Rect {
                x: Pt::from_in(x),
                y: Pt::from_in(y),
                w: Pt::from_in(w),
                h: Pt::from_in(h),
            },
            rotation: 0.0,
            locked: false,
            from_master: None,
            description: None,
        }
    }

    fn text(&mut self, x: f64, y: f64, w: f64, h: f64, style: &str, text: String) -> Frame {
        self.used_styles.insert(style.to_owned());
        Frame::Text(TextFrame {
            base: self.base(x, y, w, h),
            style: StyleId(style.to_owned()),
            content: Some(InlineContent {
                inline: vec![Run {
                    text,
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

    fn frame(&mut self, sample: &SampleFrame) -> Frame {
        let (sx, sy) = (self.scale.x, self.scale.y);
        match sample {
            SampleFrame::Text {
                x,
                y,
                w,
                h,
                style,
                text,
            } => {
                let text = self.substitute(style, text);
                self.text(x * sx, y * sy, w * sx, h * sy, style, text)
            }
            SampleFrame::Image {
                x, y, w, h, label, ..
            } => Frame::Image(ImageFrame {
                base: self.base(x * sx, y * sy, w * sx, h * sy),
                image: ImageRef {
                    asset: None,
                    fit: Fit::Fill,
                    focus: (0.5, 0.5),
                    crop: None,
                },
                placeholder_label: Some(label.clone()),
            }),
        }
    }

    /// The sample words with the book's own title and author put in.
    fn substitute(&self, style: &str, text: &str) -> String {
        if text == self.data.sample_title {
            return self.title.to_owned();
        }
        if style == "byline" && !self.author.is_empty() {
            return text.replace("Mom", self.author);
        }
        text.to_owned()
    }

    fn spread(&mut self, sample: &SampleSpread) -> Spread {
        let (kind, pages) = if sample.single {
            (SpreadKind::Single, 1)
        } else if sample.opener {
            (SpreadKind::Opener, 1)
        } else {
            (SpreadKind::Spread, 2)
        };
        let frames = sample.frames.iter().map(|f| self.frame(f)).collect();
        self.assembled(kind, pages, frames)
    }

    fn empty_spread(&mut self) -> Spread {
        // The page after the cover opens on the right.
        self.assembled(SpreadKind::Opener, 1, Vec::new())
    }

    fn assembled(&mut self, kind: SpreadKind, pages: usize, frames: Vec<Frame>) -> Spread {
        Spread {
            id: SpreadId((self.ids)("sp")),
            kind,
            pages: (0..pages)
                .map(|_| Page {
                    id: PageId((self.ids)("pg")),
                    master: None,
                    bg: None,
                })
                .collect(),
            frames,
            guides: Vec::new(),
            master_overrides: Vec::new(),
            bg_span: None,
            section: None,
        }
    }

    /// The cover: the chosen colour as the page's background, with the kind's own cover pictures and
    /// words over it, or a title and byline when the kind has none.
    fn cover(
        &mut self,
        spec: &NewBookSpec,
        cover_colour: &Colour,
        width: f64,
        height: f64,
    ) -> Spread {
        let mut frames = Vec::new();
        let data = self.data;
        if spec.words == Words::Samples && is_sample_cover(&data.spreads) {
            for sample in &data.spreads[0].frames {
                frames.push(self.frame(sample));
            }
        } else {
            let margin = data.margin_in;
            let style = if spec.kind == BookKind::Picture {
                "display"
            } else {
                "title"
            };
            let title = self.title.to_owned();
            frames.push(self.text(
                margin,
                height * 0.28,
                width - 2.0 * margin,
                height * 0.3,
                style,
                title,
            ));
            if !self.author.is_empty() {
                let author = self.author.to_owned();
                frames.push(self.text(
                    margin,
                    height * 0.8,
                    width - 2.0 * margin,
                    0.6,
                    "byline",
                    author,
                ));
            }
        }
        let mut cover = self.assembled(SpreadKind::Cover, 1, frames);
        cover.pages[0].bg = Some(Background::Colour {
            colour: cover_colour.clone(),
        });
        cover
    }
}
