// Tests for how the page marks sit in settings
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { describe, expect, it } from 'vitest';
import { DEFAULT_SETTINGS } from '../settings/fakeSettingsClient';
import { coloursOf, MARK_PALETTES, marksOf, marksPatch } from './marksModel';

describe('marksOf and coloursOf', () => {
	it('read which marks are on and the colour each one takes', () => {
		const settings = {
			...DEFAULT_SETTINGS.marks,
			columns: { on: true, colour: 2 },
		};
		expect(marksOf(settings)).toEqual({
			bleed: true,
			margins: true,
			columns: true,
			baseline: false,
			frameEdges: false,
		});
		expect(coloursOf(settings).columns).toBe(MARK_PALETTES.columns[2]);
		expect(coloursOf(settings).bleed).toBe(MARK_PALETTES.bleed[0]);
	});

	it('fall back to the first colour for an index there is no colour for', () => {
		const settings = { ...DEFAULT_SETTINGS.marks, bleed: { on: true, colour: 9 } };
		expect(coloursOf(settings).bleed).toBe(MARK_PALETTES.bleed[0]);
	});
});

describe('marksPatch', () => {
	const now = marksOf(DEFAULT_SETTINGS.marks);
	it('names only the marks that change', () => {
		expect(marksPatch(now, { margins: false, bleed: true, columns: true })).toEqual({
			marks: { margins: { on: false }, columns: { on: true } },
		});
	});
	it('is empty when nothing changes', () => {
		expect(marksPatch(now, { bleed: true })).toEqual({ marks: {} });
	});
});
