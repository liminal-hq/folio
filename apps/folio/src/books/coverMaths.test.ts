// Tests for the cover's width, pinned to the same numbers as `folio-model`'s cover maths
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { Book } from '@liminal-hq/folio-protocol/generated/Book';
import { describe, expect, it } from 'vitest';
import { coverWidth, spinePages, spineWidth } from './coverMaths';

function book(pages: number, paper?: 'white' | 'cream' | 'colour'): Book {
	const spread = (count: number) => ({ pages: Array.from({ length: count }, () => ({})) });
	return {
		trim: { w: 612, h: 612 },
		paper,
		spreads: [spread(1), ...Array.from({ length: pages / 2 }, () => spread(2))],
	} as unknown as Book;
}

describe('the cover maths', () => {
	// The same cases are in `folio-model`'s `the_cover_is_as_wide_as_the_pages_and_the_paper_make_it`.
	it('counts the interior in twos, at least 24, and not the cover', () => {
		expect(spinePages(book(10))).toBe(24);
		expect(spinePages(book(32))).toBe(32);
	});

	it('makes the spine from the pages and the paper', () => {
		expect(spineWidth(book(32))).toBeCloseTo(32 * 0.002252 * 72, 9);
		expect(spineWidth(book(32, 'cream'))).toBeCloseTo(32 * 0.0025 * 72, 9);
	});

	it('is two trim widths and a spine wide', () => {
		expect(coverWidth(book(32, 'colour'))).toBeCloseTo(2 * 612 + 32 * 0.002347 * 72, 9);
	});
});
