// Tests for resolving a spread's backgrounds
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { Book } from '@liminal-hq/folio-protocol/generated/Book';
import type { Master } from '@liminal-hq/folio-protocol/generated/Master';
import type { Spread } from '@liminal-hq/folio-protocol/generated/Spread';
import { describe, expect, it } from 'vitest';
import { backgroundAssets, resolveBackgrounds } from './backgrounds';
import cases from './fixtures/backgroundCases.json';

describe('resolveBackgrounds', () => {
	// The Rust `resolve` is tested against the same file, so the two cannot drift apart.
	it.each(cases)('$name', ({ masters, spread, expected }) => {
		expect(resolveBackgrounds(masters as unknown as Master[], spread as unknown as Spread)).toEqual(
			expected,
		);
	});
});

describe('backgroundAssets', () => {
	it('lists each picture the backgrounds use once, from pages, spans and masters', () => {
		const picture = (asset: string) => ({ type: 'picture', asset, fit: 'fill', focus: [0.5, 0.5] });
		const book = {
			masterPages: [{ id: 'm', bg: picture('assets/images/c.jpg') }],
			spreads: [
				{
					pages: [{ id: 'p', bg: picture('assets/images/a.jpg') }, { id: 'q' }],
					bgSpan: picture('assets/images/b.jpg'),
				},
				{ pages: [{ id: 'r', bg: picture('assets/images/a.jpg') }] },
			],
		} as unknown as Book;
		expect(backgroundAssets(book).sort()).toEqual([
			'assets/images/a.jpg',
			'assets/images/b.jpg',
			'assets/images/c.jpg',
		]);
	});
});
