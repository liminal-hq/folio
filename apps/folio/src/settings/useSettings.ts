// React access to the window's settings store
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { createContext, useContext } from 'react';
import { useStore } from 'zustand';
import type {
	Settings,
	SettingsCommandError,
	SettingsPatch,
} from '@liminal-hq/folio-plugin-settings';
import { DEFAULT_SETTINGS } from './fakeSettingsClient';
import type { SettingsStore } from './settingsStore';
import { settingsOf } from './settingsStore';

export const SettingsContext = createContext<SettingsStore | null>(null);

function useSettingsStore(): SettingsStore {
	const store = useContext(SettingsContext);
	if (store === null) throw new Error('useSettings needs a SettingsProvider above it');
	return store;
}

/** The settings in force; the defaults until Rust has answered. */
export function useSettings(): Settings {
	const store = useSettingsStore();
	return useStore(store.state, (state) => settingsOf(state, DEFAULT_SETTINGS));
}

/** The last refusal from a change, or `null`. */
export function useSettingsError(): SettingsCommandError | null {
	const store = useSettingsStore();
	return useStore(store.state, (state) => state.error);
}

/** Asks Rust to change settings. */
export function useUpdateSettings(): (patch: SettingsPatch) => Promise<void> {
	return useSettingsStore().update;
}
