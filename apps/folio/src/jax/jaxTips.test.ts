// Tests for which tip Jax gives in which context, and the shape of the guides
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { describe, expect, it } from 'vitest';
import {
	FIXED_GUIDE,
	FLOWING_GUIDE,
	GENERAL_TIPS,
	contextualTip,
	type TipContext,
} from './jaxTips';

const base: TipContext = {
	picture: false,
	text: false,
	reader: 'print',
	flowing: false,
	grid: false,
};

describe('contextualTip', () => {
	it('has nothing to say when nothing is going on', () => {
		expect(contextualTip(base)).toBeNull();
	});

	it('speaks about a selected picture before anything else', () => {
		expect(contextualTip({ ...base, picture: true, text: true, grid: true })).toBe(
			'jax.tip.picture',
		);
	});

	it('speaks about selected text', () => {
		expect(contextualTip({ ...base, text: true })).toBe('jax.tip.text');
	});

	it('explains a reader preview differently for flowing and fixed books', () => {
		expect(contextualTip({ ...base, reader: 'kindle', flowing: true })).toBe('jax.tip.reflow');
		expect(contextualTip({ ...base, reader: 'kobo' })).toBe('jax.tip.fixedPreview');
	});

	it('mentions the grid when it is drawn', () => {
		expect(contextualTip({ ...base, grid: true })).toBe('jax.tip.grid');
	});
});

describe('the tips and guides', () => {
	it('keeps general tips distinct', () => {
		expect(new Set(GENERAL_TIPS).size).toBe(GENERAL_TIPS.length);
	});

	it('has steps in both guides', () => {
		expect(FIXED_GUIDE.length).toBeGreaterThan(0);
		expect(FLOWING_GUIDE.length).toBeGreaterThan(0);
	});
});
