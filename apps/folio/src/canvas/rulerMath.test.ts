// Tests for the ruler maths: units, ticks at a zoom, and written positions
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { describe, expect, it } from 'vitest';
import {
	formatPosition,
	MAJOR_MAX_PX,
	MAJOR_MIN_PX,
	plain,
	RULER_UNIT_ORDER,
	RULER_UNITS,
	rulerTicks,
	stepFor,
} from './rulerMath';

describe('stepFor', () => {
	it('keeps major ticks 44–160 px apart at any zoom, in any unit', () => {
		for (const unit of RULER_UNIT_ORDER) {
			for (let scale = 0.1; scale < 12; scale *= 1.07) {
				const { major } = stepFor(unit, scale);
				const px = major * RULER_UNITS[unit].points * scale;
				expect(px).toBeGreaterThanOrEqual(MAJOR_MIN_PX - 1e-6);
				expect(px).toBeLessThanOrEqual(MAJOR_MAX_PX * 1.25);
			}
		}
	});

	it('takes a coarser step when zoomed out and a finer one when zoomed in', () => {
		expect(stepFor('inches', 0.5).major).toBeGreaterThan(stepFor('inches', 4).major);
	});
});

describe('rulerTicks', () => {
	it('labels a major tick every half inch at 100%, with tenths between', () => {
		// 4/3 px a point is 100%: an inch is 96 px, so a major tick is the first step 44 px or wider.
		const ticks = rulerTicks(0, 216, 96 / 72, 'inches');
		const majors = ticks.filter((t) => t.major);
		expect(majors.map((t) => t.label)).toEqual(['0', '0.5', '1', '1.5', '2', '2.5', '3']);
		expect(majors.map((t) => t.pos)).toEqual([0, 36, 72, 108, 144, 180, 216]);
		expect(ticks.filter((t) => !t.major)).toHaveLength(24);
	});

	it('runs into the bleed with negative values', () => {
		const ticks = rulerTicks(-30, 20, 96 / 72, 'points');
		expect(ticks[0]!.pos).toBe(-30);
		expect(ticks.some((t) => t.major && t.label === '-50')).toBe(false);
		expect(rulerTicks(-150, 0, 96 / 72, 'points').some((t) => t.label === '-100')).toBe(true);
	});

	it('gives nothing for an empty range or a zero zoom', () => {
		expect(rulerTicks(10, 10, 1, 'points')).toEqual([]);
		expect(rulerTicks(0, 100, 0, 'points')).toEqual([]);
	});
});

describe('written positions', () => {
	it('drop the zeros a position does not need', () => {
		expect(plain(0.5, 3)).toBe('0.5');
		expect(plain(12, 3)).toBe('12');
		expect(plain(-0, 2)).toBe('0');
		expect(plain(0.25, 3)).toBe('0.25');
	});

	it('read in the unit with its short name', () => {
		expect(formatPosition(90, 'inches')).toBe('1.25 in');
		expect(formatPosition(72 / 25.4, 'millimetres')).toBe('1 mm');
		expect(formatPosition(18, 'picas')).toBe('1.5 pc');
		expect(formatPosition(0.75, 'pixels')).toBe('1 px');
	});
});
