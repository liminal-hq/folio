// The lines of the frames of one story on a spread as one set, so the caret can cross from frame to frame
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { LayoutLine } from '@liminal-hq/folio-protocol/generated/LayoutLine';
import type { FrameLayout } from '@liminal-hq/folio-protocol/generated/FrameLayout';
import type { TextPos } from '@liminal-hq/folio-protocol/generated/TextPos';
import { hitTest, type Box } from './caretGeometry';

/** One frame of a story: where it is on the spread (in points) and the lines laid out in it. */
export interface FlowFrame {
	id: string;
	box: Box;
	/** How far the frame is turned, in degrees. */
	rotation?: number;
	layout: FrameLayout | undefined;
}

/** The frames of a story as the editor sees them, in the coordinates of the frame being edited. */
export interface FlowGeometry {
	/** Every frame's lines, in reading order, placed as the frames are relative to the edited one. */
	layout: FrameLayout;
	/** Where each frame is, relative to the top-left of the edited frame. */
	frames: { id: string; box: Box }[];
	/** The left edge of the frame a line is in, so a column is kept from one frame to the next. */
	leftOf(line: LayoutLine): number;
	/** The position under a point of the edited frame's coordinates: the nearest frame's, then its line. */
	hit(x: number, y: number, snap?: (para: number, offset: number) => number): TextPos | undefined;
}

const shifted = (layout: FrameLayout, dx: number, dy: number): FrameLayout => ({
	...layout,
	lines: layout.lines.map((l) => ({
		...l,
		x: l.x + dx,
		top: l.top + dy,
		baseline: l.baseline + dy,
	})),
});

/** How far a point is from a box; zero inside it. */
function distance(box: Box, x: number, y: number): number {
	const dx = x < box.x ? box.x - x : x > box.x + box.w ? x - box.x - box.w : 0;
	const dy = y < box.y ? box.y - y : y > box.y + box.h ? y - box.y - box.h : 0;
	return Math.hypot(dx, dy);
}

/**
 * Puts the lines of the frames of a story on one plane, the edited frame's. Answers `null` when
 * there is one frame to edit, or when a frame is turned (a turned frame's lines would not fit the
 * plane, so the caret stays in the frame being edited). Frames on other spreads are not among
 * `frames`: the caret does not leave the spread.
 */
export function flowGeometry(frames: readonly FlowFrame[], edited: string): FlowGeometry | null {
	const own = frames.find((f) => f.id === edited);
	if (!own || frames.length < 2) return null;
	if (frames.some((f) => f.rotation)) return null;
	const placed = frames.map((f) => ({
		id: f.id,
		box: { x: f.box.x - own.box.x, y: f.box.y - own.box.y, w: f.box.w, h: f.box.h },
		layout: f.layout ? shifted(f.layout, f.box.x - own.box.x, f.box.y - own.box.y) : undefined,
	}));
	const lefts = new Map<LayoutLine, number>();
	for (const f of placed) for (const l of f.layout?.lines ?? []) lefts.set(l, f.box.x);
	const lines = placed.flatMap((f) => f.layout?.lines ?? []);
	const last = [...placed].reverse().find((f) => f.layout)?.layout;
	return {
		layout: { frame: own.id, lines, overset: last?.overset ?? false },
		frames: placed.map(({ id, box }) => ({ id, box })),
		leftOf: (line) => lefts.get(line) ?? 0,
		hit(x, y, snap) {
			let best: (typeof placed)[number] | undefined;
			let gap = Infinity;
			for (const f of placed) {
				if (!f.layout?.lines.some((l) => l.paragraph != null)) continue;
				const d = distance(f.box, x, y);
				if (d < gap) {
					best = f;
					gap = d;
				}
			}
			return best?.layout ? hitTest(best.layout, x, y, snap) : undefined;
		},
	};
}
