// Bookmarks, page labels and file information for a PDF
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! What a reader of a PDF navigates by: the bookmarks panel, the numbers a viewer shows for a page
//! (`iv`, `12`, not "page 16 of 200"), and the file's subject, keywords and language. The paperback
//! targets ask for none of it (D-33), so `Navigation::for_target` drops it for them.

use std::num::NonZeroU32;

use krilla::destination::XyzDestination;
use krilla::geom::Point;
use krilla::metadata::{Metadata, PageLayout};
use krilla::outline::{Outline, OutlineNode};
use krilla::page::{NumberingStyle, PageLabel};

use crate::pdf::PdfTarget;

/// A chapter of the book and where its first words are, as the engine works it out from layout.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Chapter {
    pub title: String,
    /// The spread the chapter starts on, by id.
    pub spread: String,
    /// Which page of that spread, from 0.
    pub page: usize,
}

/// What the bookmarks panel lists.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Bookmarks {
    #[default]
    None,
    /// One for each chapter, in reading order.
    Chapters(Vec<Chapter>),
    /// One for each spread.
    Spreads,
    /// One for each page.
    Pages,
}

/// How a viewer lays the pages out when the file opens.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ViewerLayout {
    /// Whatever the viewer usually does.
    #[default]
    Single,
    /// Two pages side by side with the first on its own, as in a book.
    Spreads,
}

/// What the file says about itself. `None` leaves a field out; the title and authors are the
/// book's when not given here.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PdfInfo {
    pub title: Option<String>,
    pub authors: Option<Vec<String>>,
    /// The file's subject.
    pub subject: Option<String>,
    pub keywords: Option<Vec<String>>,
    /// The language of the words, such as `en-CA`; the Sharing PDF says the book's when not given.
    pub language: Option<String>,
}

/// The ways round a PDF.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Navigation {
    pub bookmarks: Bookmarks,
    pub info: PdfInfo,
    /// Pages are labelled with the numbers the book shows, in its sections' styles.
    pub page_labels: bool,
    pub viewer_layout: ViewerLayout,
}

impl Navigation {
    /// What `target` allows: a paperback or its cover has nothing but the title and author.
    pub fn for_target(&self, target: PdfTarget) -> Navigation {
        match target {
            PdfTarget::Print | PdfTarget::Cover => Navigation::default(),
            PdfTarget::Sharing | PdfTarget::PrintShop => self.clone(),
        }
    }
}

/// A page of the file and where it came from.
#[derive(Debug, Clone)]
pub struct Origin {
    pub spread: String,
    /// Which page of the spread, from 0.
    pub index: usize,
    /// The number the book shows on the page; empty for a page with none.
    pub label: String,
    /// The name of the section the page is in.
    pub section: String,
}

fn roman_value(text: &str) -> Option<u32> {
    let mut total = 0;
    let mut previous = 0;
    for c in text.chars().rev() {
        let v = match c {
            'i' => 1,
            'v' => 5,
            'x' => 10,
            'l' => 50,
            'c' => 100,
            'd' => 500,
            'm' => 1000,
            _ => return None,
        };
        if v < previous {
            total -= v;
        } else {
            total += v;
            previous = v;
        }
    }
    Some(total)
}

/// The label of a page as a PDF viewer shows it. A page without a number is labelled with its
/// section's name, so the numbers of the pages after it still count from where they should.
pub fn page_label(origin: &Origin) -> Option<PageLabel> {
    if origin.label.is_empty() {
        let prefix = if origin.section.is_empty() {
            "—".to_string()
        } else {
            origin.section.clone()
        };
        return Some(PageLabel::new(None, Some(prefix), None));
    }
    if let Ok(n) = origin.label.parse::<u32>() {
        return Some(PageLabel::new(
            Some(NumberingStyle::Arabic),
            None,
            NonZeroU32::new(n),
        ));
    }
    roman_value(&origin.label)
        .and_then(NonZeroU32::new)
        .map(|n| PageLabel::new(Some(NumberingStyle::LowerRoman), None, Some(n)))
}

/// What a page is called in a list: its number, or its place in the file when it has none.
fn page_name(origin: &Origin, at: usize) -> String {
    if origin.label.is_empty() {
        if origin.section.is_empty() {
            format!("Page {}", at + 1)
        } else {
            origin.section.clone()
        }
    } else {
        format!("Page {}", origin.label)
    }
}

fn node(title: String, page: usize) -> OutlineNode {
    OutlineNode::new(title, XyzDestination::new(page, Point::from_xy(0.0, 0.0)))
}

/// The bookmarks for `pages`, or `None` when there are none to make.
pub fn outline(bookmarks: &Bookmarks, pages: &[Origin]) -> Option<Outline> {
    let mut out = Outline::new();
    let mut any = false;
    match bookmarks {
        Bookmarks::None => {}
        Bookmarks::Chapters(chapters) => {
            for chapter in chapters {
                let at = pages
                    .iter()
                    .position(|p| p.spread == chapter.spread && p.index == chapter.page)
                    .or_else(|| pages.iter().position(|p| p.spread == chapter.spread));
                if let Some(at) = at {
                    out.push_child(node(chapter.title.clone(), at));
                    any = true;
                }
            }
        }
        Bookmarks::Spreads => {
            let mut at = 0;
            while at < pages.len() {
                let spread = &pages[at].spread;
                let last = pages[at..]
                    .iter()
                    .take_while(|p| &p.spread == spread)
                    .count()
                    - 1;
                let (first, end) = (&pages[at], &pages[at + last]);
                let title = match (first.label.as_str(), end.label.as_str()) {
                    ("", _) => page_name(first, at),
                    (a, b) if a == b || b.is_empty() => format!("Page {a}"),
                    (a, b) => format!("Pages {a}–{b}"),
                };
                out.push_child(node(title, at));
                any = true;
                at += last + 1;
            }
        }
        Bookmarks::Pages => {
            for (at, page) in pages.iter().enumerate() {
                out.push_child(node(page_name(page, at), at));
                any = true;
            }
        }
    }
    any.then_some(out)
}

/// The file's information from what the book and the person say, for a file that is `target`.
pub fn metadata(
    navigation: &Navigation,
    target: PdfTarget,
    title: &str,
    authors: &[String],
    language: &str,
) -> Metadata {
    let info = &navigation.info;
    let mut metadata = Metadata::new().title(info.title.clone().unwrap_or_else(|| title.into()));
    let authors = info.authors.clone().unwrap_or_else(|| authors.to_vec());
    if !authors.is_empty() {
        metadata = metadata.authors(authors);
    }
    if let Some(subject) = info.subject.as_ref().filter(|s| !s.trim().is_empty()) {
        metadata = metadata.description(subject.clone());
    }
    if let Some(keywords) = info.keywords.as_ref().filter(|k| !k.is_empty()) {
        metadata = metadata.keywords(keywords.clone());
    }
    match (&info.language, target) {
        (Some(language), _) => metadata = metadata.language(language.clone()),
        (None, PdfTarget::Sharing) => metadata = metadata.language(language.into()),
        _ => {}
    }
    if navigation.viewer_layout == ViewerLayout::Spreads {
        metadata = metadata.page_layout(PageLayout::TwoPageRight);
    }
    metadata
}
