// Tests for resizing and turning a turned frame
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { describe, expect, it } from 'vitest';
import { resizeTurned, turnAngle, turnedCursor, turnedHandle, turnPoint } from './turnedHandles';

const box = { x: 100, y: 100, w: 80, h: 40 };

describe('turnPoint', () => {
	it('turns clockwise on the page, where y runs down', () => {
		const p = turnPoint({ x: 10, y: 0 }, { x: 0, y: 0 }, 90);
		expect(p.x).toBeCloseTo(0);
		expect(p.y).toBeCloseTo(10);
	});
});

describe('resizeTurned', () => {
	it('is the plain resize when upright', () => {
		expect(resizeTurned(box, 0, 'se', 10, 5)).toEqual({ x: 100, y: 100, w: 90, h: 45 });
	});

	it('reads the drag along the frame, and holds the opposite corner on the page', () => {
		// Turned a quarter, the frame's east edge faces down the page: pulling the e handle down by
		// 10 makes it 10 wider, and the west edge stays where it was on the page.
		const west = turnedHandle(box, 90, 'w');
		const next = resizeTurned(box, 90, 'e', 0, 10);
		expect(next.w).toBeCloseTo(90);
		expect(next.h).toBeCloseTo(40);
		const after = turnedHandle(next, 90, 'w');
		expect(after.x).toBeCloseTo(west.x);
		expect(after.y).toBeCloseTo(west.y);
	});

	it('ignores a drag across the frame for a side handle', () => {
		const next = resizeTurned(box, 90, 'e', 25, 0);
		expect(next.w).toBeCloseTo(80);
	});

	it('holds a corner fixed on a diagonal turn', () => {
		const fixed = turnedHandle(box, 30, 'nw');
		const next = resizeTurned(box, 30, 'se', 12, 7);
		const after = turnedHandle(next, 30, 'nw');
		expect(after.x).toBeCloseTo(fixed.x);
		expect(after.y).toBeCloseTo(fixed.y);
	});
});

describe('turnAngle', () => {
	const centre = { x: 0, y: 0 };
	it('is upright with the handle straight above, and quarter turns around', () => {
		expect(turnAngle(centre, { x: 0, y: -50 }, false)).toBe(0);
		expect(turnAngle(centre, { x: 50, y: 0 }, false)).toBe(90);
		expect(turnAngle(centre, { x: 0, y: 50 }, false)).toBe(180);
		expect(turnAngle(centre, { x: -50, y: 0 }, false)).toBe(270);
	});

	it('snaps to 15° when asked', () => {
		expect(turnAngle(centre, { x: 50, y: -42 }, true)).toBe(45);
		expect(turnAngle(centre, { x: 50, y: -42 }, false)).not.toBe(45);
	});
});

describe('turnedCursor', () => {
	it('follows the handle round to the nearest resize cursor', () => {
		expect(turnedCursor('n', 0)).toBe('ns-resize');
		expect(turnedCursor('n', 90)).toBe('ew-resize');
		expect(turnedCursor('n', 45)).toBe('nesw-resize');
		expect(turnedCursor('se', 90)).toBe('nesw-resize');
		expect(turnedCursor('w', 270)).toBe('ns-resize');
	});
});
