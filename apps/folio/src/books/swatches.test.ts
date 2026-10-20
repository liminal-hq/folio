// Tests for resolving swatch references for drawing
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { describe, expect, it } from 'vitest';
import { resolveSwatches } from './swatches';

const displays = new Map([['brand', '#c8102e']]);

describe('resolveSwatches', () => {
	it('replaces references anywhere in a value and leaves the rest', () => {
		const spread = {
			frames: [{ type: 'shape', fill: 'swatch:brand', stroke: '#000000', n: 3 }],
			bgSpan: { type: 'colour', colour: 'swatch:brand' },
		};
		expect(resolveSwatches(spread, displays)).toEqual({
			frames: [{ type: 'shape', fill: '#c8102e', stroke: '#000000', n: 3 }],
			bgSpan: { type: 'colour', colour: '#c8102e' },
		});
	});

	it('draws a reference to a swatch that is not there as grey', () => {
		expect(resolveSwatches('swatch:gone', displays)).toBe('#808080');
	});

	it('returns the same object when there is nothing to resolve', () => {
		const value = { a: [{ b: '#123456' }], c: null };
		expect(resolveSwatches(value, displays)).toBe(value);
	});
});
