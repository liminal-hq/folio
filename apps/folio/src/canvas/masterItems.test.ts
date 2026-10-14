// Tests for finding a page's master items and what hides them
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { Book } from '@liminal-hq/folio-protocol/generated/Book';
import type { Spread } from '@liminal-hq/folio-protocol/generated/Spread';
import { describe, expect, it } from 'vitest';
import { masterItemsAt, pageAt } from './masterItems';

const number = (id: string) => ({
	type: 'text',
	id,
	box: { x: 0, y: 0, w: 10, h: 10 },
	content: { inline: [{ text: '{page}' }] },
});

const book = {
	trim: { w: 100, h: 100 },
	bleed: 9,
	margins: { top: 10, bottom: 10, inside: 10, outside: 10 },
	masterPages: [
		{ id: 'mst_a', name: 'Recto', frames: [number('n1')] },
		{ id: 'mst_b', name: 'Verso', frames: [number('n2'), { type: 'shape', id: 'rule' }] },
	],
} as unknown as Book;

const spread = {
	id: 's1',
	kind: 'standard',
	pages: [
		{ id: 'p1', master: 'mst_b' },
		{ id: 'p2', master: 'mst_a' },
	],
	frames: [],
	masterOverrides: [{ pageId: 'p2', hide: ['n1'] }],
} as unknown as Spread;

describe('pageAt', () => {
	it('names the page a point is over, and the nearest when it is off the pages', () => {
		expect(pageAt(book, spread, { x: 30 })).toBe(0);
		expect(pageAt(book, spread, { x: 130 })).toBe(1);
		expect(pageAt(book, spread, { x: -5 })).toBe(0);
		expect(pageAt(book, spread, { x: 400 })).toBe(1);
	});
});

describe('masterItemsAt', () => {
	it('lists the master frames of the page under the point', () => {
		const items = masterItemsAt(book, spread, { x: 30 });
		expect(items.map((i) => [i.page, i.frame, i.hidden])).toEqual([
			['p1', 'n2', false],
			['p1', 'rule', false],
		]);
		expect(items[0]!.label).toBe('the page number');
		expect(items[1]!.label).toBe('Verso');
	});

	it('marks the ones this page hides', () => {
		expect(masterItemsAt(book, spread, { x: 130 })).toMatchObject([
			{ page: 'p2', frame: 'n1', hidden: true },
		]);
	});

	it('lists nothing for a page without a master, or a book without any', () => {
		const bare = { ...spread, pages: [{ id: 'p1', master: null }] } as unknown as Spread;
		expect(masterItemsAt(book, bare, { x: 10 })).toEqual([]);
		expect(masterItemsAt({ ...book, masterPages: undefined } as Book, spread, { x: 10 })).toEqual(
			[],
		);
	});
});
