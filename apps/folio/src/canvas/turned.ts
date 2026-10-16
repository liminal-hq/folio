// The turn of a frame about the middle of its box, as an SVG transform
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { Frame } from '@liminal-hq/folio-protocol/generated/Frame';
import type { PlainRect } from './geometry';

/** The turn of a frame about the middle of its box; nothing when it is upright. */
export function turned(frame: Frame): string | undefined {
	if (!frame.rotation) return undefined;
	const { x, y, w, h } = frame.box;
	return `rotate(${frame.rotation} ${x + w / 2} ${y + h / 2})`;
}

/**
 * The upright box that holds a rectangle after it is turned `degrees` clockwise about its middle:
 * where a turned frame is for snapping. The twin of `Rect::turned_bounds` in `folio-model`.
 */
export function turnedBounds(box: PlainRect, degrees: number): PlainRect {
	if (!degrees) return box;
	const radians = (degrees * Math.PI) / 180;
	const sin = Math.abs(Math.sin(radians));
	const cos = Math.abs(Math.cos(radians));
	const w = box.w * cos + box.h * sin;
	const h = box.w * sin + box.h * cos;
	const tidy = (n: number) => Math.round(n * 1e9) / 1e9;
	return {
		x: tidy(box.x + box.w / 2 - w / 2),
		y: tidy(box.y + box.h / 2 - h / 2),
		w: tidy(w),
		h: tidy(h),
	};
}
