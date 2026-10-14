// Exposes typed guest-side wrappers for the folio-settings plugin
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type { ColourMode } from '@liminal-hq/folio-protocol/generated/ColourMode';
import type { InterfaceSettings } from '@liminal-hq/folio-protocol/generated/InterfaceSettings';
import type { JaxHelp } from '@liminal-hq/folio-protocol/generated/JaxHelp';
import type { Mode } from '@liminal-hq/folio-protocol/generated/Mode';
import type { PluginStatus } from '@liminal-hq/folio-protocol/generated/PluginStatus';
import type { Settings } from '@liminal-hq/folio-protocol/generated/Settings';
import type { SettingsSnapshot } from '@liminal-hq/folio-protocol/generated/SettingsSnapshot';
import type { SidebarLayout } from '@liminal-hq/folio-protocol/generated/SidebarLayout';
import type { ToolbarLayout } from '@liminal-hq/folio-protocol/generated/ToolbarLayout';
import type { Units } from '@liminal-hq/folio-protocol/generated/Units';

const PREFIX = 'plugin:folio-settings|';

/** Sent to every window after every change; the payload is a `SettingsSnapshot`. */
export const SETTINGS_EVENT = 'folio-settings://changed';

/**
 * What a rejected command carries. `invalid` is a value outside its range or not shaped like the
 * settings (`field`, `min` and `max` say which, for a range); `storage` is settings that could not
 * be saved. Either way nothing changed.
 */
export interface SettingsCommandError {
	kind: 'invalid' | 'storage';
	message: string;
	field?: string;
	min?: number;
	max?: number;
}

/** A change to the settings: only the fields that change, nested like `Settings`. */
export type SettingsPatch = {
	[K in keyof Settings]?: Settings[K] extends object ? Partial<Settings[K]> : Settings[K];
};

function cmd<T>(name: string, args?: Record<string, unknown>): Promise<T> {
	return invoke<T>(`${PREFIX}${name}`, args);
}

/** Reports whether the settings plugin works. */
export function getStatus(): Promise<PluginStatus> {
	return cmd<PluginStatus>('get_status');
}

/** The settings in force and their revision. */
export function getSettings(): Promise<SettingsSnapshot> {
	return cmd<SettingsSnapshot>('get_settings');
}

/**
 * Changes the settings named in `patch` on top of what is in force now (Rust merges them), so a
 * window with an older copy of the document cannot turn another window's change back. Rejects with
 * a `SettingsCommandError` and changes nothing for a bad value or a save that fails.
 */
export function updateSettings(patch: SettingsPatch): Promise<SettingsSnapshot> {
	return cmd<SettingsSnapshot>('update_settings', { patch });
}

/** Puts every setting back to its default. */
export function resetSettings(): Promise<SettingsSnapshot> {
	return cmd<SettingsSnapshot>('reset_settings');
}

/** Hears every change. Read `getSettings` first and apply snapshots with a higher revision. */
export function onSettingsChanged(
	listener: (snapshot: SettingsSnapshot) => void,
): Promise<UnlistenFn> {
	return listen<SettingsSnapshot>(SETTINGS_EVENT, (e) => listener(e.payload));
}

export type {
	ColourMode,
	InterfaceSettings,
	JaxHelp,
	Mode,
	PluginStatus,
	Settings,
	SettingsSnapshot,
	SidebarLayout,
	ToolbarLayout,
	Units,
};
