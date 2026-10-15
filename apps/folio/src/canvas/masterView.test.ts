// Tests for showing a master page as a spread and addressing its edits
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { Book } from '@liminal-hq/folio-protocol/generated/Book';
import type { Spread } from '@liminal-hq/folio-protocol/generated/Spread';
import { describe, expect, it } from 'vitest';
import { isMasterSpread, masterOfPage, masterSpread, surfaceOf } from './masterView';

const book = {
	masterPages: [
		{
			id: 'mst_a',
			name: 'A',
			frames: [{ type: 'text', id: 'n1' }],
			guides: [{ axis: 'y', pos: 36, locked: false }],
		},
		{ id: 'mst_b', name: 'B', frames: [] },
	],
} as unknown as Book;

describe('masterSpread', () => {
	it('shows a master as a one-page spread of its frames and guides', () => {
		const shown = masterSpread(book, 'mst_a')!;
		expect(shown.pages).toHaveLength(1);
		expect(shown.frames.map((f) => f.id)).toEqual(['n1']);
		expect(shown.guides).toEqual([{ axis: 'y', pos: 36, locked: false }]);
		expect(shown.kind).toBe('single');
	});

	it('shows master B as a left-hand page, and a master with no guides as having none', () => {
		const shown = masterSpread(book, 'mst_b')!;
		expect(shown.kind).toBe('closer');
		expect(shown.guides).toEqual([]);
	});

	it('is nothing for a master the book does not have', () => {
		expect(masterSpread(book, 'mst_z')).toBeNull();
	});
});

describe('surfaceOf', () => {
	it('addresses a shown master by its id, and an ordinary spread by its own', () => {
		expect(surfaceOf(masterSpread(book, 'mst_a')!)).toEqual({ master: 'mst_a' });
		expect(surfaceOf({ id: 'sp_1' } as Spread)).toEqual({ spread: 'sp_1' });
		expect(isMasterSpread(masterSpread(book, 'mst_a')!)).toBe(true);
		expect(isMasterSpread({ id: 'sp_1' } as Spread)).toBe(false);
	});
});

describe('masterOfPage', () => {
	it('is the master a page uses, or null for a page without one', () => {
		const spread = { pages: [{ master: null }, { master: 'mst_a' }] } as unknown as Spread;
		expect(masterOfPage(spread, 0)).toBeNull();
		expect(masterOfPage(spread, 1)).toBe('mst_a');
	});
});
