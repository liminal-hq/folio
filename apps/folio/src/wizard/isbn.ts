// ISBN-13 checking, so a mistyped number is caught as it is typed
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

/** Mirrors `is_valid_isbn13` in `crates/folio-presets`: 13 digits (hyphens and spaces allowed), starting 978 or 979, with a correct check digit. */
export function isValidIsbn13(text: string): boolean {
	const digits = [...text].filter((c) => c >= '0' && c <= '9').map(Number);
	if (digits.length !== 13) return false;
	if (digits[0] !== 9 || digits[1] !== 7 || (digits[2] !== 8 && digits[2] !== 9)) return false;
	const sum = digits.slice(0, 12).reduce((total, d, i) => total + d * (i % 2 === 1 ? 3 : 1), 0);
	return (10 - (sum % 10)) % 10 === digits[12];
}
