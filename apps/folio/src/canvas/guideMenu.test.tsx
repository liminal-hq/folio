// Tests for a ruler guide's menu: putting it on every page, or back on one spread
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { describe, expect, it } from 'vitest';
import { guideMenuItems, isGuideCommand } from './guideMenu';

const guide = { axis: 'y', pos: 10, locked: false } as never;
const ids = (items: ReturnType<typeof guideMenuItems>) =>
	items.map((i) => (i as { id?: string }).id);

describe('guideMenuItems', () => {
	it('offers every page for a spread guide, only when it can go there', () => {
		expect(ids(guideMenuItems(guide, { canPromote: true }))).toContain('promote');
		const refused = guideMenuItems(guide, { canPromote: false }).find(
			(i) => (i as { id?: string }).id === 'promote',
		);
		expect(refused).toMatchObject({ disabled: true });
	});

	it('offers only this spread for a master guide, and neither when it is not asked about', () => {
		expect(ids(guideMenuItems(guide, { canDemote: true }))).toContain('demote');
		expect(ids(guideMenuItems(guide, { canDemote: true }))).not.toContain('promote');
		expect(ids(guideMenuItems(guide))).not.toContain('promote');
		expect(ids(guideMenuItems(guide))).not.toContain('demote');
	});

	it('knows the two new commands', () => {
		expect(isGuideCommand('promote')).toBe(true);
		expect(isGuideCommand('demote')).toBe(true);
	});
});
