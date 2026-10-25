// Where a spread's pages, bleed and margins sit, in points from the top-left of the first page's trim
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { Book } from '@liminal-hq/folio-protocol/generated/Book';
import type { Spread } from '@liminal-hq/folio-protocol/generated/Spread';
import { COVER_EDGE, coverWidth } from '../books/coverMaths';

export interface PlainRect {
	x: number;
	y: number;
	w: number;
	h: number;
}

export interface PageGeometry {
	/** The trimmed page. */
	trim: PlainRect;
	/** Inside the margins, where text is safe. */
	safe: PlainRect;
}

export interface SpreadGeometry {
	pages: PageGeometry[];
	/** The whole spread with the bleed around it. */
	bounds: PlainRect;
	/** The spread's trimmed width. */
	width: number;
	height: number;
}

/** CSS pixels in a point at 100%: the screen is taken as 96 dpi. */
export const PX_PER_POINT = 96 / 72;

/** Whether the page at `index` sits on the right of its spread, so its binding edge is on the left. */
function isRecto(spread: Spread, index: number): boolean {
	if (spread.pages.length === 2) return index === 1;
	return spread.kind !== 'closer';
}

export function spreadGeometry(book: Book, spread: Spread): SpreadGeometry {
	const { h } = book.trim;
	// The cover is the printed sheet, back, spine and front, with KDP's own bleed round it.
	const cover = spread.kind === 'cover';
	const w = cover ? coverWidth(book) : book.trim.w;
	const bleed = cover ? COVER_EDGE : book.bleed;
	const { top, bottom, inside, outside } = book.margins;
	const pages = spread.pages.map((_, index) => {
		const x = index * w;
		const left = isRecto(spread, index) ? inside : outside;
		const right = isRecto(spread, index) ? outside : inside;
		return {
			trim: { x, y: 0, w, h },
			safe: { x: x + left, y: top, w: w - left - right, h: h - top - bottom },
		};
	});
	const width = w * spread.pages.length;
	return {
		pages,
		bounds: { x: -bleed, y: -bleed, w: width + bleed * 2, h: h + bleed * 2 },
		width,
		height: h,
	};
}

/** The zoom (1 is 100%) that fits `bounds` in a stage of the given size, leaving `padding` around it. */
export function fitZoom(bounds: PlainRect, stage: { w: number; h: number }, padding = 32): number {
	const room = { w: stage.w - padding * 2, h: stage.h - padding * 2 };
	if (room.w <= 0 || room.h <= 0) return 1;
	return Math.min(room.w / (bounds.w * PX_PER_POINT), room.h / (bounds.h * PX_PER_POINT));
}

export const MIN_ZOOM = 0.1;
export const MAX_ZOOM = 4;

export function clampZoom(zoom: number): number {
	return Math.min(MAX_ZOOM, Math.max(MIN_ZOOM, zoom));
}

/** The next step up or down a ladder of familiar zoom levels. */
export function stepZoom(zoom: number, direction: 1 | -1): number {
	const steps = [0.1, 0.25, 0.33, 0.5, 0.67, 0.75, 1, 1.25, 1.5, 2, 3, 4];
	if (direction === 1) return steps.find((s) => s > zoom + 0.001) ?? MAX_ZOOM;
	return [...steps].reverse().find((s) => s < zoom - 0.001) ?? MIN_ZOOM;
}
