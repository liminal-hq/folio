// Tests for the page shelf's labels
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { Book } from '@liminal-hq/folio-protocol/generated/Book';
import { describe, expect, it } from 'vitest';
import { shelfLabels } from './shelfLabels';

const spread = (kind: string, pages: number) => ({
	id: `s${Math.random()}`,
	kind,
	pages: Array.from({ length: pages }, (_, i) => ({ id: `p${i}`, master: null })),
	frames: [],
	guides: [],
	masterOverrides: [],
});

describe('shelfLabels', () => {
	it('numbers pages from the first one after the cover', () => {
		const book = {
			spreads: [
				spread('cover', 1),
				spread('opener', 1),
				spread('spread', 2),
				spread('spread', 2),
				spread('closer', 1),
			],
		} as unknown as Book;
		expect(shelfLabels(book)).toEqual(['Cover', 'Page 1', 'Pages 2–3', 'Pages 4–5', 'Page 6']);
	});

	it('is empty for a book with no spreads', () => {
		expect(shelfLabels({ spreads: [] } as unknown as Book)).toEqual([]);
	});
});
