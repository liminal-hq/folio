// Aligning and distributing frames: where each one moves to, measured on the box a turned frame fills
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { Frame } from '@liminal-hq/folio-protocol/generated/Frame';
import type { FrameMove } from '@liminal-hq/folio-protocol/generated/FrameMove';
import type { PlainRect } from './geometry';
import { turnedBounds } from './turned';

/** Which edge, or middle, the frames line up on. */
export type AlignEdge = 'left' | 'centre' | 'right' | 'top' | 'middle' | 'bottom';
/** Whether frames are spread out across the page or down it. */
export type DistributeAxis = 'horizontal' | 'vertical';

/** The control that asks for each alignment. */
export const ALIGN_CONTROLS: Readonly<Record<string, AlignEdge>> = {
	alignLeftE: 'left',
	alignCenterE: 'centre',
	alignRightE: 'right',
	alignTopE: 'top',
	alignMidE: 'middle',
	alignBotE: 'bottom',
};

/** The control that asks for each distribution. */
export const DISTRIBUTE_CONTROLS: Readonly<Record<string, DistributeAxis>> = {
	distH: 'horizontal',
	distV: 'vertical',
};

/** The box a frame fills on the page: its own, or the upright box round it once it is turned. */
const filled = (frame: Frame): PlainRect => turnedBounds(frame.box, frame.rotation);

/** A move by `dx` and `dy`, or nothing when that is no move at all. */
function shifted(frame: Frame, dx: number, dy: number): FrameMove[] {
	if (Math.abs(dx) < 1e-9 && Math.abs(dy) < 1e-9) return [];
	const { x, y, w, h } = frame.box;
	return [{ frame: frame.id, box: { x: x + dx, y: y + dy, w, h } }];
}

const union = (rects: PlainRect[]): PlainRect => {
	const left = Math.min(...rects.map((r) => r.x));
	const top = Math.min(...rects.map((r) => r.y));
	const right = Math.max(...rects.map((r) => r.x + r.w));
	const bottom = Math.max(...rects.map((r) => r.y + r.h));
	return { x: left, y: top, w: right - left, h: bottom - top };
};

/** The page a lone frame lines up with: the one its middle is over, or the nearest. */
function pageOf(frame: Frame, pages: readonly PlainRect[]): PlainRect | null {
	if (pages.length === 0) return null;
	const rect = filled(frame);
	const middle = rect.x + rect.w / 2;
	return (
		pages.find((p) => middle >= p.x && middle < p.x + p.w) ??
		(middle < pages[0]!.x ? pages[0]! : pages[pages.length - 1]!)
	);
}

/**
 * The moves that line `frames` up on `edge`. Several frames line up with the box that holds them
 * all; a single frame lines up with its page. A turned frame is measured on the box it fills, so
 * its visible edge is what lines up.
 */
export function alignMoves(
	frames: readonly Frame[],
	edge: AlignEdge,
	pages: readonly PlainRect[],
): FrameMove[] {
	if (frames.length === 0) return [];
	const target = frames.length === 1 ? pageOf(frames[0]!, pages) : union(frames.map(filled));
	if (!target) return [];
	return frames.flatMap((frame) => {
		const rect = filled(frame);
		switch (edge) {
			case 'left':
				return shifted(frame, target.x - rect.x, 0);
			case 'centre':
				return shifted(frame, target.x + target.w / 2 - (rect.x + rect.w / 2), 0);
			case 'right':
				return shifted(frame, target.x + target.w - (rect.x + rect.w), 0);
			case 'top':
				return shifted(frame, 0, target.y - rect.y);
			case 'middle':
				return shifted(frame, 0, target.y + target.h / 2 - (rect.y + rect.h / 2));
			case 'bottom':
				return shifted(frame, 0, target.y + target.h - (rect.y + rect.h));
		}
	});
}

/**
 * The moves that leave equal gaps between `frames` along `axis`. The outermost frames stay where
 * they are, and the ones between are spread out by the room that is left.
 */
export function distributeMoves(frames: readonly Frame[], axis: DistributeAxis): FrameMove[] {
	if (frames.length < 3) return [];
	const horizontal = axis === 'horizontal';
	const start = (r: PlainRect) => (horizontal ? r.x : r.y);
	const size = (r: PlainRect) => (horizontal ? r.w : r.h);
	const ordered = frames
		.map((frame) => ({ frame, rect: filled(frame) }))
		.sort((a, b) => start(a.rect) - start(b.rect));
	const first = Math.min(...ordered.map((o) => start(o.rect)));
	const last = Math.max(...ordered.map((o) => start(o.rect) + size(o.rect)));
	const filling = ordered.reduce((sum, o) => sum + size(o.rect), 0);
	const gap = (last - first - filling) / (ordered.length - 1);
	let at = first;
	return ordered.flatMap(({ frame, rect }) => {
		const shift = at - start(rect);
		at += size(rect) + gap;
		return horizontal ? shifted(frame, shift, 0) : shifted(frame, 0, shift);
	});
}
