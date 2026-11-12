// Tests for what the colour pickers offer
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { describe, expect, it } from 'vitest';
import {
	colourName,
	colourPickerLabels,
	pinnedColours,
	swatchOptions,
	themeColours,
} from './colourPicker';

describe('the colours Folio offers', () => {
	it('always has pure black, then white, pinned first', () => {
		expect(pinnedColours()).toEqual([
			{ value: '#000000', name: 'Black' },
			{ value: '#ffffff', name: 'White' },
		]);
	});

	it('lists six greys and then the eight Folio colours, each with a name and no repeats', () => {
		const theme = themeColours();
		expect(theme).toHaveLength(14);
		expect(theme.slice(6).map((c) => c.name)).toEqual([
			'Ink',
			'Plum',
			'Deep teal',
			'Forest',
			'Brick',
			'Marigold',
			'Navy',
			'Slate',
		]);
		expect(new Set(theme.map((c) => c.value)).size).toBe(theme.length);
		expect(theme.every((c) => /^#[0-9a-f]{6}$/.test(c.value) && c.name)).toBe(true);
	});

	it('names the colours it offers, and no others', () => {
		expect(colourName('#000000')).toBe('Black');
		expect(colourName('#5b2a4a')).toBe('Plum');
		expect(colourName('#123456')).toBeNull();
	});

	it('lists a book swatch by its reference, shown colour and ink', () => {
		expect(
			swatchOptions([
				{ id: 'a', name: 'A', display: '#112233', ink: { c: 1, m: 2, y: 3, k: 4 } },
				{ id: 'b', name: 'B' },
			]),
		).toEqual([
			{ value: 'swatch:a', name: 'A', shown: '#112233', ink: [1, 2, 3, 4] },
			{ value: 'swatch:b', name: 'B' },
		]);
	});

	it('speaks in the app language', () => {
		const labels = colourPickerLabels();
		expect(labels.swatchName('Plum', '#5b2a4a')).toBe('Plum, #5b2a4a');
		expect(labels.saturationBrightnessValue(10, 20)).toBe(
			'Saturation 10 percent, brightness 20 percent',
		);
	});
});
