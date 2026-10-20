// ISBN-13 checking, so a mistyped number is caught before it reaches a cover
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

pub use folio_model::is_valid_isbn13;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_real_numbers_with_or_without_hyphens() {
        assert!(is_valid_isbn13("9780306406157"));
        assert!(is_valid_isbn13("978-0-306-40615-7"));
        assert!(is_valid_isbn13("978 1 4028 9462 6"));
    }

    #[test]
    fn rejects_a_wrong_check_digit_length_or_prefix() {
        assert!(!is_valid_isbn13("9780306406158"));
        assert!(!is_valid_isbn13("978030640615"));
        assert!(!is_valid_isbn13("9770306406154"));
        assert!(!is_valid_isbn13(""));
        assert!(!is_valid_isbn13("not an isbn"));
    }
}
