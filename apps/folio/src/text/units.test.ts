// Tests counting in code points and finding grapheme and word boundaries
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { describe, expect, it } from 'vitest';
import { boundaries, cpLength, cpSlice, cpToUtf16, snapToGrapheme, utf16ToCp } from './units';

describe('code points', () => {
	it('counts an astral character as one', () => {
		expect(cpLength('a\u{1f600}b')).toBe(3);
		expect('a\u{1f600}b'.length).toBe(4);
	});

	it('converts between code points and UTF-16 indexes', () => {
		const text = 'a\u{1f600}b';
		expect(cpToUtf16(text, 0)).toBe(0);
		expect(cpToUtf16(text, 2)).toBe(3);
		expect(cpToUtf16(text, 9)).toBe(4);
		expect(utf16ToCp(text, 3)).toBe(2);
		expect(cpSlice(text, 1, 2)).toBe('\u{1f600}');
		expect(cpSlice(text, 2)).toBe('b');
	});
});

describe('boundaries', () => {
	it('keeps a base and its combining mark, and a joined emoji, as one grapheme', () => {
		// e + combining acute, then a family of three joined with zero-width joiners.
		const text = 'é\u{1f469}‍\u{1f469}‍\u{1f467}x';
		expect(boundaries(text, 'grapheme')).toEqual([0, 2, 7, 8]);
	});

	it('finds word boundaries', () => {
		expect(boundaries('one two', 'word')).toEqual([0, 3, 4, 7]);
	});

	it('snaps a position inside a grapheme to its edge', () => {
		const text = 'éx';
		expect(snapToGrapheme(text, 1)).toBe(0);
		expect(snapToGrapheme(text, 1, 'after')).toBe(2);
		expect(snapToGrapheme(text, 2)).toBe(2);
	});
});
