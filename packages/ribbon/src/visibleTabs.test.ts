// Tests for which parts of the ribbon model show in each mode
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { describe, expect, it } from 'vitest';
import type { RibbonModel } from './types';
import { visibleTabs } from './visibleTabs';

const item = (id: string, expert = false) => ({ id, label: id, d: 'M0 0', expert });

const model: RibbonModel = {
	tabs: [
		{
			id: 'home',
			label: 'Home',
			groups: [
				{ id: 'a', label: 'A', items: [item('one'), item('two', true)] },
				{ id: 'b', label: 'B', expert: true, items: [item('three')] },
				{ id: 'c', label: 'C', rows: [[[item('four'), item('five', true)], [item('six', true)]]] },
			],
		},
		{
			id: 'picture',
			label: 'Picture',
			contextual: true,
			groups: [{ id: 'p', label: 'P', items: [item('p1')] }],
		},
		{
			id: 'only-expert',
			label: 'Hidden',
			groups: [{ id: 'x', label: 'X', expert: true, items: [item('x1')] }],
		},
	],
};

describe('visibleTabs', () => {
	it('hides expert items and groups in Easy mode', () => {
		const [home] = visibleTabs(model, 'easy');
		expect(home?.groups.map((g) => g.id)).toEqual(['a', 'c']);
		expect(home?.groups[0]?.items?.map((i) => i.id)).toEqual(['one']);
		expect(home?.groups[1]?.rows).toEqual([[[item('four')]]]);
	});

	it('shows everything in Expert mode', () => {
		const tabs = visibleTabs(model, 'expert');
		expect(tabs.map((t) => t.id)).toEqual(['home', 'only-expert']);
		expect(tabs[0]?.groups.map((g) => g.id)).toEqual(['a', 'b', 'c']);
	});

	it('drops a tab left with no groups', () => {
		expect(visibleTabs(model, 'easy').map((t) => t.id)).toEqual(['home']);
	});

	it('shows a contextual tab only while it is listed', () => {
		expect(visibleTabs(model, 'easy', ['picture']).map((t) => t.id)).toEqual(['home', 'picture']);
	});
});
