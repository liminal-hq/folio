// Tests for the page's copy of the settings
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { describe, expect, it } from 'vitest';
import type { SettingsClient } from './settingsClient';
import { createFakeSettingsClient, DEFAULT_SETTINGS } from './fakeSettingsClient';
import { createSettingsStore } from './settingsStore';

describe('createSettingsStore', () => {
	it('reads the settings on start', async () => {
		const store = createSettingsStore(createFakeSettingsClient());
		expect(store.state.getState().snapshot).toBeNull();
		await store.start();
		expect(store.state.getState().snapshot?.settings).toEqual(DEFAULT_SETTINGS);
	});

	it('follows changes made through the client, from this window or another', async () => {
		const client = createFakeSettingsClient();
		const mine = createSettingsStore(client);
		const other = createSettingsStore(client);
		await mine.start();
		await other.start();
		await other.update({ mode: 'expert', interface: { menuBar: true } });
		const seen = mine.state.getState().snapshot;
		expect(seen?.revision).toBe(1);
		expect(seen?.settings.mode).toBe('expert');
		expect(seen?.settings.interface.menuBar).toBe(true);
		// The patch named only some fields: the others kept their values.
		expect(seen?.settings.interface.sidebarWidth).toBe(292);
	});

	it('ignores a snapshot older than the one it holds', async () => {
		const client = createFakeSettingsClient();
		const store = createSettingsStore(client);
		await store.start();
		await store.update({ units: 'millimetres' });
		await store.update({ mode: 'expert' });
		let announce: (snapshot: {
			revision: number;
			settings: typeof DEFAULT_SETTINGS;
		}) => void = () => {};
		const lagging: SettingsClient = {
			...client,
			subscribe: async (listener) => {
				announce = listener;
				return () => {};
			},
		};
		const late = createSettingsStore(lagging);
		await late.start();
		announce({ revision: 1, settings: { ...DEFAULT_SETTINGS, units: 'millimetres' } });
		expect(late.state.getState().snapshot?.revision).toBe(2);
	});

	it('keeps a refusal until the next change succeeds', async () => {
		const client = createFakeSettingsClient();
		const refusing: SettingsClient = {
			...client,
			update: async () => {
				throw { kind: 'invalid', message: 'too wide', field: 'interface.sidebarWidth' };
			},
		};
		const store = createSettingsStore(refusing);
		await store.start();
		await store.update({ interface: { sidebarWidth: 9000 } });
		expect(store.state.getState().error?.field).toBe('interface.sidebarWidth');
		expect(store.state.getState().snapshot?.revision).toBe(0);
	});

	it('resets to the defaults', async () => {
		const store = createSettingsStore(createFakeSettingsClient());
		await store.start();
		await store.update({ mode: 'expert' });
		await store.reset();
		expect(store.state.getState().snapshot?.settings).toEqual(DEFAULT_SETTINGS);
	});
});
