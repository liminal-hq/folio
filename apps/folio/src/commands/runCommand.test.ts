// Tests for running menu commands
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { SettingsPatch } from '@liminal-hq/folio-plugin-settings';
import { describe, expect, it, vi } from 'vitest';
import { DEFAULT_SETTINGS } from '../settings/fakeSettingsClient';
import { runCommand, type CommandContext } from './runCommand';

function context(overrides: Partial<CommandContext['settings']> = {}) {
	const patches: SettingsPatch[] = [];
	const closeWindow = vi.fn();
	const openWindow = vi.fn();
	const value: CommandContext = {
		settings: { ...DEFAULT_SETTINGS, ...overrides },
		update: async (patch) => {
			patches.push(patch);
		},
		openWindow,
		closeWindow,
	};
	return { value, patches, closeWindow, openWindow };
}

describe('runCommand', () => {
	it('toggles the menu bar', async () => {
		const { value, patches } = context();
		await runCommand('view.menuBar', value);
		expect(patches).toEqual([{ interface: { menuBar: true } }]);
	});

	it('swaps the ribbon and the classic toolbar', async () => {
		const { value, patches } = context();
		await runCommand('view.ribbon', value);
		expect(patches).toEqual([{ interface: { toolbar: 'classic' } }]);
	});

	it('switches between Easy and Expert', async () => {
		const easy = context();
		await runCommand('view.expert', easy.value);
		expect(easy.patches).toEqual([{ mode: 'expert' }]);
		const expert = context({ mode: 'expert' });
		await runCommand('view.expert', expert.value);
		expect(expert.patches).toEqual([{ mode: 'easy' }]);
	});

	it('turns Jax off, and back on to tips', async () => {
		const on = context();
		await runCommand('view.jax', on.value);
		expect(on.patches).toEqual([{ jax: 'off' }]);
		const off = context({ jax: 'off' });
		await runCommand('view.jax', off.value);
		expect(off.patches).toEqual([{ jax: 'tips' }]);
	});

	it('opens Preferences, About and Book tools', async () => {
		const { value, openWindow } = context();
		await runCommand('app.preferences', value);
		await runCommand('app.about', value);
		await runCommand('app.bookTools', value);
		expect(openWindow.mock.calls).toEqual([['Preferences'], ['About'], ['BookTools']]);
	});

	it('quits by closing the window', async () => {
		const { value, closeWindow } = context();
		await runCommand('app.quit', value);
		expect(closeWindow).toHaveBeenCalledOnce();
	});

	it('does nothing for a command that has no behaviour yet, or an unknown id', async () => {
		const { value, patches, closeWindow } = context();
		await runCommand('file.save', value);
		await runCommand('nonsense', value);
		expect(patches).toEqual([]);
		expect(closeWindow).not.toHaveBeenCalled();
	});
});
