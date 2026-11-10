// Tests copying words with their look and reading them back
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { describe, expect, it } from 'vitest';
import { fromClipboard, paragraphRuns, sliceParagraphs, toHtml, toPlain } from './clipboard';

const run = (text: string, extra = {}) => ({
	text,
	style: null,
	bold: false,
	italic: false,
	...extra,
});
const content = {
	inline: [run('Hello '), run('brave', { bold: true }), run(' world\nsecond', { italic: true })],
};

describe('sliceParagraphs', () => {
	it('keeps the look of each stretch inside the range', () => {
		const slice = sliceParagraphs(content, { para: 0, offset: 3 }, { para: 0, offset: 13 });
		expect(slice).toHaveLength(1);
		expect(slice[0]!.inline.map((r) => [r.text, r.bold, r.italic])).toEqual([
			['lo ', false, false],
			['brave', true, false],
			[' w', false, true],
		]);
	});

	it('takes whole paragraphs across a line break', () => {
		const slice = sliceParagraphs(content, { para: 0, offset: 12 }, { para: 1, offset: 3 });
		expect(toPlain(slice)).toBe('world\nsec');
		expect(slice).toHaveLength(2);
	});

	it('counts code points', () => {
		const emoji = { inline: [run('a\u{1f600}b', { bold: true })] };
		expect(toPlain(sliceParagraphs(emoji, { para: 0, offset: 1 }, { para: 0, offset: 2 }))).toBe(
			'\u{1f600}',
		);
	});
});

describe('formats', () => {
	it('writes HTML other programs read, escaping the words', () => {
		const html = toHtml([
			{ style: null, inline: [run('a<b', { bold: true, colour: '#a23b2a' }), run(' & c', {})] },
		]);
		expect(html).toBe('<p><strong><span style="color:#a23b2a">a&lt;b</span></strong> &amp; c</p>');
	});

	it('reads back what it wrote and refuses anything else', () => {
		const paras = sliceParagraphs(content, { para: 0, offset: 0 }, { para: 1, offset: 6 });
		expect(fromClipboard(JSON.stringify(paras))).toEqual(paras);
		expect(fromClipboard('not json')).toBeNull();
		expect(fromClipboard('[]')).toBeNull();
		expect(fromClipboard('[{"inline":[{"nope":1}]}]')).toBeNull();
		expect(paragraphRuns(null)).toEqual([[]]);
	});
});

describe('underline and strike', () => {
	it('are written as <u> and <s> for other programs', () => {
		const html = toHtml([
			{ style: null, inline: [run('a', { underline: true, strike: true }), run('b')] },
		]);
		expect(html).toBe('<p><u><s>a</s></u>b</p>');
	});
});
