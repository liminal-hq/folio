// Tests for the About, Preferences and Book tools screens
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { WindowControls } from '@liminal-hq/folio-chrome/TitleBar/windowControls';
import { WindowChromeProvider } from '@liminal-hq/folio-chrome/WindowChromeProvider/WindowChromeProvider';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import type { ReactNode } from 'react';
import { describe, expect, it, vi } from 'vitest';
import type { AppInfoClient } from '../services/appInfoClient';
import { createFakeSettingsClient } from '../settings/fakeSettingsClient';
import { SettingsProvider } from '../settings/SettingsProvider';
import { AboutScreen } from './AboutScreen';
import { BookToolsScreen } from './BookToolsScreen';
import { PreferencesScreen } from './PreferencesScreen';

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

function mount(children: ReactNode, settings = createFakeSettingsClient()) {
	render(
		<SettingsProvider client={settings}>
			<WindowChromeProvider controls={controls}>{children}</WindowChromeProvider>
		</SettingsProvider>,
	);
	return settings;
}

describe('AboutScreen', () => {
	it('shows the name, tagline and version, with only a close button in the title bar', async () => {
		mount(<AboutScreen client={info} />);
		expect(screen.getByRole('heading', { name: 'Folio' })).toBeInTheDocument();
		expect(
			screen.getByText('Books, laid out by hand, without the hard parts.'),
		).toBeInTheDocument();
		expect(await screen.findByText('Version 1.2.3')).toBeInTheDocument();
		expect(screen.getByRole('button', { name: 'Close' })).toBeInTheDocument();
		expect(screen.queryByRole('button', { name: 'Minimise' })).not.toBeInTheDocument();
		expect(screen.queryByRole('button', { name: 'Maximise' })).not.toBeInTheDocument();
	});

	it('copies the build details', async () => {
		const copy = vi.fn(async () => {});
		mount(<AboutScreen client={info} copy={copy} />);
		await userEvent.click(await screen.findByRole('button', { name: 'Copy build details' }));
		expect(copy).toHaveBeenCalledWith('Folio 1.2.3 (ca.liminalhq.folio) on linux');
		expect(await screen.findByRole('button', { name: 'Copied' })).toBeInTheDocument();
	});

	it('still shows the name when the build details are not available', () => {
		mount(
			<AboutScreen
				client={{
					appInfo: async () => {
						throw new Error('not in Tauri');
					},
				}}
			/>,
		);
		expect(screen.getByRole('heading', { name: 'Folio' })).toBeInTheDocument();
		expect(screen.queryByRole('button', { name: 'Copy build details' })).not.toBeInTheDocument();
	});
});

describe('PreferencesScreen', () => {
	it('changes settings as the rows are used', async () => {
		const settings = mount(<PreferencesScreen />);
		await userEvent.click(await screen.findByRole('switch', { name: 'Show the menu bar' }));
		await waitFor(async () => expect((await settings.get()).settings.interface.menuBar).toBe(true));
		await userEvent.click(screen.getByRole('radio', { name: 'Expert' }));
		await waitFor(async () => expect((await settings.get()).settings.mode).toBe('expert'));
	});

	it('moves between its pages', async () => {
		const settings = mount(<PreferencesScreen />);
		await userEvent.click(await screen.findByRole('button', { name: 'Appearance' }));
		await userEvent.click(screen.getByRole('radio', { name: 'Dark' }));
		await waitFor(async () => expect((await settings.get()).settings.colourMode).toBe('dark'));
		await userEvent.click(screen.getByRole('button', { name: 'Units' }));
		await userEvent.click(screen.getByRole('radio', { name: 'Millimetres' }));
		await waitFor(async () => expect((await settings.get()).settings.units).toBe('millimetres'));
	});
});

describe('BookToolsScreen', () => {
	it('says what will open there', () => {
		mount(<BookToolsScreen />);
		expect(screen.getByRole('heading', { name: 'Book tools' })).toBeInTheDocument();
	});
});
