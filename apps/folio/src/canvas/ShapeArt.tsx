// Draws a shape frame: its fill and outline in its box
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { ShapeFrame } from '@liminal-hq/folio-protocol/generated/ShapeFrame';
import { isPathShape, SHAPE_PATHS } from './shapePaths';

/** The outline width a shape with an outline but no width of its own has, in points. */
export const DEFAULT_STROKE = 1;
/** How round a `round` shape's corners are when it has no radius of its own, as a share of its short side. */
const DEFAULT_ROUND = 0.12;

/**
 * A shape's look: no fill when it has none, and an outline only when it has a colour for one. The
 * outline is drawn with a non-scaling stroke, so it keeps its width however the frame is stretched.
 */
export function ShapeArt({ frame }: { frame: ShapeFrame }) {
	const { x, y, w, h } = frame.box;
	const paint = {
		fill: frame.fill ?? 'none',
		stroke: frame.stroke ?? 'none',
		strokeWidth: frame.stroke ? (frame.strokeWidth ?? DEFAULT_STROKE) : 0,
		vectorEffect: 'non-scaling-stroke' as const,
		strokeLinejoin: 'round' as const,
		strokeLinecap: 'round' as const,
	};
	if (frame.shape === 'ellipse') {
		return <ellipse cx={x + w / 2} cy={y + h / 2} rx={w / 2} ry={h / 2} {...paint} />;
	}
	if (frame.shape === 'line') {
		// A line is drawn in its outline colour, which a fill cannot show.
		const stroke = frame.stroke ?? frame.fill ?? '#14110f';
		const flipped = frame.flip === true;
		return (
			<line
				x1={x}
				x2={x + w}
				y1={flipped ? y + h : y}
				y2={flipped ? y : y + h}
				{...paint}
				fill="none"
				stroke={stroke}
				strokeWidth={frame.strokeWidth ?? 2}
			/>
		);
	}
	if (isPathShape(frame.shape)) {
		return (
			<g transform={`translate(${x} ${y}) scale(${w} ${h})`}>
				{SHAPE_PATHS[frame.shape].map((d, i) => (
					<path key={i} d={d} {...paint} />
				))}
			</g>
		);
	}
	const radius = frame.shape === 'round' ? (frame.radius ?? Math.min(w, h) * DEFAULT_ROUND) : 0;
	return <rect x={x} y={y} width={w} height={h} rx={radius} {...paint} />;
}
