// Tests for converting lengths between points and the units the person reads
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { describe, expect, it } from 'vitest';
import { fromPoints, toPoints } from './units';

describe('fromPoints', () => {
	it('shows inches to three places and millimetres to two, without trailing zeros', () => {
		expect(fromPoints(9, 'inches')).toBe(0.125);
		expect(fromPoints(72, 'inches')).toBe(1);
		expect(fromPoints(612, 'millimetres')).toBe(215.9);
	});
});

describe('toPoints', () => {
	it('reads a typed length in the units given', () => {
		expect(toPoints('1', 'inches')).toBe(72);
		expect(toPoints('25.4', 'millimetres')).toBeCloseTo(72);
		expect(toPoints(' 0,5 ', 'inches')).toBe(36);
	});
	it('refuses what is not a number', () => {
		expect(toPoints('', 'inches')).toBeNull();
		expect(toPoints('wide', 'inches')).toBeNull();
		expect(toPoints('Infinity', 'inches')).toBeNull();
	});
});
