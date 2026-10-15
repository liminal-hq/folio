// The outlines of the bubble shapes, drawn in a unit square and scaled to a frame's box
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { Shape } from '@liminal-hq/folio-protocol/generated/Shape';

/** The kinds that are one path in the unit square (the others are drawn as a rect, ellipse or line). */
export type PathShape = Extract<Shape, 'speech' | 'thought' | 'shout'>;

export const isPathShape = (shape: Shape): shape is PathShape =>
	shape === 'speech' || shape === 'thought' || shape === 'shout';

const num = (n: number) => Number(n.toFixed(4));

/** A scalloped cloud: arcs bulging outward between points round an ellipse. */
function cloud(): string {
	const bumps = 11;
	const cx = 0.5;
	const cy = 0.4;
	const rx = 0.46;
	const ry = 0.34;
	const point = (i: number) => {
		const angle = (i / bumps) * Math.PI * 2 - Math.PI / 2;
		return [cx + rx * Math.cos(angle), cy + ry * Math.sin(angle)] as const;
	};
	const [x0, y0] = point(0);
	let d = `M${num(x0)} ${num(y0)}`;
	for (let i = 1; i <= bumps; i++) {
		const [x, y] = point(i % bumps);
		d += `A0.11 0.11 0 0 1 ${num(x)} ${num(y)}`;
	}
	return `${d}Z`;
}

/** A jagged burst: points alternating between an outer and an inner ring round the box's centre. */
function burst(): string {
	const spikes = 12;
	let d = '';
	for (let i = 0; i < spikes * 2; i++) {
		const outer = i % 2 === 0;
		const angle = (i / (spikes * 2)) * Math.PI * 2 - Math.PI / 2;
		const r = outer ? 0.5 : 0.36;
		const x = 0.5 + r * Math.cos(angle);
		const y = 0.5 + r * Math.sin(angle);
		d += `${i === 0 ? 'M' : 'L'}${num(x)} ${num(y)}`;
	}
	return `${d}Z`;
}

/** What each bubble is drawn with, as paths in the unit square. A thought bubble also trails two small clouds. */
export const SHAPE_PATHS: Record<PathShape, string[]> = {
	speech: [
		'M0.12 0H0.88Q1 0 1 0.12V0.58Q1 0.7 0.88 0.7H0.42L0.2 1L0.26 0.7H0.12Q0 0.7 0 0.58V0.12Q0 0 0.12 0Z',
	],
	thought: [
		cloud(),
		'M0.26 0.84a0.06 0.05 0 1 0 0.12 0a0.06 0.05 0 1 0 -0.12 0Z',
		'M0.1 0.95a0.035 0.03 0 1 0 0.07 0a0.035 0.03 0 1 0 -0.07 0Z',
	],
	shout: [burst()],
};

/**
 * Where words sit inside each kind, as a share of the box on each side, so a bubble's text stays in
 * its body and out of its tail.
 */
export const TEXT_INSET: Record<
	Shape,
	{ left: number; top: number; right: number; bottom: number }
> = {
	rect: { left: 0, top: 0, right: 0, bottom: 0 },
	round: { left: 0.04, top: 0.04, right: 0.04, bottom: 0.04 },
	ellipse: { left: 0.15, top: 0.15, right: 0.15, bottom: 0.15 },
	line: { left: 0, top: 0, right: 0, bottom: 0 },
	speech: { left: 0.08, top: 0.08, right: 0.08, bottom: 0.38 },
	thought: { left: 0.18, top: 0.14, right: 0.18, bottom: 0.4 },
	shout: { left: 0.2, top: 0.2, right: 0.2, bottom: 0.2 },
};
