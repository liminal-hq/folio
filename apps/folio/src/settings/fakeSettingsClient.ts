// An in-memory settings client for tests and for the page outside Tauri
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { Settings, SettingsPatch, SettingsSnapshot } from '@liminal-hq/folio-plugin-settings';
import type { SettingsClient } from './settingsClient';

/** The defaults Rust applies to a fresh install (`folio_settings::Settings::default`). */
export const DEFAULT_SETTINGS: Settings = {
	mode: 'easy',
	colourMode: 'system',
	units: 'inches',
	jax: 'tips',
	interface: {
		menuBar: false,
		toolbar: 'ribbon',
		sidebarLayout: 'rightGrouped',
		sidebarWidth: 292,
		sidebarCollapsed: false,
		largerControls: false,
	},
	marks: {
		bleed: { on: true, colour: 0 },
		margins: { on: true, colour: 0 },
		columns: { on: false, colour: 0 },
		baseline: { on: false, colour: 0 },
		frameEdges: { on: false, colour: 0 },
		guides: { on: true, colour: 0 },
		snap: true,
	},
	rulers: { on: false, units: 'inches' },
	spelling: { enabled: true },
	recentBooks: [],
};

/** Lays a patch over settings the way Rust does: nested objects merge, anything else replaces. */
function merge<T extends object>(base: T, patch: object): T {
	const result: Record<string, unknown> = { ...(base as Record<string, unknown>) };
	for (const [key, change] of Object.entries(patch)) {
		const current = result[key];
		result[key] =
			change !== null &&
			typeof change === 'object' &&
			!Array.isArray(change) &&
			current !== null &&
			typeof current === 'object'
				? merge(current as object, change)
				: change;
	}
	return result as T;
}

/** A client that behaves like the plugin, in memory, announcing each change to its subscribers. */
export function createFakeSettingsClient(initial: Settings = DEFAULT_SETTINGS): SettingsClient {
	let snapshot: SettingsSnapshot = { revision: 0, settings: initial };
	const listeners = new Set<(snapshot: SettingsSnapshot) => void>();

	const commit = (settings: Settings): SettingsSnapshot => {
		if (JSON.stringify(settings) === JSON.stringify(snapshot.settings)) return snapshot;
		snapshot = { revision: snapshot.revision + 1, settings };
		for (const listener of listeners) listener(snapshot);
		return snapshot;
	};

	return {
		get: async () => snapshot,
		update: async (patch: SettingsPatch) => commit(merge(snapshot.settings, patch)),
		reset: async () => commit(DEFAULT_SETTINGS),
		subscribe: async (listener) => {
			listeners.add(listener);
			return () => listeners.delete(listener);
		},
	};
}
