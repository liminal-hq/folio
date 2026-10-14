// Tests for reading a book's styles
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { describe, expect, it } from 'vitest';
import { defaultTextStyle, styleChoices } from './styleChoices';

describe('styleChoices', () => {
	it('lists the styles with their names and sizes, in file order', () => {
		const out = styleChoices({
			styles: { story: { name: 'Story text', sizePt: 13 }, caption: { sizePt: 9 } },
		});
		expect(out).toEqual([
			{ id: 'story', name: 'Story text', size: 13 },
			{ id: 'caption', name: 'caption', size: 9 },
		]);
	});
	it('is empty for a book with no style file', () => {
		expect(styleChoices(null)).toEqual([]);
		expect(styleChoices({})).toEqual([]);
	});
});

describe('defaultTextStyle', () => {
	const of = (...ids: string[]) =>
		styleChoices({ styles: Object.fromEntries(ids.map((i) => [i, {}])) });
	it('prefers body, then story, then the first style', () => {
		expect(defaultTextStyle(of('byline', 'body', 'story'))).toBe('body');
		expect(defaultTextStyle(of('byline', 'story'))).toBe('story');
		expect(defaultTextStyle(of('byline', 'caption'))).toBe('byline');
		expect(defaultTextStyle([])).toBe('body');
	});
});
