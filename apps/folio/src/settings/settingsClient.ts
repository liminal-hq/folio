// Reads and changes the app's settings through the settings plugin's guest-js API
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import {
	getSettings,
	onSettingsChanged,
	resetSettings,
	updateSettings,
	type SettingsPatch,
	type SettingsSnapshot,
} from '@liminal-hq/folio-plugin-settings';

/** What the page needs from the settings: Rust owns them, so every call goes through it. */
export interface SettingsClient {
	get(): Promise<SettingsSnapshot>;
	update(patch: SettingsPatch): Promise<SettingsSnapshot>;
	reset(): Promise<SettingsSnapshot>;
	/** Hears every change, from any window. Resolves to the function that stops listening. */
	subscribe(listener: (snapshot: SettingsSnapshot) => void): Promise<() => void>;
}

export const tauriSettingsClient: SettingsClient = {
	get: getSettings,
	update: updateSettings,
	reset: resetSettings,
	subscribe: onSettingsChanged,
};
