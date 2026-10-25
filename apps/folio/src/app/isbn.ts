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

/** What typing an ISBN comes to: its digits as a 13-digit number, or why it is not one. */
export type IsbnReading =
	| { kind: 'empty' }
	| { kind: 'ok'; isbn: string }
	/** Thirteen digits that start right but whose last digit is wrong: `fixed` has the right one. */
	| { kind: 'checkDigit'; fixed: string }
	| { kind: 'bad'; reason: 'length' | 'prefix' | 'check' };

/** The check digit that follows the first twelve digits of an ISBN-13. */
const check13 = (twelve: number[]) =>
	(10 - (twelve.reduce((total, digit, i) => total + digit * (i % 2 === 1 ? 3 : 1), 0) % 10)) % 10;

/**
 * Reads an ISBN as typed, with or without hyphens, 13 digits or 10. A ten-digit ISBN is made
 * thirteen by putting 978 in front and working out the new check digit.
 */
export function readIsbn(text: string): IsbnReading {
	const chars = [...text].filter((c) => !/[\s-]/.test(c));
	if (chars.length === 0) return { kind: 'empty' };
	if (chars.length === 10) {
		const nine = chars.slice(0, 9);
		if (!nine.every((c) => c >= '0' && c <= '9')) return { kind: 'bad', reason: 'length' };
		const twelve = [9, 7, 8, ...nine.map(Number)];
		return { kind: 'ok', isbn: [...twelve, check13(twelve)].join('') };
	}
	if (chars.length !== 13 || !chars.every((c) => c >= '0' && c <= '9')) {
		return { kind: 'bad', reason: 'length' };
	}
	const digits = chars.map(Number);
	if (digits[0] !== 9 || digits[1] !== 7 || (digits[2] !== 8 && digits[2] !== 9)) {
		return { kind: 'bad', reason: 'prefix' };
	}
	const right = check13(digits.slice(0, 12));
	if (right === digits[12]) return { kind: 'ok', isbn: chars.join('') };
	return { kind: 'checkDigit', fixed: [...digits.slice(0, 12), right].join('') };
}
