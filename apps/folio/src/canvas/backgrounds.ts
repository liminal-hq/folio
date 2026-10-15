// The background each page of a spread shows: its own, else the spread's, else its master's
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { Background } from '@liminal-hq/folio-protocol/generated/Background';
import type { Book } from '@liminal-hq/folio-protocol/generated/Book';
import type { Master } from '@liminal-hq/folio-protocol/generated/Master';
import type { ResolvedBackground } from '@liminal-hq/folio-protocol/generated/ResolvedBackground';
import type { Spread } from '@liminal-hq/folio-protocol/generated/Spread';
import { spreadGeometry } from './geometry';

/**
 * The twin of Rust's `folio_model::resolve`, for drawing the canvas and the shelf's thumbnails
 * (which never fetch a layout). Both are tested against `fixtures/backgroundCases.json`. A page
 * with none of the three shows the paper, which is `null`.
 */
export function resolveBackgrounds(
	masters: readonly Master[],
	spread: Spread,
): (ResolvedBackground | null)[] {
	return spread.pages.map((page) => {
		if (page.bg) return { source: 'page', bg: page.bg };
		// A span needs two pages: a hand-edited book may carry one on a single page.
		if (spread.bgSpan && spread.pages.length > 1) return { source: 'span', bg: spread.bgSpan };
		const master = masters.find((m) => m.id === page.master);
		return master?.bg ? { source: 'master', bg: master.bg } : null;
	});
}

export interface Rect {
	x: number;
	y: number;
	w: number;
	h: number;
}

/**
 * Where each page's background is painted: the trimmed page, reaching into the bleed on the edges
 * that are the outside of the spread, so the colour runs off the sheet when it is cut.
 */
export function paintRects(book: Book, spread: Spread): Rect[] {
	const { pages, bounds } = spreadGeometry(book, spread);
	return pages.map((page, index) => {
		const left = index === 0 ? bounds.x : page.trim.x;
		const right = index === pages.length - 1 ? bounds.x + bounds.w : page.trim.x + page.trim.w;
		return { x: left, y: bounds.y, w: right - left, h: bounds.h };
	});
}

/**
 * The line a gradient runs along inside `rect`, as the ends of it. The angle is clockwise from up,
 * as in CSS: 0° runs from the bottom to the top, 90° left to right and 180° top to bottom. It is as long as it takes for the corners to take the first and last
 * colour, so the angle looks the same on a wide spread as on a single page.
 */
export function gradientLine(
	rect: Rect,
	angle: number,
): { x1: number; y1: number; x2: number; y2: number } {
	const radians = (angle * Math.PI) / 180;
	const dx = Math.sin(radians);
	const dy = -Math.cos(radians);
	const half = (Math.abs(rect.w * dx) + Math.abs(rect.h * dy)) / 2;
	const cx = rect.x + rect.w / 2;
	const cy = rect.y + rect.h / 2;
	return { x1: cx - dx * half, y1: cy - dy * half, x2: cx + dx * half, y2: cy + dy * half };
}

/** The pictures the book's backgrounds use, each once: on pages, across spreads and on masters. */
export function backgroundAssets(book: Book): string[] {
	const found = new Set<string>();
	const note = (bg: Background | null | undefined) => {
		if (bg?.type === 'picture') found.add(bg.asset);
	};
	for (const spread of book.spreads) {
		note(spread.bgSpan);
		spread.pages.forEach((page) => note(page.bg));
	}
	(book.masterPages ?? []).forEach((master) => note(master.bg));
	return [...found];
}
