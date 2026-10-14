// ISBN-13 checking, so a mistyped number is caught before it reaches a cover
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

/// Whether `text` is a valid ISBN-13: 13 digits (hyphens and spaces allowed), starting 978 or 979,
/// with a correct check digit.
pub fn is_valid_isbn13(text: &str) -> bool {
    let digits: Vec<u32> = text.chars().filter_map(|c| c.to_digit(10)).collect();
    if digits.len() != 13 || !(digits[..3] == [9, 7, 8] || digits[..3] == [9, 7, 9]) {
        return false;
    }
    let sum: u32 = digits[..12]
        .iter()
        .enumerate()
        .map(|(i, d)| d * if i % 2 == 1 { 3 } else { 1 })
        .sum();
    (10 - sum % 10) % 10 == digits[12]
}

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
