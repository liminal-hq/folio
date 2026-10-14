// Tests for the spread geometry and the zoom ladder
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { Book } from '@liminal-hq/folio-protocol/generated/Book';
import type { Spread } from '@liminal-hq/folio-protocol/generated/Spread';
import { describe, expect, it } from 'vitest';
import { fitZoom, spreadGeometry, stepZoom } from './geometry';

const book = {
	trim: { w: 100, h: 200 },
	bleed: 9,
	margins: { top: 10, bottom: 12, inside: 20, outside: 6 },
} as unknown as Book;

const spread = (kind: Spread['kind'], pages: number): Spread =>
	({ id: 's', kind, pages: Array.from({ length: pages }, () => ({})), frames: [] }) as never;

describe('spreadGeometry', () => {
	it('puts a facing pair side by side with the gutter between them', () => {
		const g = spreadGeometry(book, spread('spread', 2));
		expect(g.pages.map((p) => p.trim.x)).toEqual([0, 100]);
		expect(g.width).toBe(200);
		expect(g.bounds).toEqual({ x: -9, y: -9, w: 218, h: 218 });
	});

	it('keeps the inside margin at the gutter on both pages', () => {
		const [left, right] = spreadGeometry(book, spread('spread', 2)).pages;
		expect(left!.safe).toEqual({ x: 6, y: 10, w: 74, h: 178 });
		expect(right!.safe).toEqual({ x: 120, y: 10, w: 74, h: 178 });
	});

	it('treats a lone page as a right-hand page unless it closes the book', () => {
		expect(spreadGeometry(book, spread('opener', 1)).pages[0]!.safe.x).toBe(20);
		expect(spreadGeometry(book, spread('closer', 1)).pages[0]!.safe.x).toBe(6);
	});
});

describe('zoom', () => {
	it('fits the bounds in the stage with padding', () => {
		const zoom = fitZoom({ x: 0, y: 0, w: 96, h: 96 }, { w: 1000, h: 500 });
		// 96 pt is 128 px at 100%, and 436 px of height is left.
		expect(zoom).toBeCloseTo(436 / 128);
	});

	it('steps through familiar levels and stops at the ends', () => {
		expect(stepZoom(1, 1)).toBe(1.25);
		expect(stepZoom(1, -1)).toBe(0.75);
		expect(stepZoom(0.52, 1)).toBe(0.67);
		expect(stepZoom(4, 1)).toBe(4);
		expect(stepZoom(0.1, -1)).toBe(0.1);
	});
});
