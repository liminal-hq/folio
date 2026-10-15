// The turn of a frame about the middle of its box, as an SVG transform
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { Frame } from '@liminal-hq/folio-protocol/generated/Frame';

/** The turn of a frame about the middle of its box; nothing when it is upright. */
export function turned(frame: Frame): string | undefined {
	if (!frame.rotation) return undefined;
	const { x, y, w, h } = frame.box;
	return `rotate(${frame.rotation} ${x + w / 2} ${y + h / 2})`;
}
