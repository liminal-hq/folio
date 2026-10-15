// Tests for finding a page's master items and what hides them
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { Book } from '@liminal-hq/folio-protocol/generated/Book';
import type { Spread } from '@liminal-hq/folio-protocol/generated/Spread';
import { describe, expect, it } from 'vitest';
import { masterAt, masterFurniture, masterItemsAt, pageAt } from './masterItems';

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

describe('masterFurniture', () => {
	const furnishedBook = {
		...book,
		masterPages: [
			{
				id: 'mst_b',
				name: 'Verso',
				frames: [number('n2'), { type: 'shape', id: 'rule', box: { x: 5, y: 6, w: 7, h: 8 } }],
			},
			{
				id: 'mst_a',
				name: 'Recto',
				frames: [{ type: 'shape', id: 'bar', box: { x: 1, y: 2, w: 3, h: 4 } }],
			},
		],
	} as unknown as Book;

	it('moves a master picture or shape to its page, leaves text to the layout, and skips what a page hides', () => {
		const furnished = masterFurniture(furnishedBook, {
			...spread,
			masterOverrides: [{ pageId: 'p2', hide: ['bar'] }],
		} as Spread);
		expect(furnished.map((f) => f.id)).toEqual(['rule@p1']);
		expect(furnished[0]!.box).toEqual({ x: 5, y: 6, w: 7, h: 8 });
	});

	it('offsets the items of the second page by a page width', () => {
		const furnished = masterFurniture(furnishedBook, {
			...spread,
			masterOverrides: [],
			pages: [
				{ id: 'p1', master: 'mst_a' },
				{ id: 'p2', master: 'mst_b' },
			],
		} as unknown as Spread);
		expect(furnished.map((f) => [f.id, f.box.x])).toEqual([
			['bar@p1', 1],
			['rule@p2', 105],
		]);
	});
});

describe('masterAt', () => {
	it('names the master of the page under the point, or none for a page without one', () => {
		expect(masterAt(book, spread, { x: 20 })).toEqual({ id: 'mst_b', name: 'Verso' });
		expect(masterAt(book, spread, { x: 150 })).toEqual({ id: 'mst_a', name: 'Recto' });
		const bare = { ...spread, pages: [{ id: 'p1', master: null }] } as unknown as Spread;
		expect(masterAt(book, bare, { x: 20 })).toBeNull();
	});
});
