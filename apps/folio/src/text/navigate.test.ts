// Tests moving the caret by grapheme, word and line
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { describe, expect, it } from 'vitest';
import { layoutOf } from './fixtureLayout';
import { moveGrapheme, moveLineEdge, moveVertical, moveWord, wordAt } from './navigate';

const texts = ['the quick brown fox jumps', 'end'];
const layout = layoutOf([['the quick brown', 'fox jumps'], ['end']]);

describe('moveGrapheme', () => {
	it('steps over a base and its mark as one', () => {
		expect(moveGrapheme(['éx'], { para: 0, offset: 0 }, 1)).toEqual({ para: 0, offset: 2 });
		expect(moveGrapheme(['éx'], { para: 0, offset: 2 }, -1)).toEqual({ para: 0, offset: 0 });
	});

	it('crosses a paragraph break and stops at the ends of the words', () => {
		expect(moveGrapheme(texts, { para: 0, offset: 25 }, 1)).toEqual({ para: 1, offset: 0 });
		expect(moveGrapheme(texts, { para: 1, offset: 0 }, -1)).toEqual({ para: 0, offset: 25 });
		expect(moveGrapheme(texts, { para: 0, offset: 0 }, -1)).toEqual({ para: 0, offset: 0 });
		expect(moveGrapheme(texts, { para: 1, offset: 3 }, 1)).toEqual({ para: 1, offset: 3 });
	});
});

describe('moveWord', () => {
	it('goes to the end of the next word and the start of the last', () => {
		expect(moveWord(texts, { para: 0, offset: 0 }, 1)).toEqual({ para: 0, offset: 3 });
		expect(moveWord(texts, { para: 0, offset: 3 }, 1)).toEqual({ para: 0, offset: 9 });
		expect(moveWord(texts, { para: 0, offset: 9 }, -1)).toEqual({ para: 0, offset: 4 });
		expect(moveWord(texts, { para: 0, offset: 4 }, -1)).toEqual({ para: 0, offset: 0 });
	});

	it('crosses paragraphs', () => {
		expect(moveWord(texts, { para: 0, offset: 25 }, 1)).toEqual({ para: 1, offset: 0 });
		expect(moveWord(texts, { para: 1, offset: 0 }, -1)).toEqual({ para: 0, offset: 20 });
	});
});

describe('wordAt', () => {
	it('selects the word, or a space', () => {
		expect(wordAt('the quick', 5)).toEqual({ start: 4, end: 9 });
		expect(wordAt('the quick', 3)).toEqual({ start: 3, end: 4 });
		expect(wordAt('the quick', 9)).toEqual({ start: 4, end: 9 });
	});
});

describe('moveLineEdge', () => {
	it('goes to where the words on the line start and end, not past the space it broke at', () => {
		expect(moveLineEdge(layout, { para: 0, offset: 7 }, 'home')).toEqual({ para: 0, offset: 0 });
		expect(moveLineEdge(layout, { para: 0, offset: 7 }, 'end')).toEqual({ para: 0, offset: 15 });
		expect(moveLineEdge(layout, { para: 0, offset: 20 }, 'home')).toEqual({ para: 0, offset: 16 });
		expect(moveLineEdge(layout, { para: 0, offset: 20 }, 'end')).toEqual({ para: 0, offset: 25 });
	});
});

describe('moveVertical', () => {
	it('keeps to the x it started at across short lines', () => {
		const down = moveVertical(layout, { para: 0, offset: 12 }, 1);
		expect(down.pos).toEqual({ para: 0, offset: 25 });
		expect(down.goalX).toBe(120);
		const further = moveVertical(layout, down.pos, 1, down.goalX);
		expect(further.pos).toEqual({ para: 1, offset: 3 });
		const back = moveVertical(layout, further.pos, -1, down.goalX);
		expect(back.pos).toEqual({ para: 0, offset: 25 });
		const up = moveVertical(layout, back.pos, -1, down.goalX);
		expect(up.pos).toEqual({ para: 0, offset: 12 });
	});

	it('goes to the start of the text on the first line and the end on the last', () => {
		expect(moveVertical(layout, { para: 0, offset: 5 }, -1).pos).toEqual({ para: 0, offset: 0 });
		expect(moveVertical(layout, { para: 1, offset: 1 }, 1).pos).toEqual({ para: 1, offset: 3 });
	});
});
