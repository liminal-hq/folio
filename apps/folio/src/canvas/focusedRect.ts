// Where a picture sits in its frame so that the point the person cares about stays in view
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { Rect } from './backgrounds';

export interface Size {
	w: number;
	h: number;
}

const clamp = (n: number, lo: number, hi: number) => Math.min(hi, Math.max(lo, n));

/**
 * The rectangle to draw a picture of `natural` size in, inside `box`. Fill covers the box and crops
 * the rest, sliding so the `focus` point (as 0–1 across and down the picture) is as near the middle
 * of the box as the picture allows. Fit shows the whole picture, centred, and Stretch pulls it to
 * the box.
 */
export function focusedRect(
	box: Rect,
	natural: Size,
	fit: 'fill' | 'fit' | 'stretch',
	focus: readonly [number, number],
): Rect {
	if (fit === 'stretch' || natural.w <= 0 || natural.h <= 0) return box;
	const scale =
		fit === 'fill'
			? Math.max(box.w / natural.w, box.h / natural.h)
			: Math.min(box.w / natural.w, box.h / natural.h);
	const w = natural.w * scale;
	const h = natural.h * scale;
	if (fit === 'fit') return { x: box.x + (box.w - w) / 2, y: box.y + (box.h - h) / 2, w, h };
	const fx = clamp(focus[0], 0, 1);
	const fy = clamp(focus[1], 0, 1);
	return {
		x: box.x + clamp(box.w / 2 - fx * w, Math.min(0, box.w - w), 0),
		y: box.y + clamp(box.h / 2 - fy * h, Math.min(0, box.h - h), 0),
		w,
		h,
	};
}
