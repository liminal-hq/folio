// Tests for the box around a turned frame
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { describe, expect, it } from 'vitest';
import { snapMove, snapTargets } from './snap';
import { turnedBounds } from './turned';

const box = { x: 100, y: 50, w: 40, h: 10 };

describe('turnedBounds', () => {
	it('is the box itself when upright', () => {
		expect(turnedBounds(box, 0)).toBe(box);
	});

	it('swaps the sides about the middle for a quarter turn, either way', () => {
		expect(turnedBounds(box, 90)).toEqual({ x: 115, y: 35, w: 10, h: 40 });
		expect(turnedBounds(box, -90)).toEqual(turnedBounds(box, 270));
		expect(turnedBounds(box, 180)).toEqual(box);
	});

	it('holds the corners of a diagonal turn', () => {
		const d = turnedBounds(box, 45);
		expect(d.w).toBeCloseTo(50 / Math.SQRT2);
		expect(d.h).toBeCloseTo(d.w);
		expect(d.x + d.w / 2).toBeCloseTo(120);
	});
});

describe('snapping a turned frame', () => {
	it('meets a target with the box around its turned corners', () => {
		// On its side the frame spans x 115–125, so its left edge is 5 from a target at 110.
		const moved = snapMove(box, { x: [110], y: [] }, 8, 90);
		expect(moved.rect.x).toBe(box.x - 5);
		expect(moved.lines).toEqual([{ axis: 'x', pos: 110 }]);
		// Upright, the same frame is 10 away and does not snap.
		expect(snapMove(box, { x: [110], y: [] }, 8).rect.x).toBe(box.x);
	});

	it("offers another turned frame's bounding edges as targets", () => {
		const geometry = { pages: [], bounds: box, width: 0, height: 0 };
		const turnedFrame = { id: 'a', box, rotation: 90 } as never;
		const targets = snapTargets(geometry, [turnedFrame], []);
		expect(targets.x).toEqual([115, 120, 125]);
		expect(targets.y).toEqual([35, 55, 75]);
	});
});
