// Tests for the status bar's size and page count
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { Book } from '@liminal-hq/folio-protocol/generated/Book';
import { describe, expect, it } from 'vitest';
import { formatTrim, pageCount } from './format';

describe('formatTrim', () => {
	it('writes inches with the decimals that matter', () => {
		expect(formatTrim({ w: 612, h: 612 }, 'inches')).toBe('8.5 × 8.5 in');
		expect(formatTrim({ w: 360, h: 576 }, 'inches')).toBe('5 × 8 in');
		expect(formatTrim({ w: 594, h: 756 }, 'inches')).toBe('8.25 × 10.5 in');
	});

	it('writes millimetres to a tenth', () => {
		expect(formatTrim({ w: 612, h: 612 }, 'millimetres')).toBe('215.9 × 215.9 mm');
		expect(formatTrim({ w: 360, h: 576 }, 'millimetres')).toBe('127 × 203.2 mm');
	});
});

describe('pageCount', () => {
	it('leaves the cover out', () => {
		const spread = (kind: string, pages: number) => ({ kind, pages: new Array(pages).fill({}) });
		const book = {
			spreads: [spread('cover', 1), spread('opener', 1), spread('spread', 2)],
		} as unknown as Book;
		expect(pageCount(book)).toBe(3);
	});
});
