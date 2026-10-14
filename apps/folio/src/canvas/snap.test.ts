// Tests for snapping and resizing a frame
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { describe, expect, it } from 'vitest';
import { resizeRect, snapAxis, snapMove } from './snap';

describe('snapAxis', () => {
	it('snaps to the nearest target within the threshold', () => {
		expect(snapAxis([10, 15], [0, 16], 2)).toEqual({ shift: 1, target: 16 });
	});
	it('does not snap outside the threshold', () => {
		expect(snapAxis([10], [20], 2)).toBeNull();
	});
});

describe('snapMove', () => {
	it('moves each axis onto its own target and reports the lines', () => {
		const out = snapMove({ x: 9, y: 50, w: 20, h: 10 }, { x: [0, 10], y: [100] }, 3);
		expect(out.rect).toEqual({ x: 10, y: 50, w: 20, h: 10 });
		expect(out.lines).toEqual([{ axis: 'x', pos: 10 }]);
	});
	it('snaps the centre as well as the edges', () => {
		const out = snapMove({ x: 0, y: 0, w: 20, h: 10 }, { x: [51], y: [] }, 3);
		// The centre is at 10, which is 41 away: nothing snaps.
		expect(out.rect.x).toBe(0);
		const near = snapMove({ x: 40, y: 0, w: 20, h: 10 }, { x: [51], y: [] }, 3);
		expect(near.rect.x).toBe(41);
	});
});

describe('resizeRect', () => {
	const rect = { x: 10, y: 10, w: 100, h: 50 };
	it('moves only the edges the handle owns', () => {
		expect(resizeRect(rect, 'se', 20, 10)).toEqual({ x: 10, y: 10, w: 120, h: 60 });
		expect(resizeRect(rect, 'nw', 10, 5)).toEqual({ x: 20, y: 15, w: 90, h: 45 });
		expect(resizeRect(rect, 'e', 5, 99)).toEqual({ x: 10, y: 10, w: 105, h: 50 });
	});
	it('never goes below the minimum size or flips over', () => {
		const out = resizeRect(rect, 'w', 500, 0);
		expect(out.w).toBe(4);
		expect(out.x + out.w).toBe(110);
	});
	it('keeps the shape on a corner when asked', () => {
		const out = resizeRect(rect, 'se', 100, 0, true);
		expect(out.w / out.h).toBeCloseTo(2);
		expect(out.w).toBe(200);
		const nw = resizeRect(rect, 'nw', -50, 0, true);
		expect(nw.x + nw.w).toBe(110);
		expect(nw.y + nw.h).toBe(60);
	});
});
