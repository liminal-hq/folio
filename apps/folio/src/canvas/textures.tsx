// The textures a page background can have, drawn as SVG patterns of a fixed size on the page
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { ReactNode } from 'react';

interface Drawing {
	/** The size of one tile in points, so a texture is the same size on every book and page. */
	size: number;
	/** What the tile is without a tint. */
	base: string;
	/** The marks over the base. Drawn in black or white at low strength so a tint shows through. */
	marks(id: string, size: number): ReactNode;
}

const stroke = (opacity: number, width = 0.5) => ({
	stroke: '#000',
	strokeOpacity: opacity,
	strokeWidth: width,
});

const DRAWINGS: Record<string, Drawing> = {
	linen: {
		size: 12,
		base: '#ece5d3',
		marks: (_id, s) => (
			<>
				<path d={`M0 ${s / 4}H${s}M0 ${(s * 3) / 4}H${s}`} {...stroke(0.1)} />
				<path d={`M${s / 4} 0V${s}M${(s * 3) / 4} 0V${s}`} {...stroke(0.08)} />
			</>
		),
	},
	'laid-paper': {
		size: 36,
		base: '#f2eee3',
		marks: (_id, s) => (
			<>
				<path
					d={Array.from({ length: 12 }, (_, i) => `M0 ${i * 3 + 1}H${s}`).join('')}
					{...stroke(0.07, 0.4)}
				/>
				<path d={`M0 0V${s}`} {...stroke(0.12, 0.8)} />
			</>
		),
	},
	kraft: {
		size: 120,
		base: '#c8a27a',
		marks: (id, s) => (
			<>
				<filter id={`${id}-noise`} x="0" y="0" width="100%" height="100%">
					<feTurbulence type="fractalNoise" baseFrequency="0.9" numOctaves="2" seed="3" />
					<feColorMatrix type="saturate" values="0" />
				</filter>
				<rect width={s} height={s} filter={`url(#${id}-noise)`} opacity="0.4" />
			</>
		),
	},
	watercolour: {
		// The blobs stay a few blurs in from the tile's edge, so the blur is not cut where tiles meet.
		size: 200,
		base: '#e3eef2',
		marks: (id, s) => (
			<>
				<filter id={`${id}-blur`} x="-30%" y="-30%" width="160%" height="160%">
					<feGaussianBlur stdDeviation={s / 25} />
				</filter>
				<g filter={`url(#${id}-blur)`}>
					<ellipse
						cx={s * 0.34}
						cy={s * 0.34}
						rx={s * 0.14}
						ry={s * 0.11}
						fill="#8fc0d2"
						opacity="0.55"
					/>
					<ellipse
						cx={s * 0.64}
						cy={s * 0.5}
						rx={s * 0.15}
						ry={s * 0.12}
						fill="#efb9c4"
						opacity="0.55"
					/>
					<ellipse
						cx={s * 0.42}
						cy={s * 0.68}
						rx={s * 0.16}
						ry={s * 0.1}
						fill="#bfdcaa"
						opacity="0.55"
					/>
				</g>
			</>
		),
	},
	'dot-grid': {
		size: 14,
		base: 'none',
		marks: (_id, s) => <circle cx={s / 2} cy={s / 2} r="0.9" fill="#000" fillOpacity="0.28" />,
	},
};

/** Whether the app can draw a texture of this id. A book from a newer build may name one it cannot. */
export const hasTexture = (id: string) => id in DRAWINGS;

/**
 * The `<pattern>` for a texture, to be used as `url(#id)`. A tint replaces the base colour and the
 * marks stay over it. The tiles are laid out from the spread's origin, so a texture runs on through
 * the gutter.
 */
export function TexturePattern({
	id,
	texture,
	tint,
}: {
	id: string;
	texture: string;
	tint?: string | null;
}) {
	const drawing = DRAWINGS[texture];
	if (!drawing) return null;
	const base = tint ?? drawing.base;
	return (
		<pattern id={id} width={drawing.size} height={drawing.size} patternUnits="userSpaceOnUse">
			{base !== 'none' && <rect width={drawing.size} height={drawing.size} fill={base} />}
			{drawing.marks(id, drawing.size)}
		</pattern>
	);
}
