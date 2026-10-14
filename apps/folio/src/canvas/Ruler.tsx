// A ruler along the top or left of the stage: ticks in the person's unit, the selection tinted
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { RulerUnit } from '@liminal-hq/folio-protocol/generated/RulerUnit';
import type { MouseEvent, PointerEvent } from 'react';
import { t } from '../i18n/messages';
import styles from './Ruler.module.css';
import { rulerTicks } from './rulerMath';

/** How wide a ruler is, in pixels. */
export const RULER_SIZE = 20;

export interface RulerProps {
	/** `x` runs along the top; `y` runs down the left. */
	axis: 'x' | 'y';
	unit: RulerUnit;
	/** Screen pixels per point. */
	scale: number;
	/** Pixels from the ruler's start to the spread's zero, which can be off the ruler. */
	origin: number;
	/** How long the ruler is, in pixels. */
	length: number;
	/** What the selection covers along this axis, in points. */
	extent?: { from: number; to: number } | null;
	onPointerDown?(event: PointerEvent<SVGSVGElement>): void;
	onContextMenu?(event: MouseEvent<SVGSVGElement>): void;
}

export function Ruler({
	axis,
	unit,
	scale,
	origin,
	length,
	extent,
	onPointerDown,
	onContextMenu,
}: RulerProps) {
	const horizontal = axis === 'x';
	const ticks = rulerTicks(-origin / scale, (length - origin) / scale, scale, unit);
	const width = horizontal ? length : RULER_SIZE;
	const height = horizontal ? RULER_SIZE : length;
	const at = (points: number) => origin + points * scale;

	return (
		<svg
			className={styles.ruler}
			data-ruler={axis}
			width={width}
			height={height}
			role="img"
			aria-label={t(horizontal ? 'ruler.horizontal' : 'ruler.vertical')}
			onPointerDown={onPointerDown}
			onContextMenu={onContextMenu}
		>
			{extent && extent.to > extent.from && (
				<rect
					data-extent
					className={styles.extent}
					x={horizontal ? at(extent.from) : 0}
					y={horizontal ? 0 : at(extent.from)}
					width={horizontal ? (extent.to - extent.from) * scale : RULER_SIZE}
					height={horizontal ? RULER_SIZE : (extent.to - extent.from) * scale}
				/>
			)}
			{ticks.map((tick) => {
				const p = at(tick.pos);
				const size = tick.major ? RULER_SIZE * 0.55 : RULER_SIZE * 0.25;
				return (
					<g key={tick.pos}>
						<line
							className={styles.tick}
							x1={horizontal ? p : RULER_SIZE - size}
							x2={horizontal ? p : RULER_SIZE}
							y1={horizontal ? RULER_SIZE - size : p}
							y2={horizontal ? RULER_SIZE : p}
						/>
						{tick.label !== undefined && (
							<text
								className={styles.label}
								x={horizontal ? p + 3 : 9}
								y={horizontal ? 9 : p - 3}
								transform={horizontal ? undefined : `rotate(-90 9 ${p - 3})`}
							>
								{tick.label}
							</text>
						)}
					</g>
				);
			})}
		</svg>
	);
}
