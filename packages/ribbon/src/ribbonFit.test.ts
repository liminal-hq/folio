// Tests for how the ribbon's groups give way in a narrow panel
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { describe, expect, it } from 'vitest';
import { WHOLE, narrower, type RibbonFit } from './ribbonFit';

const groups = [
	{ id: 'clipboard', compactable: true },
	{ id: 'font', compactable: false },
	{ id: 'add', compactable: true },
];

/** Every fit `narrower` steps through, from the whole groups to the last one there is. */
function steps(): RibbonFit[] {
	const out: RibbonFit[] = [];
	let fit: RibbonFit | null = WHOLE;
	while ((fit = narrower(fit, groups))) out.push(fit);
	return out;
}

describe('narrower', () => {
	it('drops the words from the right first, then folds from the right, then puts groups under More', () => {
		expect(steps()).toEqual([
			{ levels: { add: 'compact' }, more: 0 },
			{ levels: { add: 'compact', clipboard: 'compact' }, more: 0 },
			{ levels: { add: 'folded', clipboard: 'compact' }, more: 0 },
			{ levels: { add: 'folded', clipboard: 'compact', font: 'folded' }, more: 0 },
			{ levels: { add: 'folded', clipboard: 'folded', font: 'folded' }, more: 0 },
			{ levels: { add: 'folded', clipboard: 'folded', font: 'folded' }, more: 1 },
			{ levels: { add: 'folded', clipboard: 'folded', font: 'folded' }, more: 2 },
			{ levels: { add: 'folded', clipboard: 'folded', font: 'folded' }, more: 3 },
		]);
	});
});
