// Where the spread's zero sits in the stage, which moves as the stage scrolls, resizes and zooms
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { useLayoutEffect, useState, type RefObject } from 'react';

export interface PageOrigin {
	/** Pixels from the stage's left edge to the top-left trim corner of the first page. */
	x: number;
	/** Pixels from the stage's top edge to the same corner. */
	y: number;
}

/**
 * Reads the position of the spread's trim origin from where the page is drawn in `stage`, so a
 * ruler agrees with the page to the pixel however it got there. `bleed` is how many points the
 * page's own box extends past the origin, `scale` is pixels a point.
 */
export function usePageOrigin(
	stage: RefObject<HTMLElement | null>,
	scale: number,
	bleed: number,
): PageOrigin {
	const [origin, setOrigin] = useState<PageOrigin>({ x: 0, y: 0 });

	useLayoutEffect(() => {
		const el = stage.current;
		if (el === null) return;
		const read = () => {
			const page = el.querySelector('svg[role="group"]');
			if (page === null) return;
			const here = el.getBoundingClientRect();
			const there = page.getBoundingClientRect();
			const next = {
				x: there.left - here.left + bleed * scale,
				y: there.top - here.top + bleed * scale,
			};
			// Half a pixel is not worth another render, and keeps a settled layout from looping.
			setOrigin((now) =>
				Math.abs(now.x - next.x) < 0.5 && Math.abs(now.y - next.y) < 0.5 ? now : next,
			);
		};
		read();
		el.addEventListener('scroll', read, { passive: true });
		window.addEventListener('resize', read);
		return () => {
			el.removeEventListener('scroll', read);
			window.removeEventListener('resize', read);
		};
	});

	return origin;
}
