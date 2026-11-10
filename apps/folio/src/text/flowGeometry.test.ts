// Tests the caret crossing from one frame of a story to the next on a spread
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { FrameLayout } from '@liminal-hq/folio-protocol/generated/FrameLayout';
import { describe, expect, it } from 'vitest';
import { caretRect, selectionRects } from './caretGeometry';
import { CHAR_W, LINE_H, layoutOf } from './fixtureLayout';
import { flowGeometry } from './flowGeometry';
import { moveLineEdge, moveVertical } from './navigate';

// One paragraph of four lines, the first two in frame A and the last two in frame B, side by side.
const story = layoutOf([['one two', 'three four', 'five six', 'seven eight'], ['end']]);
const part = (from: number, to: number, last = false): FrameLayout => {
	const lines = story.lines
		.slice(from, to)
		.map((l, i) => ({ ...l, top: i * LINE_H, baseline: i * LINE_H + 15 }));
	return {
		frame: 'f',
		lines: last ? lines : lines.filter((l) => l.paragraph === 0),
		overset: false,
	};
};
const A = { id: 'a', box: { x: 0, y: 0, w: 100, h: 100 }, layout: part(0, 2) };
const B = { id: 'b', box: { x: 300, y: 50, w: 100, h: 100 }, layout: part(2, 5, true) };

describe('the frames of a story on a spread', () => {
	it("are one set of lines in the edited frame's coordinates", () => {
		const geometry = flowGeometry([A, B], 'a')!;
		expect(geometry.layout.lines).toHaveLength(5);
		// Frame B is 300 points right and 50 down of A, and its first line is the third of the paragraph.
		const rect = caretRect(geometry.layout, { para: 0, offset: story.lines[2]!.start })!;
		expect(rect.x).toBe(300);
		expect(rect.y).toBe(50);
		// Editing from B puts A to its left.
		const from = flowGeometry([A, B], 'b')!;
		expect(caretRect(from.layout, { para: 0, offset: 0 })!.x).toBe(-300);
	});

	it('are not joined when there is one, or when one is turned', () => {
		expect(flowGeometry([A], 'a')).toBeNull();
		expect(flowGeometry([A, { ...B, rotation: 10 }], 'a')).toBeNull();
		expect(flowGeometry([A, B], 'missing')).toBeNull();
	});

	it('go down from the last line of a frame to the first line of the next, and back up', () => {
		const { layout, leftOf } = flowGeometry([A, B], 'a')!;
		const lastOfA = { para: 0, offset: story.lines[1]!.start + 2 };
		const down = moveVertical(layout, lastOfA, 1, undefined, leftOf);
		expect(down.pos).toEqual({ para: 0, offset: story.lines[2]!.start + 2 });
		const up = moveVertical(layout, down.pos, -1, down.goalX, leftOf);
		expect(up.pos).toEqual(lastOfA);
	});

	it('take Home and End to the edges of a line in the second frame', () => {
		const { layout } = flowGeometry([A, B], 'a')!;
		const inB = { para: 0, offset: story.lines[3]!.start + 3 };
		expect(moveLineEdge(layout, inB, 'home').offset).toBe(story.lines[3]!.start);
		expect(moveLineEdge(layout, inB, 'end').offset).toBe(
			story.lines[3]!.start + 'seven eight'.length,
		);
	});

	it('are hit in the nearest frame, so a drag can cross from one to the next', () => {
		const geometry = flowGeometry([A, B], 'a')!;
		// Inside frame A's first line, 2 letters in.
		expect(geometry.hit(2 * CHAR_W, 5)).toEqual({ para: 0, offset: 2 });
		// Inside frame B (300, 50): its first line holds the paragraph's third line.
		expect(geometry.hit(300 + 3 * CHAR_W, 50 + 5)).toEqual({
			para: 0,
			offset: story.lines[2]!.start + 3,
		});
		// Past frame A's right edge and level with B: the nearer frame wins.
		expect(geometry.hit(250, 60)!.offset).toBe(story.lines[2]!.start);
		// Below everything, in the last frame's last line.
		expect(geometry.hit(310, 400)!.para).toBe(1);
	});

	it('cover the lines of both frames in a selection across the break', () => {
		const { layout } = flowGeometry([A, B], 'a')!;
		const rects = selectionRects(layout, { para: 0, offset: 8 }, { para: 0, offset: 25 });
		const xs = new Set(rects.map((r) => r.x >= 300));
		expect(xs).toEqual(new Set([false, true]));
		// The end of frame A is not a paragraph break, so it is not marked as one.
		const first = rects.find((r) => r.x < 300 && r.y === LINE_H)!;
		expect(first.w).toBe('three four'.length * CHAR_W);
	});
});
