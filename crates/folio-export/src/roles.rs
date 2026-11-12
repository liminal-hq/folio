// What each paragraph style is in a book's structure
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! The book's named styles say what its words are: a title, a heading, a caption, a note, or
//! something that is only on the page to be looked at (a page number, a running head). The EPUB
//! builds its markup from this, and the tagged PDF builds its structure tree from it, so the two
//! agree on what a book is made of.

/// What a style is in the book's structure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    /// A title that opens a chapter or the book.
    Title,
    Subheading,
    Paragraph,
    Caption,
    Epigraph,
    Poem,
    Note,
    /// Words in a speech bubble or on a label: read where they are on the page, with no place in a
    /// reflowable book.
    Speech,
    /// Only meaningful on a printed page: page numbers, running heads, the spine. See `printed_of`
    /// for which.
    Skip,
}

/// Words that are on the page as furniture, not as part of the story.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Printed {
    PageNumber,
    RunningHead,
    Spine,
    Ornament,
}

/// What the furniture style `style` is, or `None` for a style that carries the story. The speech
/// bubble and label styles are the story's words, set apart, and are not furniture.
pub fn printed_of(style: &str) -> Option<Printed> {
    match style {
        "folio" => Some(Printed::PageNumber),
        "rh" => Some(Printed::RunningHead),
        "spine" => Some(Printed::Spine),
        "ornament" => Some(Printed::Ornament),
        _ => None,
    }
}

/// The role of the paragraph style `style`.
pub fn role_of(style: &str) -> Role {
    if printed_of(style).is_some() {
        return Role::Skip;
    }
    match style {
        "display" | "title" | "h1big" | "h1" => Role::Title,
        "numeral" | "h2" => Role::Subheading,
        "caption" => Role::Caption,
        "epigraph" => Role::Epigraph,
        "poem" => Role::Poem,
        "callout" | "footnote" => Role::Note,
        "bubble" | "label" => Role::Speech,
        _ => Role::Paragraph,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn furniture_is_skipped_and_the_rest_has_a_role() {
        for style in ["folio", "rh", "spine", "ornament"] {
            assert_eq!(role_of(style), Role::Skip, "{style}");
        }
        assert_eq!(role_of("bubble"), Role::Speech);
        assert_eq!(role_of("label"), Role::Speech);
        assert_eq!(printed_of("folio"), Some(Printed::PageNumber));
        assert_eq!(printed_of("bubble"), None);
        assert_eq!(role_of("display"), Role::Title);
        assert_eq!(role_of("numeral"), Role::Subheading);
        assert_eq!(role_of("epigraph"), Role::Epigraph);
        assert_eq!(role_of("callout"), Role::Note);
        assert_eq!(role_of("body"), Role::Paragraph);
        assert_eq!(role_of("anything-else"), Role::Paragraph);
    }
}
