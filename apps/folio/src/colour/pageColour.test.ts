// Tests for reading the colour of a page
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { Book } from '@liminal-hq/folio-protocol/generated/Book';
import type { Spread } from '@liminal-hq/folio-protocol/generated/Spread';
import { describe, expect, it } from 'vitest';
import { pageColourOf } from './pageColour';

const book = {
	styles: { swatches: [{ id: 'cream', name: 'Cream', display: '#f6efe0' }] },
	masterPages: [],
} as unknown as Book;
const spread = (pages: unknown[], bgSpan?: unknown) => ({ pages, bgSpan }) as unknown as Spread;

describe('pageColourOf', () => {
	it('is white when nothing is set', () => {
		expect(pageColourOf(spread([{ id: 'p1' }]), book)).toBe('#ffffff');
	});

	it('reads the first page colour, and resolves a swatch', () => {
		const solid = { type: 'colour', colour: '#112233' };
		expect(pageColourOf(spread([{ id: 'p1', bg: solid }]), book)).toBe('#112233');
		const swatch = { type: 'colour', colour: 'swatch:cream' };
		expect(pageColourOf(spread([{ id: 'p1', bg: swatch }]), book)).toBe('#f6efe0');
	});

	it('falls back to the spread background, and to white for a texture', () => {
		expect(pageColourOf(spread([{ id: 'p1' }], { type: 'colour', colour: '#abcdef' }), book)).toBe(
			'#abcdef',
		);
		expect(pageColourOf(spread([{ id: 'p1', bg: { type: 'texture', id: 'linen' } }]), book)).toBe(
			'#ffffff',
		);
	});
});
