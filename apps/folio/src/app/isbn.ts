// Whether a typed ISBN is one: thirteen digits, 978 or 979 first, and a correct check digit
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

/** The Rust check (`folio_model::is_valid_isbn13`), so the dialog says so before the book is asked. */
export function isValidIsbn13(text: string): boolean {
	const digits = [...text].filter((c) => c >= '0' && c <= '9').map(Number);
	if (digits.length !== 13) return false;
	if (digits[0] !== 9 || digits[1] !== 7 || (digits[2] !== 8 && digits[2] !== 9)) return false;
	const sum = digits
		.slice(0, 12)
		.reduce((total, digit, i) => total + digit * (i % 2 === 1 ? 3 : 1), 0);
	return (10 - (sum % 10)) % 10 === digits[12];
}
