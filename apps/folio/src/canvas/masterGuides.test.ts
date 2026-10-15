// Tests for where master guides land on a spread and which guides can go on every page
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { Book } from '@liminal-hq/folio-protocol/generated/Book';
import type { Spread } from '@liminal-hq/folio-protocol/generated/Spread';
import { describe, expect, it } from 'vitest';
import { canPromote, masterGuidesOn } from './masterGuides';
import { masterSpread } from './masterView';

const book = {
	trim: { w: 100, h: 150 },
	masterPages: [
		{
			id: 'mst_a',
			name: 'A',
			frames: [],
			guides: [
				{ axis: 'x', pos: 30, locked: false },
				{ axis: 'y', pos: 40, locked: false },
			],
		},
		{ id: 'mst_b', name: 'B', frames: [], guides: [{ axis: 'x', pos: 70, locked: false }] },
	],
} as unknown as Book;

const spread = {
	id: 's1',
	kind: 'spread',
	pages: [
		{ id: 'p1', master: 'mst_b' },
		{ id: 'p2', master: 'mst_a' },
	],
	frames: [],
	guides: [],
} as unknown as Spread;

describe('masterGuidesOn', () => {
	it('places a vertical guide by its page and a horizontal one at its height', () => {
		expect(masterGuidesOn(book, spread)).toEqual([
			{ axis: 'x', pos: 70, page: 0, master: 'mst_b' },
			{ axis: 'x', pos: 130, page: 1, master: 'mst_a' },
			{ axis: 'y', pos: 40, page: 1, master: 'mst_a' },
		]);
	});

	it('gives nothing for a page without a master, or for a master being edited', () => {
		const bare = { ...spread, pages: [{ id: 'p1', master: null }] } as unknown as Spread;
		expect(masterGuidesOn(book, bare)).toEqual([]);
		expect(masterGuidesOn(book, masterSpread(book, 'mst_a')!)).toEqual([]);
	});
});

describe('canPromote', () => {
	const guide = (axis: 'x' | 'y', pos: number, locked = false) => ({ axis, pos, locked }) as never;

	it('allows a horizontal guide, and a vertical one over a page', () => {
		expect(canPromote(book, spread, guide('y', 12))).toBe(true);
		expect(canPromote(book, spread, guide('x', 130))).toBe(true);
		expect(canPromote(book, spread, guide('x', 200))).toBe(true);
	});

	it('refuses the gutter, off the pages, the cover, a locked guide and a master being edited', () => {
		expect(canPromote(book, spread, guide('x', 100))).toBe(false);
		expect(canPromote(book, spread, guide('x', 201))).toBe(false);
		expect(canPromote(book, spread, guide('x', -1))).toBe(false);
		expect(canPromote(book, { ...spread, kind: 'cover' } as Spread, guide('y', 5))).toBe(false);
		expect(canPromote(book, spread, guide('y', 5, true))).toBe(false);
		expect(canPromote(book, masterSpread(book, 'mst_a')!, guide('y', 5))).toBe(false);
	});
});
