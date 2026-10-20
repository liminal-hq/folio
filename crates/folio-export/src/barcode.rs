// An EAN-13 barcode for an ISBN, as the bars a cover can draw
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

/// How the six left-hand digits are coded for each first digit: `false` is the L set, `true` the G set.
const PARITY: [[bool; 6]; 10] = [
    [false, false, false, false, false, false],
    [false, false, true, false, true, true],
    [false, false, true, true, false, true],
    [false, false, true, true, true, false],
    [false, true, false, false, true, true],
    [false, true, true, false, false, true],
    [false, true, true, true, false, false],
    [false, true, false, true, false, true],
    [false, true, false, true, true, false],
    [false, true, true, false, true, false],
];

/// The L-set pattern of each digit, seven modules, the left bit first.
const L: [u8; 10] = [
    0b0001101, 0b0011001, 0b0010011, 0b0111101, 0b0100011, 0b0110001, 0b0101111, 0b0111011,
    0b0110111, 0b0001011,
];

fn bits(pattern: u8) -> [bool; 7] {
    std::array::from_fn(|i| pattern >> (6 - i) & 1 == 1)
}

/// The 95 modules of the EAN-13 for `isbn` (hyphens and spaces allowed), `true` for a bar. `None`
/// unless it is thirteen digits with a correct check digit.
pub fn ean13(isbn: &str) -> Option<[bool; 95]> {
    let digits: Vec<usize> = isbn
        .chars()
        .filter(|c| !matches!(c, '-' | ' '))
        .map(|c| c.to_digit(10).map(|d| d as usize))
        .collect::<Option<_>>()?;
    if digits.len() != 13 {
        return None;
    }
    let sum: usize = digits[..12]
        .iter()
        .enumerate()
        .map(|(i, d)| d * if i % 2 == 1 { 3 } else { 1 })
        .sum();
    if (10 - sum % 10) % 10 != digits[12] {
        return None;
    }
    let mut modules = Vec::with_capacity(95);
    modules.extend([true, false, true]);
    for (i, &d) in digits[1..7].iter().enumerate() {
        let mut m = bits(L[d]);
        if PARITY[digits[0]][i] {
            // The G set is the R set read backwards, and R is L with every module flipped.
            m = bits(!L[d] & 0x7f);
            m.reverse();
        }
        modules.extend(m);
    }
    modules.extend([false, true, false, true, false]);
    for &d in &digits[7..] {
        modules.extend(bits(!L[d] & 0x7f));
    }
    modules.extend([true, false, true]);
    modules.try_into().ok()
}

/// How the five digits of an add-on are coded, by the check digit worked out from them: `true` is
/// the G set, `false` the L set.
const ADDON_PARITY: [[bool; 5]; 10] = [
    [true, true, false, false, false],
    [true, false, true, false, false],
    [true, false, false, true, false],
    [true, false, false, false, true],
    [false, true, true, false, false],
    [false, false, true, true, false],
    [false, false, false, true, true],
    [false, true, false, true, false],
    [false, true, false, false, true],
    [false, false, true, false, true],
];

/// The 47 modules of the EAN-5 price add-on for five digits: `90000` for no price, `5` and four
/// digits of US dollars, `6` and four of Canadian. `None` unless it is five digits.
pub fn ean5(digits: &str) -> Option<[bool; 47]> {
    let d: Vec<usize> = digits
        .chars()
        .map(|c| c.to_digit(10).map(|n| n as usize))
        .collect::<Option<_>>()?;
    if d.len() != 5 {
        return None;
    }
    let check = (3 * (d[0] + d[2] + d[4]) + 9 * (d[1] + d[3])) % 10;
    let mut modules = Vec::with_capacity(47);
    modules.extend([true, false, true, true]);
    for (i, &digit) in d.iter().enumerate() {
        if i > 0 {
            modules.extend([false, true]);
        }
        let mut m = bits(L[digit]);
        if ADDON_PARITY[check][i] {
            m = bits(!L[digit] & 0x7f);
            m.reverse();
        }
        modules.extend(m);
    }
    modules.try_into().ok()
}

/// The runs of bars in `modules` as `(first module, how many)`.
pub fn bars(modules: &[bool]) -> Vec<(usize, usize)> {
    let mut runs = Vec::new();
    let mut at = 0;
    while at < modules.len() {
        if modules[at] {
            let start = at;
            while at < modules.len() && modules[at] {
                at += 1;
            }
            runs.push((start, at - start));
        } else {
            at += 1;
        }
    }
    runs
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Reads the digits back from the modules, as a scanner would.
    fn decode(m: &[bool; 95]) -> String {
        let seven = |from: usize| -> u8 {
            m[from..from + 7]
                .iter()
                .fold(0, |a, &b| a << 1 | u8::from(b))
        };
        let mut left = Vec::new();
        let mut parity = Vec::new();
        for i in 0..6 {
            let p = seven(3 + i * 7);
            if let Some(d) = L.iter().position(|&x| x == p) {
                left.push(d);
                parity.push(false);
            } else {
                let mut r = bits(p);
                r.reverse();
                let flipped = r.iter().fold(0u8, |a, &b| a << 1 | u8::from(!b));
                left.push(L.iter().position(|&x| x == flipped).expect("a G pattern"));
                parity.push(true);
            }
        }
        let first = PARITY.iter().position(|p| p[..] == parity[..]).unwrap();
        let right: Vec<usize> = (0..6)
            .map(|i| {
                let p = seven(50 + i * 7);
                L.iter()
                    .position(|&x| !x & 0x7f == p)
                    .expect("an R pattern")
            })
            .collect();
        let mut out = first.to_string();
        out.extend(
            left.iter()
                .chain(&right)
                .map(|d| char::from(b'0' + *d as u8)),
        );
        out
    }

    #[test]
    fn an_isbn_becomes_95_modules_that_read_back_as_the_same_digits() {
        for isbn in ["978-0-306-40615-7", "9791090636071", "978 1 4028 9462 6"] {
            let m = ean13(isbn).unwrap();
            assert_eq!(&m[..3], &[true, false, true]);
            assert_eq!(&m[45..50], &[false, true, false, true, false]);
            assert_eq!(&m[92..], &[true, false, true]);
            let plain: String = isbn.chars().filter(|c| c.is_ascii_digit()).collect();
            assert_eq!(decode(&m), plain, "{isbn}");
        }
    }

    #[test]
    fn a_price_addon_is_47_modules_with_its_digits_coded_by_the_check_digit() {
        let m = ean5("90000").unwrap();
        assert_eq!(&m[..4], &[true, false, true, true]);
        // Digits are seven modules wide with a 01 between them: 4 + 7 + 5 * ... = 47.
        assert_eq!(m.len(), 47);
        // 5 + 1999: the check digit is (3 * (5 + 9 + 9) + 9 * (1 + 9)) % 10 = 9, coded LLGLG.
        let us = ean5("51999").unwrap();
        let digit = |at: usize| -> [bool; 7] { us[4 + at * 9..4 + at * 9 + 7].try_into().unwrap() };
        assert_eq!(digit(0), bits(L[5]));
        assert_eq!(digit(1), bits(L[1]));
        let mut g = bits(!L[9] & 0x7f);
        g.reverse();
        assert_eq!(digit(2), g);
        assert!(ean5("9000").is_none());
        assert!(ean5("9000a").is_none());
    }

    #[test]
    fn a_wrong_check_digit_or_length_is_no_barcode() {
        assert!(ean13("978-0-306-40615-8").is_none());
        assert!(ean13("97803064061").is_none());
        assert!(ean13("97803064061ab").is_none());
    }

    #[test]
    fn bars_are_the_runs_of_dark_modules() {
        assert_eq!(bars(&[true, true, false, true]), vec![(0, 2), (3, 1)]);
    }
}
