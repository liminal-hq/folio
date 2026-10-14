// Tests for the app menu and menu bar models
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { expectEveryItemHasIcon } from '@liminal-hq/folio-chrome/ContextMenu/expectEveryItemHasIcon';
import type { MenuItem } from '@liminal-hq/folio-chrome/ContextMenu/types';
import { describe, expect, it } from 'vitest';
import { DEFAULT_SETTINGS } from '../settings/fakeSettingsClient';
import { buildAppMenu, buildMenuBar } from './appMenuModel';
import { ENABLED_COMMANDS } from './runCommand';

const available = { enabled: ENABLED_COMMANDS };

function flatten(items: readonly MenuItem[]): MenuItem[] {
	return items.flatMap((item) =>
		item.type === 'submenu' ? [item, ...flatten(item.items)] : [item],
	);
}

function find(items: readonly MenuItem[], id: string): MenuItem | undefined {
	return flatten(items).find((item) => 'id' in item && item.id === id);
}

describe('the app menu', () => {
	it('gives every row an icon', () => {
		expectEveryItemHasIcon(buildAppMenu(DEFAULT_SETTINGS, available));
		expectEveryItemHasIcon(buildMenuBar(DEFAULT_SETTINGS, available));
	});

	it('reflects the settings in its toggles', () => {
		const items = buildAppMenu(
			{
				...DEFAULT_SETTINGS,
				mode: 'expert',
				jax: 'off',
				interface: { ...DEFAULT_SETTINGS.interface, menuBar: true, toolbar: 'classic' },
			},
			available,
		);
		const checked = (id: string) => {
			const item = find(items, id);
			return item?.type === 'checkbox' ? item.checked : undefined;
		};
		expect(checked('view.menuBar')).toBe(true);
		expect(checked('view.ribbon')).toBe(false);
		expect(checked('view.expert')).toBe(true);
		expect(checked('view.jax')).toBe(false);
	});

	it('lists commands that do nothing yet as unavailable, with the reason', () => {
		const items = buildAppMenu(DEFAULT_SETTINGS, available);
		const save = find(items, 'file.save');
		expect(save).toMatchObject({ disabled: true, title: 'Not available yet' });
		const expert = find(items, 'view.expert');
		expect(expert).toMatchObject({ disabled: false });
	});

	it('names recent books, or says there are none', () => {
		const none = find(buildAppMenu(DEFAULT_SETTINGS, available), 'file.openRecent');
		expect(none?.type === 'submenu' && none.items[0]).toMatchObject({ label: 'No recent books' });
		const some = find(
			buildAppMenu({ ...DEFAULT_SETTINGS, recentBooks: ['/books/otter.folio'] }, available),
			'file.openRecent',
		);
		expect(some?.type === 'submenu' && some.items[0]).toMatchObject({ label: 'otter.folio' });
	});

	it('gives the menu bar File, Edit, View and Help, matching the mnemonic ids', () => {
		const bar = buildMenuBar(DEFAULT_SETTINGS, available);
		expect(bar.map((menu) => ('id' in menu ? menu.id : ''))).toEqual([
			'menu:file',
			'menu:edit',
			'menu:view',
			'menu:help',
		]);
	});
});
