// Tests the plain-string mirror of an edit to words
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { describe, expect, it } from 'vitest';
import {
	caretAfterText,
	clampPos,
	flatOffset,
	paragraphsOf,
	replaceInTexts,
	textBetween,
} from './edit';

const at = (para: number, offset: number) => ({ para, offset });

describe('paragraphsOf', () => {
	it('splits the words of every run at the line breaks', () => {
		const content = {
			inline: [
				{ text: 'ab', style: null, bold: false, italic: false },
				{ text: 'c\nd', style: null, bold: true, italic: false },
			],
		};
		expect(paragraphsOf(content)).toEqual(['abc', 'd']);
		expect(paragraphsOf(null)).toEqual(['']);
	});
});

describe('replaceInTexts', () => {
	it('types, deletes across paragraphs and splits with a line break', () => {
		const texts = ['one', 'two', 'three'];
		expect(replaceInTexts(texts, at(0, 3), at(0, 3), 'X')).toEqual(['oneX', 'two', 'three']);
		expect(replaceInTexts(texts, at(0, 1), at(2, 2), '')).toEqual(['oree']);
		expect(replaceInTexts(texts, at(1, 1), at(1, 1), '\n')).toEqual(['one', 't', 'wo', 'three']);
		expect(replaceInTexts(['a\u{1f600}b'], at(0, 2), at(0, 2), 'z')).toEqual(['a\u{1f600}zb']);
	});

	it('gives the caret where Rust does', () => {
		expect(caretAfterText(at(0, 1), 'xy')).toEqual(at(0, 3));
		expect(caretAfterText(at(0, 1), 'x\nyz')).toEqual(at(1, 2));
		expect(caretAfterText(at(2, 5), '\n')).toEqual(at(3, 0));
	});
});

describe('positions', () => {
	it('counts line breaks in a flat offset and reads text between places', () => {
		expect(flatOffset(['ab', 'cd'], at(1, 1))).toBe(4);
		expect(textBetween(['ab', 'cd', 'ef'], at(0, 1), at(2, 1))).toBe('b\ncd\ne');
		expect(textBetween(['abcd'], at(0, 1), at(0, 3))).toBe('bc');
	});

	it('pulls a place inside the words', () => {
		expect(clampPos(['ab'], at(5, 9))).toEqual(at(0, 2));
		expect(clampPos(['ab', 'c'], at(1, 9))).toEqual(at(1, 1));
	});
});
