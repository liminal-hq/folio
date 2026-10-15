// Draws a spread's page backgrounds, behind its frames
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { Background } from '@liminal-hq/folio-protocol/generated/Background';
import type { Book } from '@liminal-hq/folio-protocol/generated/Book';
import type { Spread } from '@liminal-hq/folio-protocol/generated/Spread';
import { useId } from 'react';
import { gradientLine, paintRects, resolveBackgrounds, type Rect } from './backgrounds';
import { hasTexture, TexturePattern } from './textures';

/** What a background paints with, and the `<defs>` it needs: a colour, or a gradient by id. */
function Fill({
	bg,
	rect,
	id,
	images,
	children,
}: {
	bg: Background;
	rect: Rect;
	id: string;
	images: Record<string, string>;
	children(fill: string): React.ReactNode;
}) {
	if (bg.type === 'colour') return <>{children(bg.colour)}</>;
	if (bg.type === 'gradient') {
		const line = gradientLine(rect, bg.angle);
		return (
			<>
				<defs>
					<linearGradient id={id} gradientUnits="userSpaceOnUse" {...line}>
						{bg.stops.map((stop, i) => (
							<stop key={i} offset={stop.at} stopColor={stop.colour} />
						))}
					</linearGradient>
				</defs>
				{children(`url(#${id})`)}
			</>
		);
	}
	if (bg.type === 'texture') {
		// A texture this build cannot draw leaves the paper showing.
		if (!hasTexture(bg.id)) return null;
		return (
			<>
				<defs>
					<TexturePattern id={id} texture={bg.id} tint={bg.tint} />
				</defs>
				{children(`url(#${id})`)}
			</>
		);
	}
	// A picture is drawn once it has been read from the book; until then the paper shows. It fills
	// the rectangle (cropped at the edges) or sits whole inside it, centred.
	const href = images[bg.asset];
	if (!href) return null;
	return (
		<image
			href={href}
			x={rect.x}
			y={rect.y}
			width={rect.w}
			height={rect.h}
			preserveAspectRatio={bg.fit === 'fill' ? 'xMidYMid slice' : 'xMidYMid meet'}
		/>
	);
}

/**
 * The background each page shows, drawn into the bleed on the outside edges. One across both pages
 * is drawn once over the spread and cut at each page that takes it, so a gradient runs through the
 * gutter. It never takes a click, so the paper under it still does.
 */
export function BackgroundLayer({
	book,
	spread,
	images = {},
}: {
	book: Book;
	spread: Spread;
	/** An address to draw for each picture's asset path, once it has been read. */
	images?: Record<string, string>;
}) {
	const key = useId().replace(/[^A-Za-z0-9_-]/g, '');
	const resolved = resolveBackgrounds(book.masterPages ?? [], spread);
	const rects = paintRects(book, spread);
	const whole: Rect = {
		x: rects[0]!.x,
		y: rects[0]!.y,
		w: rects.at(-1)!.x + rects.at(-1)!.w - rects[0]!.x,
		h: rects[0]!.h,
	};
	return (
		<g pointerEvents="none" data-backgrounds="">
			{resolved.map((shown, index) => {
				if (!shown) return null;
				const rect = shown.source === 'span' ? whole : rects[index]!;
				const clip = `${key}-clip-${index}`;
				return (
					<g key={index} data-background={shown.source} clipPath={`url(#${clip})`}>
						<defs>
							<clipPath id={clip}>
								<rect
									x={rects[index]!.x}
									y={rects[index]!.y}
									width={rects[index]!.w}
									height={rects[index]!.h}
								/>
							</clipPath>
						</defs>
						<Fill bg={shown.bg} rect={rect} id={`${key}-bg-${index}`} images={images}>
							{(fill) => <rect x={rect.x} y={rect.y} width={rect.w} height={rect.h} fill={fill} />}
						</Fill>
					</g>
				);
			})}
		</g>
	);
}
