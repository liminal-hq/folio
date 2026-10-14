// Where a new book is saved by default
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::path::{Path, PathBuf};

const FALLBACK: &str = "Untitled";
const LONGEST: usize = 80;

/// A title as a file or folder name: no characters a file system refuses, no leading or trailing
/// dots or spaces, runs of spaces collapsed, and a sensible length.
fn file_name(title: &str) -> String {
    let cleaned: String = title
        .chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => ' ',
            c if c.is_control() => ' ',
            c => c,
        })
        .collect();
    let collapsed = cleaned.split_whitespace().collect::<Vec<_>>().join(" ");
    let trimmed: String = collapsed
        .trim_matches(|c| c == '.' || c == ' ')
        .chars()
        .take(LONGEST)
        .collect();
    let trimmed = trimmed.trim_matches(|c| c == '.' || c == ' ').to_owned();
    if trimmed.is_empty() {
        return FALLBACK.to_owned();
    }
    // Names Windows reserves: a folder cannot be called CON.
    let upper = trimmed.to_ascii_uppercase();
    let reserved = ["CON", "PRN", "AUX", "NUL"].contains(&upper.as_str())
        || (upper.len() == 4
            && (upper.starts_with("COM") || upper.starts_with("LPT"))
            && upper.ends_with(|c: char| c.is_ascii_digit()));
    if reserved {
        format!("{trimmed} book")
    } else {
        trimmed
    }
}

/// `<books_dir>/<Title>/<Title>.folio`: where Save as… starts for a book that has never been saved.
/// Each book gets a folder, which is also where its exports go.
pub fn suggest_save_path(title: &str, books_dir: &Path) -> PathBuf {
    let name = file_name(title);
    books_dir.join(&name).join(format!("{name}.folio"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_book_gets_its_own_folder_named_for_its_title() {
        let path = suggest_save_path("The Otter Who Couldn't Sleep", Path::new("/home/mom/Books"));
        assert_eq!(
            path,
            Path::new(
                "/home/mom/Books/The Otter Who Couldn't Sleep/The Otter Who Couldn't Sleep.folio"
            )
        );
    }

    #[test]
    fn characters_a_file_system_refuses_are_removed() {
        let path = suggest_save_path("What? Who: me/you", Path::new("/b"));
        assert_eq!(path, Path::new("/b/What Who me you/What Who me you.folio"));
    }

    #[test]
    fn an_empty_or_dotty_title_becomes_untitled() {
        assert_eq!(
            suggest_save_path("", Path::new("/b")),
            Path::new("/b/Untitled/Untitled.folio")
        );
        assert_eq!(
            suggest_save_path("  ...  ", Path::new("/b")),
            Path::new("/b/Untitled/Untitled.folio")
        );
    }

    #[test]
    fn a_title_cannot_climb_out_of_the_books_folder() {
        let path = suggest_save_path("../../etc/passwd", Path::new("/b"));
        assert!(path.starts_with("/b/"));
        assert!(!path.to_string_lossy().contains(".."));
    }

    #[test]
    fn windows_reserved_names_are_avoided_and_long_titles_are_cut() {
        assert_eq!(
            suggest_save_path("con", Path::new("/b")),
            Path::new("/b/con book/con book.folio")
        );
        assert_eq!(
            suggest_save_path("lpt1", Path::new("/b")),
            Path::new("/b/lpt1 book/lpt1 book.folio")
        );
        let long = "x".repeat(300);
        let name = suggest_save_path(&long, Path::new("/b"))
            .parent()
            .unwrap()
            .file_name()
            .unwrap()
            .len();
        assert_eq!(name, 80);
    }
}
