// Tests for where a guide lands, when it is dropped back on a ruler, and the menu's intents
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { Spread } from '@liminal-hq/folio-protocol/generated/Spread';
import { describe, expect, it } from 'vitest';
import { guideIntents, overRuler, placeGuide } from './guides';

const targets = { x: [0, 36, 72], y: [0, 100] };

describe('placeGuide', () => {
	it('snaps to a target within the threshold, and says which', () => {
		expect(placeGuide(38, 'x', targets, 4.5, 4, false)).toEqual({ pos: 36, to: 36 });
		expect(placeGuide(98, 'y', targets, 4.5, 4, false)).toEqual({ pos: 100, to: 100 });
	});

	it("falls on the unit's fine step when nothing is near", () => {
		const placed = placeGuide(50, 'x', targets, 4.5, 4, false);
		expect(placed.to).toBeNull();
		expect(placed.pos).toBeCloseTo(4.5 * 11); // 49.5, the 1/16 in step nearest 50
	});

	it('goes exactly where the pointer is when placed freely', () => {
		expect(placeGuide(37.3, 'x', targets, 4.5, 4, true)).toEqual({ pos: 37.3, to: null });
	});
});

describe('overRuler', () => {
	const stage = { left: 100, top: 80 };
	it('is over the left ruler for a vertical guide left of the stage', () => {
		expect(overRuler('x', { x: 90, y: 500 }, stage)).toBe(true);
		expect(overRuler('x', { x: 110, y: 20 }, stage)).toBe(false);
	});
	it('is over the top ruler for a horizontal guide above the stage', () => {
		expect(overRuler('y', { x: 500, y: 70 }, stage)).toBe(true);
		expect(overRuler('y', { x: 20, y: 90 }, stage)).toBe(false);
	});
});

describe('guideIntents', () => {
	const spread = { id: 'sp' } as unknown as Spread;
	it('locks, unlocks and deletes the guide named', () => {
		expect(guideIntents('lock', spread, 2)).toEqual([
			{ type: 'LockGuide', on: { spread: 'sp' }, index: 2, locked: true },
		]);
		expect(guideIntents('unlock', spread, 2)).toEqual([
			{ type: 'LockGuide', on: { spread: 'sp' }, index: 2, locked: false },
		]);
		expect(guideIntents('delete', spread, 2)).toEqual([
			{ type: 'RemoveGuide', on: { spread: 'sp' }, index: 2 },
		]);
	});
	it('puts a guide on every page, and a master guide back on the spread it was on', () => {
		expect(guideIntents('promote', spread, 1)).toEqual([
			{ type: 'PromoteGuide', spread: 'sp', index: 1 },
		]);
		const master = { id: 'master:mst_a' } as unknown as Spread;
		expect(guideIntents('demote', master, 0, 'sp_1')).toEqual([
			{ type: 'DemoteGuide', master: 'mst_a', index: 0, spread: 'sp_1' },
		]);
		expect(guideIntents('demote', master, 0, null)).toEqual([]);
		expect(guideIntents('promote', master, 0)).toEqual([]);
	});
	it("clears the spread's guides, or every spread's", () => {
		expect(guideIntents('clearSpread', spread, 0)).toEqual([{ type: 'ClearGuides', spread: 'sp' }]);
		expect(guideIntents('clearAll', spread, 0)).toEqual([{ type: 'ClearGuides' }]);
	});
});
