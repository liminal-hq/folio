// Snapping while a frame is dragged: the same maths as `folio-model`'s `snap_axis`, run in the page
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { Frame } from '@liminal-hq/folio-protocol/generated/Frame';
import type { PlainRect, SpreadGeometry } from './geometry';
import { turnedBounds } from './turned';

/** How near, in screen pixels, an edge has to come to a target before it snaps. */
export const SNAP_THRESHOLD_PX = 8;

/** The shift that brings the nearest candidate onto the nearest target, and that target. */
export function snapAxis(
	candidates: number[],
	targets: number[],
	threshold: number,
): { shift: number; target: number } | null {
	let best: { dist: number; shift: number; target: number } | null = null;
	for (const c of candidates) {
		for (const t of targets) {
			const dist = Math.abs(t - c);
			if (dist <= threshold && (best === null || dist < best.dist)) {
				best = { dist, shift: t - c, target: t };
			}
		}
	}
	return best && { shift: best.shift, target: best.target };
}

export interface SnapTargets {
	x: number[];
	y: number[];
}

/** Where a frame's edges and centre can land: the pages, their margins, other frames and guides. */
export function snapTargets(
	geometry: SpreadGeometry,
	frames: Frame[],
	moving: string[],
	guides: { axis: 'x' | 'y'; pos: number }[] = [],
): SnapTargets {
	const x: number[] = [];
	const y: number[] = [];
	for (const page of geometry.pages) {
		for (const r of [page.trim, page.safe]) {
			x.push(r.x, r.x + r.w / 2, r.x + r.w);
			y.push(r.y, r.y + r.h / 2, r.y + r.h);
		}
	}
	for (const frame of frames) {
		if (moving.includes(frame.id)) continue;
		const r = turnedBounds(frame.box, frame.rotation);
		x.push(r.x, r.x + r.w / 2, r.x + r.w);
		y.push(r.y, r.y + r.h / 2, r.y + r.h);
	}
	for (const guide of guides) (guide.axis === 'x' ? x : y).push(guide.pos);
	return { x, y };
}

export interface Snapped {
	rect: PlainRect;
	/** The lines to draw while the snap holds, in points. */
	lines: { axis: 'x' | 'y'; pos: number }[];
}

const edgesOf = (a: number, size: number) => [a, a + size / 2, a + size];

/**
 * Moves `rect` so its edges or centre meet a target on either axis, when one is near enough. A frame
 * turned by `rotation` degrees meets them with the box around its turned corners.
 */
export function snapMove(
	rect: PlainRect,
	targets: SnapTargets,
	threshold: number,
	rotation = 0,
): Snapped {
	const around = turnedBounds(rect, rotation);
	const sx = snapAxis(edgesOf(around.x, around.w), targets.x, threshold);
	const sy = snapAxis(edgesOf(around.y, around.h), targets.y, threshold);
	const lines: Snapped['lines'] = [];
	if (sx) lines.push({ axis: 'x', pos: sx.target });
	if (sy) lines.push({ axis: 'y', pos: sy.target });
	return { rect: { ...rect, x: rect.x + (sx?.shift ?? 0), y: rect.y + (sy?.shift ?? 0) }, lines };
}

export type Handle = 'nw' | 'n' | 'ne' | 'e' | 'se' | 's' | 'sw' | 'w';

export const HANDLES: Handle[] = ['nw', 'n', 'ne', 'e', 'se', 's', 'sw', 'w'];

export const MIN_SIZE = 4;

/**
 * Resizes `rect` by dragging `handle` by `(dx, dy)`. The edges the handle does not own stay put;
 * `keepAspect` holds the shape (corner handles only), and a frame never goes below `MIN_SIZE`.
 */
export function resizeRect(
	rect: PlainRect,
	handle: Handle,
	dx: number,
	dy: number,
	keepAspect = false,
): PlainRect {
	let { x, y, w, h } = rect;
	const right = x + w;
	const bottom = y + h;
	if (handle.includes('w')) {
		x = Math.min(x + dx, right - MIN_SIZE);
		w = right - x;
	}
	if (handle.includes('e')) w = Math.max(MIN_SIZE, w + dx);
	if (handle.includes('n')) {
		y = Math.min(y + dy, bottom - MIN_SIZE);
		h = bottom - y;
	}
	if (handle.includes('s')) h = Math.max(MIN_SIZE, h + dy);
	// A flat line has no proportion to keep: dividing by its height would send the size to NaN.
	if (keepAspect && handle.length === 2 && rect.w > 0 && rect.h > 0) {
		const ratio = rect.w / rect.h;
		// Follow whichever axis moved further, in proportion to the frame.
		if (Math.abs(w - rect.w) / rect.w >= Math.abs(h - rect.h) / rect.h) h = w / ratio;
		else w = h * ratio;
		if (handle.includes('w')) x = right - w;
		if (handle.includes('n')) y = bottom - h;
	}
	return { x, y, w, h };
}
