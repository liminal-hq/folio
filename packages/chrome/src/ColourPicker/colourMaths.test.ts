// Tests for the picker's colour maths
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { describe, expect, it } from 'vitest';
import {
	contrastGrade,
	contrastRatio,
	hexToInk,
	hexToRgb,
	hsvToRgb,
	inkToHex,
	parseColour,
	rgbToHex,
	rgbToHsv,
} from './colourMaths';

describe('parseColour', () => {
	it('reads short and long hex, with or without a hash, in any case', () => {
		expect(parseColour('#abc')).toBe('#aabbcc');
		expect(parseColour('ABC')).toBe('#aabbcc');
		expect(parseColour('#AABBCC')).toBe('#aabbcc');
		expect(parseColour('  1c1b20 ')).toBe('#1c1b20');
		expect(parseColour('#000')).toBe('#000000');
	});

	it('reads rgb()', () => {
		expect(parseColour('rgb(28, 27, 32)')).toBe('#1c1b20');
		expect(parseColour('rgb(0 0 0)')).toBe('#000000');
	});

	it('refuses what is not a colour', () => {
		for (const bad of [
			'',
			'#',
			'#ab',
			'#abcd',
			'#abcdef0',
			'ggg',
			'rgb(256, 0, 0)',
			'red',
			'#12 34',
		]) {
			expect(parseColour(bad)).toBeNull();
		}
	});
});

describe('rgb and hsv', () => {
	it('formats and reads channels', () => {
		expect(rgbToHex([0, 0, 0])).toBe('#000000');
		expect(rgbToHex([255, 255, 255])).toBe('#ffffff');
		expect(rgbToHex([300, -4, 15.6])).toBe('#ff0010');
		expect(hexToRgb('#1c1b20')).toEqual([28, 27, 32]);
		expect(hexToRgb('nonsense')).toEqual([0, 0, 0]);
	});

	it('places the primaries and greys', () => {
		expect(rgbToHsv([255, 0, 0])).toEqual({ h: 0, s: 1, v: 1 });
		expect(rgbToHsv([0, 255, 0]).h).toBe(120);
		expect(rgbToHsv([0, 0, 255]).h).toBe(240);
		expect(rgbToHsv([0, 0, 0])).toEqual({ h: 0, s: 0, v: 0 });
		expect(rgbToHsv([255, 255, 255])).toEqual({ h: 0, s: 0, v: 1 });
	});

	it('round-trips every colour on a coarse lattice', () => {
		for (let r = 0; r <= 255; r += 51) {
			for (let g = 0; g <= 255; g += 51) {
				for (let b = 0; b <= 255; b += 51) {
					expect(rgbToHex(hsvToRgb(rgbToHsv([r, g, b])))).toBe(rgbToHex([r, g, b]));
				}
			}
		}
	});
});

describe('contrast', () => {
	it('is 21 for black on white and 1 for a colour on itself', () => {
		expect(contrastRatio('#000000', '#ffffff')).toBeCloseTo(21, 5);
		expect(contrastRatio('#ffffff', '#000000')).toBeCloseTo(21, 5);
		expect(contrastRatio('#808080', '#808080')).toBe(1);
	});

	it('matches a known mid value', () => {
		// #767676 on white is the classic first grey to pass AA.
		expect(contrastRatio('#767676', '#ffffff')).toBeCloseTo(4.54, 2);
	});

	it('grades the ratio', () => {
		expect(contrastGrade(21)).toBe('aaa');
		expect(contrastGrade(7)).toBe('aaa');
		expect(contrastGrade(4.54)).toBe('aa');
		expect(contrastGrade(3.2)).toBe('large');
		expect(contrastGrade(1.5)).toBe('fail');
	});
});

describe('ink', () => {
	it('shows ink as the plain inverse', () => {
		expect(inkToHex([0, 0, 0, 0])).toBe('#ffffff');
		expect(inkToHex([0, 0, 0, 100])).toBe('#000000');
		expect(inkToHex([100, 0, 0, 0])).toBe('#00ffff');
	});

	it('works ink back from a screen colour', () => {
		expect(hexToInk('#000000')).toEqual([0, 0, 0, 100]);
		expect(hexToInk('#ffffff')).toEqual([0, 0, 0, 0]);
		expect(hexToInk('#00ffff')).toEqual([100, 0, 0, 0]);
	});
});
