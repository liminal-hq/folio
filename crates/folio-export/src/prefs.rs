// Export settings combined: built-in, your defaults, the book's, this export's, and each target's locks
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! A person's choices live in layers (D-73). `resolve` is the one place they are combined: the
//! highest layer that made a choice wins, and what the target locks is then put right, with a note
//! saying so ("Pictures kept at 300 ppi for KDP") so the dialog never silently ignores a choice.

use folio_model::{
    BleedChoice, Book, BookmarkChoice, CoverChoice, ExportPrefs, PdfPrefs, PictureCompression,
    PicturePreset, SheetsChoice, ViewerLayoutChoice,
};

use crate::job::PdfExtras;
use crate::outline::{Bookmarks, Navigation, PdfInfo, ViewerLayout};
use crate::pages::{CoverPages, PageChoice, Sheets};
use crate::picture::{Compression, ImagePolicy};
use crate::shop::{ShopBleed, ShopOptions};

/// Which of the three kinds of PDF settings are for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PdfKind {
    Sharing,
    /// The paperback interior and cover for Kindle Direct Publishing.
    Kdp,
    PrintShop,
}

/// What a press file's colour is, as chosen.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct InkChoice {
    pub enabled: bool,
    /// The path of the press's CMYK ICC file.
    pub profile: Option<String>,
}

/// The settings for one kind of PDF, ready to use.
#[derive(Clone)]
pub struct Resolved {
    pub extras: PdfExtras,
    pub ink: InkChoice,
    /// Bookmarks are by chapter, so the engine must say where the chapters are.
    pub wants_chapters: bool,
    /// What was changed because the target needs it, one line each.
    pub notes: Vec<String>,
    /// Every choice as it came out, for showing in a dialog.
    pub prefs: PdfPrefs,
}

/// What nothing has been chosen means for `kind`.
pub fn builtin(kind: PdfKind) -> PdfPrefs {
    let base = PdfPrefs {
        compression: Some(PictureCompression::Auto),
        keep_jpeg: Some(false),
        crop_to_frame: Some(false),
        bookmarks: Some(BookmarkChoice::None),
        page_labels: Some(false),
        links: Some(false),
        viewer_layout: Some(ViewerLayoutChoice::Single),
        sheets: Some(SheetsChoice::Pages),
        cover: Some(CoverChoice::Front),
        crop_marks: Some(false),
        bleed: Some(BleedChoice::Book),
        ink: Some(false),
        ..PdfPrefs::default()
    };
    match kind {
        PdfKind::Sharing => PdfPrefs {
            preset: Some(PicturePreset::Standard),
            bookmarks: Some(BookmarkChoice::Chapters),
            page_labels: Some(true),
            links: Some(true),
            ..base
        },
        PdfKind::Kdp => PdfPrefs {
            preset: Some(PicturePreset::High),
            ..base
        },
        PdfKind::PrintShop => PdfPrefs {
            preset: Some(PicturePreset::High),
            crop_marks: Some(true),
            ..base
        },
    }
}

fn of_kind(prefs: &ExportPrefs, kind: PdfKind) -> &PdfPrefs {
    match kind {
        PdfKind::Sharing => &prefs.pdf.sharing,
        PdfKind::Kdp => &prefs.pdf.kdp,
        PdfKind::PrintShop => &prefs.pdf.print_shop,
    }
}

/// The policy for pictures that `prefs` ask for.
fn picture_policy(prefs: &PdfPrefs) -> ImagePolicy {
    let mut policy = match prefs.preset.unwrap_or(PicturePreset::Standard) {
        PicturePreset::Small => ImagePolicy::small(),
        PicturePreset::Standard => ImagePolicy::standard(),
        PicturePreset::High => ImagePolicy::high(),
        PicturePreset::Original => ImagePolicy::original(),
    };
    if let Some(ppi) = prefs.max_ppi {
        policy.max_ppi = Some(ppi as f32);
    }
    if let Some(q) = prefs.jpeg_quality {
        policy.jpeg_quality = q.clamp(1, 100);
    }
    if let Some(c) = prefs.compression {
        policy.compression = match c {
            PictureCompression::Auto => Compression::Auto,
            PictureCompression::Jpeg => Compression::Jpeg,
            PictureCompression::Lossless => Compression::Lossless,
        };
    }
    if let Some(keep) = prefs.keep_jpeg {
        policy.keep_jpeg = keep;
    }
    policy
}

/// Puts right what `kind`'s destination does not allow, saying what was changed.
fn lock(kind: PdfKind, prefs: &mut PdfPrefs, notes: &mut Vec<String>) {
    let name = match kind {
        PdfKind::Sharing => return,
        PdfKind::Kdp => "KDP",
        PdfKind::PrintShop => "the print shop",
    };
    // A press wants 300 pixels an inch and no heavy compression.
    if matches!(
        prefs.preset,
        Some(PicturePreset::Small | PicturePreset::Standard)
    ) {
        prefs.preset = Some(PicturePreset::High);
        notes.push(format!(
            "Changed for {name}: pictures use the High setting, 300 ppi"
        ));
    }
    if prefs.max_ppi.is_some_and(|p| p < 300) {
        prefs.max_ppi = Some(300);
        notes.push(format!("Changed for {name}: pictures kept at 300 ppi"));
    }
    if prefs.jpeg_quality.is_some_and(|q| q < 80) {
        prefs.jpeg_quality = Some(80);
        notes.push(format!("Changed for {name}: JPEG quality at least 80"));
    }
    let mut set_back = |changed: bool, what: &str| {
        if changed {
            notes.push(format!("Changed for {name}: {what}"));
        }
    };
    // Links go in no press file.
    set_back(prefs.links == Some(true), "no links in the file");
    prefs.links = Some(false);
    // The whole book, as single pages.
    let pages_asked = prefs.range.is_some()
        || prefs.sheets == Some(SheetsChoice::Spreads)
        || prefs.cover == Some(CoverChoice::FrontAndBack)
        || prefs.cover == Some(CoverChoice::None);
    set_back(pages_asked, "the whole book as single pages");
    prefs.range = None;
    prefs.sheets = Some(SheetsChoice::Pages);
    prefs.cover = Some(CoverChoice::Front);
    prefs.viewer_layout = Some(ViewerLayoutChoice::Single);
    if kind == PdfKind::Kdp {
        let extras_asked = prefs.bookmarks.is_some_and(|b| b != BookmarkChoice::None)
            || prefs.page_labels == Some(true)
            || prefs.subject.is_some()
            || prefs.keywords.is_some()
            || prefs.title.is_some()
            || prefs.authors.is_some()
            || prefs.language.is_some();
        set_back(
            extras_asked,
            "no bookmarks, page labels or information beyond the title and author",
        );
        prefs.bookmarks = Some(BookmarkChoice::None);
        prefs.page_labels = Some(false);
        prefs.subject = None;
        prefs.keywords = None;
        prefs.title = None;
        prefs.authors = None;
        prefs.language = None;
        set_back(
            prefs.crop_marks == Some(true)
                || prefs.slug.is_some()
                || matches!(prefs.bleed, Some(BleedChoice::Custom { .. })),
            "no crop marks, and KDP's own bleed",
        );
        prefs.crop_marks = Some(false);
        prefs.slug = None;
        prefs.bleed = Some(BleedChoice::Book);
    }
}

/// Whether the bookmarks for `kind` are by chapter, so the engine must find where the chapters are.
pub fn wants_chapters(kind: PdfKind, layers: &[&ExportPrefs]) -> bool {
    let mut prefs = builtin(kind);
    for layer in layers {
        prefs = of_kind(layer, kind).over(&prefs);
    }
    // A paperback has no bookmarks whatever was asked.
    kind != PdfKind::Kdp && prefs.bookmarks == Some(BookmarkChoice::Chapters)
}

/// The settings for `kind`, from `layers` (lowest first: your defaults, the book's, this export's)
/// over the built-in ones, with `book`'s own description and keywords for the file's information.
pub fn resolve(kind: PdfKind, layers: &[&ExportPrefs], book: &Book) -> Resolved {
    let mut prefs = builtin(kind);
    for layer in layers {
        prefs = of_kind(layer, kind).over(&prefs);
    }
    let mut notes = Vec::new();
    lock(kind, &mut prefs, &mut notes);

    let chapters = prefs.bookmarks == Some(BookmarkChoice::Chapters);
    let navigation = Navigation {
        bookmarks: match prefs.bookmarks.unwrap_or(BookmarkChoice::None) {
            BookmarkChoice::None => Bookmarks::None,
            BookmarkChoice::Chapters => Bookmarks::Chapters(Vec::new()),
            BookmarkChoice::Spreads => Bookmarks::Spreads,
            BookmarkChoice::Pages => Bookmarks::Pages,
        },
        info: PdfInfo {
            title: prefs.title.clone(),
            authors: prefs.authors.clone(),
            subject: prefs.subject.clone().or_else(|| {
                Some(book.metadata.description.clone()).filter(|d| !d.trim().is_empty())
            }),
            keywords: prefs
                .keywords
                .clone()
                .or_else(|| Some(book.metadata.keywords.clone()).filter(|k| !k.is_empty())),
            language: prefs.language.clone(),
        },
        page_labels: prefs.page_labels.unwrap_or(false),
        links: prefs.links.unwrap_or(false),
        viewer_layout: match prefs.viewer_layout {
            Some(ViewerLayoutChoice::Spreads) => ViewerLayout::Spreads,
            _ => ViewerLayout::Single,
        },
    };
    let pages = PageChoice {
        range: prefs.range.map(|(a, b)| (a as usize, b as usize)),
        cover: match prefs.cover {
            Some(CoverChoice::FrontAndBack) => CoverPages::FrontAndBack,
            Some(CoverChoice::None) => CoverPages::None,
            _ => CoverPages::Front,
        },
        sheets: match prefs.sheets {
            Some(SheetsChoice::Spreads) => Sheets::Spreads,
            _ => Sheets::Pages,
        },
    };
    let ink = InkChoice {
        enabled: prefs.ink == Some(true),
        profile: prefs.profile.clone().filter(|p| !p.trim().is_empty()),
    };
    let shop = ShopOptions {
        bleed: match prefs.bleed {
            Some(BleedChoice::Custom { points }) => ShopBleed::Custom(points),
            _ => ShopBleed::Book,
        },
        slug: prefs.slug,
        // The engine fills this in from the profile file.
        ink: None,
    };
    Resolved {
        extras: PdfExtras {
            navigation,
            pages,
            crop_marks: prefs.crop_marks,
            shop,
            application: None,
            pictures: Some(picture_policy(&prefs)),
            cache: Default::default(),
            cancel: Default::default(),
        },
        ink,
        wants_chapters: chapters,
        notes,
        prefs,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use folio_model::{BookKind, ExportPrefs};
    use folio_presets::{new_book, NewBookSpec};

    fn book() -> Book {
        new_book(&NewBookSpec {
            kind: BookKind::Picture,
            ..NewBookSpec::default()
        })
        .unwrap()
        .book
    }

    fn sharing(prefs: PdfPrefs) -> ExportPrefs {
        let mut p = ExportPrefs::default();
        p.pdf.sharing = prefs;
        p
    }

    fn kdp(prefs: PdfPrefs) -> ExportPrefs {
        let mut p = ExportPrefs::default();
        p.pdf.kdp = prefs;
        p
    }

    #[test]
    fn nothing_chosen_is_the_built_in_settings() {
        let r = resolve(PdfKind::Sharing, &[], &book());
        assert_eq!(r.extras.pictures.unwrap().max_ppi, Some(150.0));
        assert_eq!(r.extras.navigation.bookmarks, Bookmarks::Chapters(vec![]));
        assert!(r.extras.navigation.page_labels);
        assert!(r.wants_chapters && r.notes.is_empty());
        let shop = resolve(PdfKind::PrintShop, &[], &book());
        assert_eq!(shop.extras.crop_marks, Some(true));
        assert_eq!(shop.extras.pictures.unwrap().max_ppi, Some(300.0));
    }

    #[test]
    fn the_highest_layer_that_chose_wins_and_the_rest_fall_through() {
        let user = sharing(PdfPrefs {
            preset: Some(PicturePreset::Small),
            jpeg_quality: Some(50),
            page_labels: Some(false),
            ..PdfPrefs::default()
        });
        let this_book = sharing(PdfPrefs {
            preset: Some(PicturePreset::High),
            ..PdfPrefs::default()
        });
        let once = sharing(PdfPrefs {
            jpeg_quality: Some(70),
            ..PdfPrefs::default()
        });
        let r = resolve(PdfKind::Sharing, &[&user, &this_book, &once], &book());
        let policy = r.extras.pictures.unwrap();
        // The book's preset beats the user's; this export's quality beats the user's.
        assert_eq!(policy.max_ppi, Some(300.0));
        assert_eq!(policy.jpeg_quality, 70);
        // Nothing above the user's layer touched labels.
        assert!(!r.extras.navigation.page_labels);
    }

    #[test]
    fn kdp_is_kept_at_three_hundred_pixels_an_inch_and_says_so() {
        let asked = kdp(PdfPrefs {
            preset: Some(PicturePreset::Standard),
            max_ppi: Some(150),
            jpeg_quality: Some(40),
            ..PdfPrefs::default()
        });
        let r = resolve(PdfKind::Kdp, &[&asked], &book());
        let policy = r.extras.pictures.unwrap();
        assert_eq!(policy.max_ppi, Some(300.0));
        assert_eq!(policy.jpeg_quality, 80);
        assert!(
            r.notes.iter().any(|n| n.contains("300 ppi")),
            "{:?}",
            r.notes
        );
        assert!(r.notes.iter().all(|n| n.starts_with("Changed for KDP")));
    }

    #[test]
    fn kdp_carries_nothing_but_the_title_and_author_and_is_the_whole_book() {
        let asked = kdp(PdfPrefs {
            bookmarks: Some(BookmarkChoice::Pages),
            page_labels: Some(true),
            subject: Some("no".into()),
            range: Some((2, 3)),
            crop_marks: Some(true),
            ..PdfPrefs::default()
        });
        let r = resolve(PdfKind::Kdp, &[&asked], &book());
        assert_eq!(r.extras.navigation.bookmarks, Bookmarks::None);
        assert!(!r.extras.navigation.page_labels);
        assert_eq!(r.extras.pages, PageChoice::default());
        assert_eq!(r.extras.crop_marks, Some(false));
        assert!(r.notes.len() >= 3);
    }

    #[test]
    fn the_sharing_pdf_has_links_unless_turned_off_and_a_press_file_never_does() {
        assert!(
            resolve(PdfKind::Sharing, &[], &book())
                .extras
                .navigation
                .links
        );
        let mut off = ExportPrefs::default();
        off.pdf.sharing.links = Some(false);
        assert!(
            !resolve(PdfKind::Sharing, &[&off], &book())
                .extras
                .navigation
                .links
        );
        for kind in [PdfKind::Kdp, PdfKind::PrintShop] {
            assert!(!resolve(kind, &[], &book()).extras.navigation.links);
        }
    }

    #[test]
    fn the_print_shop_keeps_bookmarks_and_marks_but_not_links_or_page_choices() {
        let mut p = ExportPrefs::default();
        p.pdf.print_shop = PdfPrefs {
            bookmarks: Some(BookmarkChoice::Pages),
            links: Some(true),
            crop_marks: Some(false),
            bleed: Some(BleedChoice::Custom { points: 9.0 }),
            sheets: Some(SheetsChoice::Spreads),
            ..PdfPrefs::default()
        };
        let r = resolve(PdfKind::PrintShop, &[&p], &book());
        assert_eq!(r.extras.navigation.bookmarks, Bookmarks::Pages);
        assert_eq!(r.extras.crop_marks, Some(false));
        assert_eq!(r.extras.shop.bleed, ShopBleed::Custom(9.0));
        assert_eq!(r.extras.pages.sheets, Sheets::Pages);
        assert_eq!(r.prefs.links, Some(false));
    }

    #[test]
    fn the_file_says_the_books_description_and_keywords_unless_told_otherwise() {
        let mut b = book();
        b.metadata.description = "An otter".into();
        b.metadata.keywords = vec!["otter".into()];
        let r = resolve(PdfKind::Sharing, &[], &b);
        assert_eq!(
            r.extras.navigation.info.subject.as_deref(),
            Some("An otter")
        );
        let told = sharing(PdfPrefs {
            subject: Some("Mine".into()),
            ..PdfPrefs::default()
        });
        let r = resolve(PdfKind::Sharing, &[&told], &b);
        assert_eq!(r.extras.navigation.info.subject.as_deref(), Some("Mine"));
        assert_eq!(
            r.extras.navigation.info.keywords,
            Some(vec!["otter".into()])
        );
    }
}
