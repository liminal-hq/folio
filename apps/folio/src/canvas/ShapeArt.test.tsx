// Tests for drawing a shape: its fill, outline and outline of its kind
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { ShapeFrame } from '@liminal-hq/folio-protocol/generated/ShapeFrame';
import { render } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { ShapeArt } from './ShapeArt';
import { SHAPE_PATHS, TEXT_INSET } from './shapePaths';

const shape = (over: Record<string, unknown>) =>
	({
		id: 's',
		box: { x: 10, y: 20, w: 100, h: 50 },
		rotation: 0,
		locked: false,
		fill: '#ffffff',
		style: null,
		shape: 'rect',
		...over,
	}) as unknown as ShapeFrame;

const draw = (frame: ShapeFrame) =>
	render(
		<svg>
			<ShapeArt frame={frame} />
		</svg>,
	).container;

describe('ShapeArt', () => {
	it('draws no fill when there is none and no outline when there is no colour for one', () => {
		const rect = draw(shape({ fill: null })).querySelector('rect')!;
		expect(rect.getAttribute('fill')).toBe('none');
		expect(rect.getAttribute('stroke')).toBe('none');
	});

	it('draws an outline that does not scale with the frame', () => {
		const rect = draw(shape({ stroke: '#14110f', strokeWidth: 3 })).querySelector('rect')!;
		expect(rect.getAttribute('stroke')).toBe('#14110f');
		expect(rect.getAttribute('stroke-width')).toBe('3');
		expect(rect.getAttribute('vector-effect')).toBe('non-scaling-stroke');
	});

	it('rounds a round shape by its radius, or by a share of its short side', () => {
		expect(
			draw(shape({ shape: 'round', radius: 8 }))
				.querySelector('rect')!
				.getAttribute('rx'),
		).toBe('8');
		expect(
			draw(shape({ shape: 'round' }))
				.querySelector('rect')!
				.getAttribute('rx'),
		).toBe('6');
		expect(
			draw(shape({ shape: 'rect' }))
				.querySelector('rect')!
				.getAttribute('rx'),
		).toBe('0');
	});

	it('draws a line across its box, or the other diagonal when flipped, even with no height', () => {
		const flat = draw(
			shape({ shape: 'line', fill: null, box: { x: 0, y: 10, w: 100, h: 0 } }),
		).querySelector('line')!;
		expect([flat.getAttribute('y1'), flat.getAttribute('y2')]).toEqual(['10', '10']);
		const rising = draw(shape({ shape: 'line', fill: null, flip: true })).querySelector('line')!;
		expect([rising.getAttribute('y1'), rising.getAttribute('y2')]).toEqual(['70', '20']);
	});

	it('scales a bubble from its unit path to its box', () => {
		const g = draw(shape({ shape: 'speech' })).querySelector('g')!;
		expect(g.getAttribute('transform')).toBe('translate(10 20) scale(100 50)');
		expect(g.querySelectorAll('path')).toHaveLength(1);
		expect(draw(shape({ shape: 'thought' })).querySelectorAll('path')).toHaveLength(3);
	});
});

describe('shape paths', () => {
	it('are closed and start with a move, so each can be filled', () => {
		for (const paths of Object.values(SHAPE_PATHS)) {
			for (const d of paths) {
				expect(d.startsWith('M')).toBe(true);
				expect(d.endsWith('Z')).toBe(true);
			}
		}
	});

	it('give a bubble its tail below the body and a cloud its trailing circles', () => {
		expect(SHAPE_PATHS.speech).toHaveLength(1);
		expect(SHAPE_PATHS.speech[0]).toContain('L0.2 1');
		expect(SHAPE_PATHS.thought).toHaveLength(3);
		expect(SHAPE_PATHS.shout[0]!.match(/L/g)).toHaveLength(23);
	});
});
