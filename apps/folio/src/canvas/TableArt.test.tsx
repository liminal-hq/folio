// Tests for drawing a table's rules and tints
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { TableFrame } from '@liminal-hq/folio-protocol/generated/TableFrame';
import { render } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { columnEdges, TableArt } from './TableArt';

const table = (extra: Partial<TableFrame> = {}) =>
	({
		type: 'table',
		id: 'tb',
		box: { x: 10, y: 20, w: 200, h: 90 },
		rotation: 0,
		locked: false,
		cols: [0.25, 0.75],
		rows: [
			[{}, {}],
			[{}, {}],
			[{}, {}],
		],
		head: 1,
		look: 'lined',
		...extra,
	}) as unknown as TableFrame;

const draw = (frame: TableFrame) =>
	render(
		<svg>
			<TableArt frame={frame} />
		</svg>,
	).container;

describe('TableArt', () => {
	it('finds each column edge from the shares', () => {
		expect(columnEdges(table())).toEqual([10, 60, 210]);
	});

	it('rules under every row when lined', () => {
		const lines = [...draw(table()).querySelectorAll('line')];
		expect(lines.map((l) => l.getAttribute('y1'))).toEqual(['50', '80', '110']);
		expect(
			lines.every((l) => l.getAttribute('x1') === '10' && l.getAttribute('x2') === '210'),
		).toBe(true);
	});

	it('rules round every cell when a grid', () => {
		const lines = draw(table({ look: 'grid' })).querySelectorAll('line');
		// Four rules across, three down.
		expect(lines).toHaveLength(4 + 3);
	});

	it('rules the header and tints every other row below it when a recipe', () => {
		const frame = table({
			look: 'recipe',
			rows: Array.from({ length: 5 }, () => [{}, {}]) as never,
		});
		const out = draw(frame);
		expect(out.querySelectorAll('line')).toHaveLength(1);
		const tinted = [...out.querySelectorAll('rect')].map((r) => r.getAttribute('y'));
		// Rows 2 and 4 (counting from 0 under the one header row) are tinted: 20 + 2 × 18 and 20 + 4 × 18.
		expect(tinted).toEqual(['56', '92']);
	});
});
