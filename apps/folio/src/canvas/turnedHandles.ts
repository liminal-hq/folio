// Handles on a turned frame: where they are, how they resize it, and how a turn handle turns it
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { PlainRect } from './geometry';
import { handlePoint, resizeRect, type Handle } from './snap';

interface Point {
	x: number;
	y: number;
}

/** `point` turned `degrees` clockwise about `centre`. */
export function turnPoint(point: Point, centre: Point, degrees: number): Point {
	const radians = (degrees * Math.PI) / 180;
	const sin = Math.sin(radians);
	const cos = Math.cos(radians);
	const dx = point.x - centre.x;
	const dy = point.y - centre.y;
	return { x: centre.x + dx * cos - dy * sin, y: centre.y + dx * sin + dy * cos };
}

const middle = (box: PlainRect): Point => ({ x: box.x + box.w / 2, y: box.y + box.h / 2 });

const OPPOSITE: Record<Handle, Handle> = {
	nw: 'se',
	n: 's',
	ne: 'sw',
	e: 'w',
	se: 'nw',
	s: 'n',
	sw: 'ne',
	w: 'e',
};

/** Where a handle of a turned frame is on the page. */
export function turnedHandle(box: PlainRect, degrees: number, handle: Handle): Point {
	return turnPoint(handlePoint(box, handle), middle(box), degrees);
}

/**
 * Resizes a turned frame by dragging `handle` by `(dx, dy)` on the page. The drag is read along the
 * frame's own sides, and the corner or edge opposite the handle stays where it is on the page, so
 * the frame grows away from it the way the person sees.
 */
export function resizeTurned(
	box: PlainRect,
	degrees: number,
	handle: Handle,
	dx: number,
	dy: number,
	keepAspect = false,
): PlainRect {
	if (!degrees) return resizeRect(box, handle, dx, dy, keepAspect);
	const along = turnPoint({ x: dx, y: dy }, { x: 0, y: 0 }, -degrees);
	const next = resizeRect(box, handle, along.x, along.y, keepAspect);
	const opposite = OPPOSITE[handle];
	const held = turnedHandle(box, degrees, opposite);
	const now = turnedHandle(next, degrees, opposite);
	return { ...next, x: next.x + held.x - now.x, y: next.y + held.y - now.y };
}

/** The angle, in degrees clockwise from upright, that a turn handle dragged to `at` gives a frame. */
export function turnAngle(centre: Point, at: Point, snap: boolean): number {
	const raw = (Math.atan2(at.y - centre.y, at.x - centre.x) * 180) / Math.PI + 90;
	const angle = snap ? Math.round(raw / 15) * 15 : raw;
	// Kept in 0–360 and to a hundredth of a degree, as a person would type it.
	return (((Math.round(angle * 100) / 100) % 360) + 360) % 360;
}

const CURSOR_AT = ['ns-resize', 'nesw-resize', 'ew-resize', 'nwse-resize'] as const;
const ANGLE_OF: Record<Handle, number> = { n: 0, ne: 1, e: 2, se: 3, s: 4, sw: 5, w: 6, nw: 7 };

/** The resize cursor for a handle of a frame turned by `degrees`: the nearest of the four. */
export function turnedCursor(handle: Handle, degrees: number): string {
	const steps = Math.round(degrees / 45);
	return CURSOR_AT[(((ANGLE_OF[handle] + steps) % 4) + 4) % 4]!;
}
