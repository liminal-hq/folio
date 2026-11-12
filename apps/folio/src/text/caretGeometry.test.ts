// Tests finding a caret, a selection and a click in the lines Rust laid out
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { describe, expect, it } from 'vitest';
import {
	caretRect,
	hitTest,
	lineOf,
	lineStops,
	paragraphLength,
	selectionRects,
	xOnLine,
} from './caretGeometry';
import { CHAR_W, layoutOf, LINE_H } from './fixtureLayout';

// "the quick brown" / "fox jumps" is one paragraph broken after "brown ", then a short second one.
const layout = layoutOf([['the quick brown', 'fox jumps'], ['end']]);

describe('stops', () => {
	it('gives a boundary for every code point, the gaps between words included', () => {
		const stops = lineStops(layout.lines[0]!);
		expect(stops.map((s) => s.offset)).toEqual(Array.from({ length: 16 }, (_, i) => i));
		expect(stops.map((s) => s.x)).toEqual(Array.from({ length: 16 }, (_, i) => i * CHAR_W));
	});

	it('puts a line start after the space the previous line broke at', () => {
		expect(layout.lines[0]!.end).toBe(16);
		expect(layout.lines[1]!.start).toBe(16);
		expect(paragraphLength(layout, 0)).toBe(25);
	});
});

describe('caretRect', () => {
	it('is as tall as its line and at the boundary', () => {
		expect(caretRect(layout, { para: 0, offset: 4 })).toEqual({ x: 40, y: 0, w: 0, h: LINE_H });
		expect(caretRect(layout, { para: 0, offset: 18 })).toEqual({
			x: 20,
			y: LINE_H,
			w: 0,
			h: LINE_H,
		});
	});

	it('puts the boundary of a wrap at the start of the later line, or the end of the earlier by affinity', () => {
		const pos = { para: 0, offset: 16 };
		expect(caretRect(layout, pos, 'after')!.y).toBe(LINE_H);
		expect(lineOf(layout, pos, 'before')).toBe(layout.lines[0]);
		// Before the space a line ends on, the caret is at the end of the last word.
		expect(xOnLine(layout.lines[0]!, 15)).toBe(150);
		expect(xOnLine(layout.lines[0]!, 16)).toBe(150);
	});

	it('finds the end of the last line of a paragraph and of an empty paragraph', () => {
		expect(caretRect(layout, { para: 0, offset: 25 })!.x).toBe(90);
		const blank = layoutOf([[''], ['a']]);
		expect(caretRect(blank, { para: 0, offset: 0 })).toEqual({ x: 0, y: 0, w: 0, h: LINE_H });
	});
});

describe('hitTest', () => {
	it('finds the nearest boundary on the line under the point', () => {
		expect(hitTest(layout, 44, 5)).toEqual({ para: 0, offset: 4 });
		expect(hitTest(layout, 46, 5)).toEqual({ para: 0, offset: 5 });
		expect(hitTest(layout, 22, LINE_H + 5)).toEqual({ para: 0, offset: 18 });
		expect(hitTest(layout, 6, 2 * LINE_H + 5)).toEqual({ para: 1, offset: 1 });
	});

	it('goes to the ends for a click past them, and to the nearest line for one above or below', () => {
		expect(hitTest(layout, 999, 5)).toEqual({ para: 0, offset: 15 });
		expect(hitTest(layout, -20, 5)).toEqual({ para: 0, offset: 0 });
		expect(hitTest(layout, 5, -50)).toEqual({ para: 0, offset: 0 });
		expect(hitTest(layout, 999, 999)).toEqual({ para: 1, offset: 3 });
	});

	it('snaps to a grapheme boundary when asked', () => {
		expect(hitTest(layout, 44, 5, (_para, offset) => offset - 1)).toEqual({ para: 0, offset: 3 });
	});

	it('has nothing to hit in an empty layout', () => {
		expect(hitTest({ frame: 'f', lines: [], overset: false }, 0, 0)).toBeUndefined();
	});
});

describe('selectionRects', () => {
	it('is one box on one line', () => {
		expect(selectionRects(layout, { para: 0, offset: 4 }, { para: 0, offset: 9 })).toEqual([
			{ x: 40, y: 0, w: 50, h: LINE_H },
		]);
	});

	it('is a box for each line it crosses, and the order of the ends does not matter', () => {
		const a = selectionRects(layout, { para: 0, offset: 10 }, { para: 0, offset: 20 });
		const b = selectionRects(layout, { para: 0, offset: 20 }, { para: 0, offset: 10 });
		expect(a).toEqual(b);
		expect(a).toEqual([
			{ x: 100, y: 0, w: 50, h: LINE_H },
			{ x: 0, y: LINE_H, w: 40, h: LINE_H },
		]);
	});

	it('shows a line break inside a selection as a little extra on the paragraphs last line', () => {
		const rects = selectionRects(layout, { para: 0, offset: 20 }, { para: 1, offset: 1 });
		expect(rects).toHaveLength(2);
		expect(rects[0]!.w).toBeGreaterThan(50);
		expect(rects[1]).toEqual({ x: 0, y: 2 * LINE_H, w: 10, h: LINE_H });
	});

	it('is nothing when the ends are the same', () => {
		expect(selectionRects(layout, { para: 0, offset: 4 }, { para: 0, offset: 4 })).toEqual([]);
	});
});
