// Tests for finding a table's cells
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { TableFrame } from '@liminal-hq/folio-protocol/generated/TableFrame';
import { describe, expect, it } from 'vitest';
import { cellAt, cellId, cellText, nextCell, parseCellId } from './tableCells';

const table = (extra: Partial<TableFrame> = {}) =>
	({
		type: 'table',
		id: 'tb',
		box: { x: 100, y: 100, w: 200, h: 90 },
		rotation: 0,
		cols: [0.25, 0.75],
		rows: [
			[{}, {}],
			[{}, {}],
			[{}, {}],
		],
		head: 0,
		look: 'lined',
		...extra,
	}) as unknown as TableFrame;

describe('cell ids', () => {
	it('round-trips, and says no to what is not one', () => {
		expect(cellId({ frame: 'fr_a#b', row: 2, col: 11 })).toBe('fr_a#b#2.11');
		expect(parseCellId('fr_a#b#2.11')).toEqual({ frame: 'fr_a#b', row: 2, col: 11 });
		expect(parseCellId('fr_a')).toBeNull();
	});
});

describe('cellAt', () => {
	it('finds the row and column under a point', () => {
		expect(cellAt(table(), { x: 110, y: 110 })).toEqual({ frame: 'tb', row: 0, col: 0 });
		expect(cellAt(table(), { x: 160, y: 140 })).toEqual({ frame: 'tb', row: 1, col: 1 });
		expect(cellAt(table(), { x: 299, y: 189 })).toEqual({ frame: 'tb', row: 2, col: 1 });
	});

	it('is null outside the table', () => {
		expect(cellAt(table(), { x: 99, y: 150 })).toBeNull();
		expect(cellAt(table(), { x: 150, y: 191 })).toBeNull();
	});

	it('reads a point in the table’s own turned axes', () => {
		// Turned a half turn, the top-left cell is at the bottom right of the box.
		expect(cellAt(table({ rotation: 180 }), { x: 290, y: 180 })).toEqual({
			frame: 'tb',
			row: 0,
			col: 0,
		});
	});
});

describe('nextCell', () => {
	it('runs along a row and on to the next, and off the ends', () => {
		const t = table();
		expect(nextCell(t, { frame: 'tb', row: 0, col: 0 }, false)).toEqual({
			frame: 'tb',
			row: 0,
			col: 1,
		});
		expect(nextCell(t, { frame: 'tb', row: 0, col: 1 }, false)).toEqual({
			frame: 'tb',
			row: 1,
			col: 0,
		});
		expect(nextCell(t, { frame: 'tb', row: 1, col: 0 }, true)).toEqual({
			frame: 'tb',
			row: 0,
			col: 1,
		});
		expect(nextCell(t, { frame: 'tb', row: 0, col: 0 }, true)).toBeNull();
		expect(nextCell(t, { frame: 'tb', row: 2, col: 1 }, false)).toBeNull();
	});
});

describe('cellText', () => {
	it('is the plain words, empty for none, and null when emphasised', () => {
		const t = table({
			rows: [
				[{ content: { inline: [{ text: 'a' }, { text: 'b' }] } }, {}],
				[{ content: { inline: [{ text: 'x', bold: true }] } }, {}],
			],
		} as never);
		expect(cellText(t, 0, 0)).toBe('ab');
		expect(cellText(t, 0, 1)).toBe('');
		expect(cellText(t, 1, 0)).toBeNull();
	});
});
