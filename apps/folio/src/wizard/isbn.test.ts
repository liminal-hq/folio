// Tests for ISBN-13 checking, with the same cases as the Rust twin
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { describe, expect, it } from 'vitest';
import { isValidIsbn13 } from './isbn';

describe('isValidIsbn13', () => {
	it('accepts real numbers with or without hyphens', () => {
		expect(isValidIsbn13('9780306406157')).toBe(true);
		expect(isValidIsbn13('978-0-306-40615-7')).toBe(true);
		expect(isValidIsbn13('978 1 4028 9462 6')).toBe(true);
	});

	it('rejects a wrong check digit, length or prefix', () => {
		expect(isValidIsbn13('9780306406158')).toBe(false);
		expect(isValidIsbn13('978030640615')).toBe(false);
		expect(isValidIsbn13('9770306406154')).toBe(false);
		expect(isValidIsbn13('')).toBe(false);
		expect(isValidIsbn13('not an isbn')).toBe(false);
	});
});
