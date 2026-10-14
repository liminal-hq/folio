// The page's copy of the settings: read once, then kept current by the plugin's change events
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type {
	Settings,
	SettingsCommandError,
	SettingsPatch,
	SettingsSnapshot,
} from '@liminal-hq/folio-plugin-settings';
import { createStore, type StoreApi } from 'zustand/vanilla';
import type { SettingsClient } from './settingsClient';

export interface SettingsState {
	/** The settings in force, or `null` until they have been read (and for good outside Tauri). */
	snapshot: SettingsSnapshot | null;
	/** The last refusal, until the next change succeeds. */
	error: SettingsCommandError | null;
}

export interface SettingsStore {
	state: StoreApi<SettingsState>;
	/** Reads the settings and follows changes. Resolves once they are known; returns a stop function. */
	start(): Promise<() => void>;
	/** Asks Rust to change settings; the answer arrives as the new snapshot. */
	update(patch: SettingsPatch): Promise<void>;
	reset(): Promise<void>;
}

function asCommandError(reason: unknown): SettingsCommandError {
	if (typeof reason === 'object' && reason !== null && 'kind' in reason && 'message' in reason) {
		return reason as SettingsCommandError;
	}
	return { kind: 'storage', message: String(reason) };
}

/**
 * The settings for one window. A change event and a read can arrive in either order, so a snapshot
 * is applied only when its revision is higher than the one held; the store subscribes before it
 * reads, so a change made in between reaches it as an event instead of falling through the gap.
 */
export function createSettingsStore(client: SettingsClient): SettingsStore {
	const state = createStore<SettingsState>(() => ({ snapshot: null, error: null }));

	const apply = (snapshot: SettingsSnapshot) => {
		const held = state.getState().snapshot;
		if (held === null || snapshot.revision > held.revision) {
			state.setState({ snapshot, error: null });
		}
	};

	const run = async (change: () => Promise<SettingsSnapshot>) => {
		try {
			apply(await change());
		} catch (reason) {
			state.setState({ error: asCommandError(reason) });
		}
	};

	return {
		state,
		async start() {
			const stop = await client.subscribe(apply);
			try {
				apply(await client.get());
			} catch {
				// Not in Tauri: the page keeps working on the defaults its components fall back to.
			}
			return stop;
		},
		update: (patch) => run(() => client.update(patch)),
		reset: () => run(() => client.reset()),
	};
}

/** The settings in force, or `fallback` while they are not known. */
export function settingsOf(state: SettingsState, fallback: Settings): Settings {
	return state.snapshot?.settings ?? fallback;
}
