// Draws a picture in a box, honouring the point of it that should stay in view
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { useEffect, useState } from 'react';
import type { Rect } from './backgrounds';
import { focusedRect, type Size } from './focusedRect';

const known = new Map<string, Size>();

/** The picture's own size once it has loaded, or `null` until then. Remembered by address. */
function useNaturalSize(href: string): Size | null {
	const [size, setSize] = useState<Size | null>(known.get(href) ?? null);
	useEffect(() => {
		const have = known.get(href);
		if (have) {
			setSize(have);
			return;
		}
		let live = true;
		const img = new Image();
		img.onload = () => {
			const found = { w: img.naturalWidth, h: img.naturalHeight };
			known.set(href, found);
			if (live) setSize(found);
		};
		img.src = href;
		return () => {
			live = false;
		};
	}, [href]);
	return size;
}

/**
 * Until the picture's size is known, it is drawn centred by the browser, which is where it ends up
 * when the focus is the middle. The caller clips it to the box.
 */
export function FocusedImage({
	href,
	box,
	fit,
	focus,
}: {
	href: string;
	box: Rect;
	fit: 'fill' | 'fit' | 'stretch';
	focus: readonly [number, number];
}) {
	const size = useNaturalSize(href);
	if (!size) {
		return (
			<image
				href={href}
				x={box.x}
				y={box.y}
				width={box.w}
				height={box.h}
				preserveAspectRatio={
					fit === 'fit' ? 'xMidYMid meet' : fit === 'stretch' ? 'none' : 'xMidYMid slice'
				}
			/>
		);
	}
	const r = focusedRect(box, size, fit, focus);
	return <image href={href} x={r.x} y={r.y} width={r.w} height={r.h} preserveAspectRatio="none" />;
}
