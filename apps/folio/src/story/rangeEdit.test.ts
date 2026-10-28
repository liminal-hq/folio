// Tests for turning an edited stretch of paragraphs into intents
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { describe, expect, it } from 'vitest';
import { boxOf, linesOf, rangeIntents } from './rangeEdit';

const para = (text: string) => ({
	inline: [{ text, style: null, bold: false, italic: false }],
});

describe('rangeEdit', () => {
	it('puts a line in the box for each paragraph and reads them back', () => {
		const paragraphs = [para('One'), para('Two'), para('Three')];
		expect(boxOf(paragraphs, 1, 2)).toBe('Two\nThree');
		expect(linesOf('Two\r\nThree\nFour')).toEqual(['Two', 'Three', 'Four']);
	});

	it('changes paragraphs in place when the number of lines is the same', () => {
		expect(rangeIntents('fl', 4, ['a', 'b', 'c'], ['a', 'B', 'c'])).toEqual([
			{ type: 'SetParagraphText', flow: 'fl', para: 5, text: 'B' },
		]);
		expect(rangeIntents('fl', 4, ['a'], ['a'])).toEqual([]);
	});

	it('splices when a line is added, keeping the style of the paragraph it stands in for', () => {
		const intents = rangeIntents('fl', 2, ['a', 'b'], ['a', 'x', 'b'], ['h1', 'body']);
		expect(intents).toEqual([
			{
				type: 'SpliceStory',
				flow: 'fl',
				at: 3,
				remove: 0,
				insert: [
					{
						style: 'body',
						inline: [{ text: 'x', style: null, bold: false, italic: false }],
					},
				],
			},
		]);
	});

	it('splices when lines are taken away', () => {
		const [splice] = rangeIntents('fl', 0, ['a', 'b', 'c'], ['a', 'c']);
		expect(splice).toMatchObject({ type: 'SpliceStory', at: 1, remove: 1, insert: [] });
	});

	it('replaces the stretch that differs when a line is both changed and split', () => {
		const [splice] = rangeIntents('fl', 0, ['a', 'bc'], ['a', 'b', 'c']);
		expect(splice).toMatchObject({ at: 1, remove: 1 });
		expect((splice as { insert: unknown[] }).insert).toHaveLength(2);
	});
});
