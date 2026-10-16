// Tests for where a picture sits in its frame
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { describe, expect, it } from 'vitest';
import { focusedRect } from './focusedRect';

const box = { x: 10, y: 20, w: 100, h: 100 };
const wide = { w: 400, h: 200 };

describe('focusedRect', () => {
	it('fills the box and centres on the middle focus', () => {
		expect(focusedRect(box, wide, 'fill', [0.5, 0.5])).toEqual({ x: -40, y: 20, w: 200, h: 100 });
	});

	it('slides to keep the focus near the middle, never leaving a gap', () => {
		// A focus at the far left stays pinned to the left edge.
		expect(focusedRect(box, wide, 'fill', [0, 0.5]).x).toBe(10);
		// A focus at the far right stays pinned to the right edge.
		expect(focusedRect(box, wide, 'fill', [1, 0.5]).x).toBe(10 + 100 - 200);
		// In between, the focus point lands in the middle of the box.
		const r = focusedRect(box, wide, 'fill', [0.4, 0.5]);
		expect(r.x + 0.4 * r.w).toBeCloseTo(60);
	});

	it('shows the whole picture centred when fitted, and ignores the focus', () => {
		expect(focusedRect(box, wide, 'fit', [0, 0])).toEqual({ x: 10, y: 45, w: 100, h: 50 });
	});

	it('pulls to the box when stretched, and tolerates an empty picture', () => {
		expect(focusedRect(box, wide, 'stretch', [0, 0])).toEqual(box);
		expect(focusedRect(box, { w: 0, h: 0 }, 'fill', [0.5, 0.5])).toEqual(box);
	});

	it('clamps a focus outside the picture', () => {
		expect(focusedRect(box, wide, 'fill', [-3, 9]).x).toBe(10);
	});
});
