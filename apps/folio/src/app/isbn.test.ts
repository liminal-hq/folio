// Tests for reading a typed ISBN
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { describe, expect, it } from 'vitest';
import { isValidIsbn13, readIsbn } from './isbn';

describe('readIsbn', () => {
	it('reads a good thirteen-digit ISBN with or without hyphens', () => {
		expect(readIsbn('978-0-306-40615-7')).toEqual({ kind: 'ok', isbn: '9780306406157' });
		expect(readIsbn(' 9780306406157 ')).toEqual({ kind: 'ok', isbn: '9780306406157' });
		expect(isValidIsbn13('9780306406157')).toBe(true);
	});

	it('turns ten digits into thirteen', () => {
		expect(readIsbn('0-306-40615-2')).toEqual({ kind: 'ok', isbn: '9780306406157' });
	});

	it('offers the right last digit when only that one is wrong', () => {
		expect(readIsbn('9780306406158')).toEqual({ kind: 'checkDigit', fixed: '9780306406157' });
	});

	it('says what is wrong otherwise', () => {
		expect(readIsbn('')).toEqual({ kind: 'empty' });
		expect(readIsbn('97803064')).toEqual({ kind: 'bad', reason: 'length' });
		expect(readIsbn('97x0306406157')).toEqual({ kind: 'bad', reason: 'length' });
		expect(readIsbn('9770306406154')).toEqual({ kind: 'bad', reason: 'prefix' });
	});
});
