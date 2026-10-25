// Tests for the swatch helpers
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { Book } from '@liminal-hq/folio-protocol/generated/Book';
import { describe, expect, it } from 'vitest';
import { inkToHex, newSwatchId, pickerSwatches } from './bookSwatches';

const book = {
	styles: {
		look: '',
		fontFamily: '',
		styles: {},
		swatches: [
			{ id: 'brand', name: 'Brand', display: '#e60017', ink: { c: 0, m: 100, y: 100, k: 0 } },
		],
	},
} as unknown as Book;

describe('the book swatches', () => {
	it('are offered after the presets, as references drawn in their display colour', () => {
		const list = pickerSwatches(book, { swatches: [{ name: 'Ink', colour: '#1c1b20' }] } as never);
		expect(list).toEqual([
			{ name: 'Ink', colour: '#1c1b20' },
			{ name: 'Brand', colour: 'swatch:brand', shown: '#e60017' },
		]);
		expect(pickerSwatches({} as Book, null)).toEqual([]);
	});

	it('show ink as the plain inverse, as Rust does', () => {
		expect(inkToHex({ c: 0, m: 0, y: 0, k: 100 })).toBe('#000000');
		expect(inkToHex({ c: 100, m: 0, y: 0, k: 0 })).toBe('#00ffff');
		expect(inkToHex({ c: 0, m: 100, y: 90, k: 10 })).toBe('#e60017');
	});

	it('get an id from their name that is never one the book has', () => {
		expect(newSwatchId(book, 'Sea Green!')).toBe('sea-green');
		expect(newSwatchId(book, 'Brand')).toBe('brand-2');
		expect(newSwatchId(book, '???')).toBe('swatch');
	});
});
