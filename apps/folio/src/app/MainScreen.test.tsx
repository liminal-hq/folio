// Tests for the main window's screen
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { WindowControls } from '@liminal-hq/folio-chrome/TitleBar/windowControls';
import { WindowChromeProvider } from '@liminal-hq/folio-chrome/WindowChromeProvider/WindowChromeProvider';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it } from 'vitest';
import type { AppInfoClient } from '../services/appInfoClient';
import { createFakeSettingsClient } from '../settings/fakeSettingsClient';
import { SettingsProvider } from '../settings/SettingsProvider';
import { MainScreen } from './MainScreen';

const controls: WindowControls = {
	minimize: () => {},
	toggleMaximize: () => {},
	close: () => {},
	setAlwaysOnTop: () => {},
	isMaximized: () => false,
	onMaximizedChange: () => () => {},
};

const info: AppInfoClient = {
	appInfo: async () => ({
		name: 'Folio',
		version: '1.2.3',
		identifier: 'ca.liminalhq.folio',
		platform: 'linux',
	}),
};

function renderScreen(settings = createFakeSettingsClient()) {
	render(
		<SettingsProvider client={settings}>
			<WindowChromeProvider controls={controls}>
				<MainScreen client={info} />
			</WindowChromeProvider>
		</SettingsProvider>,
	);
	return settings;
}

describe('MainScreen', () => {
	it('lays out the toolbar, stage, inspector, shelf and status bar', async () => {
		renderScreen();
		expect(screen.getByRole('toolbar', { name: 'Toolbar' })).toBeInTheDocument();
		expect(screen.getByRole('main')).toBeInTheDocument();
		expect(screen.getByRole('complementary', { name: 'Inspector' })).toBeInTheDocument();
		expect(screen.getByRole('region', { name: 'Page shelf' })).toBeInTheDocument();
		expect(screen.getByRole('contentinfo', { name: 'Status' })).toBeInTheDocument();
		expect(await screen.findByText('Folio 1.2.3')).toBeInTheDocument();
	});

	it('hides the menu bar by default and shows it when the setting turns on', async () => {
		const settings = renderScreen();
		expect(screen.queryByRole('menubar')).not.toBeInTheDocument();
		await settings.update({ interface: { menuBar: true } });
		expect(await screen.findByRole('menubar', { name: 'Menu bar' })).toBeInTheDocument();
	});

	it('switches between Easy and Expert through the settings', async () => {
		const settings = renderScreen();
		const easy = await screen.findByRole('radio', { name: 'Easy' });
		expect(easy).toBeChecked();
		await userEvent.click(screen.getByRole('radio', { name: 'Expert' }));
		await waitFor(async () => expect((await settings.get()).settings.mode).toBe('expert'));
		expect(screen.getByRole('radio', { name: 'Expert' })).toBeChecked();
	});

	it('shows the Jax perch only while Jax is on', async () => {
		const settings = renderScreen();
		expect(screen.getByText('Jax')).toBeInTheDocument();
		await settings.update({ jax: 'off' });
		await waitFor(() => expect(screen.queryByText('Jax')).not.toBeInTheDocument());
	});
});
