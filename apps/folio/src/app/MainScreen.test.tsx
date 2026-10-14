// Tests for the main window's screen
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { WindowChromeProvider } from '@liminal-hq/folio-chrome/WindowChromeProvider/WindowChromeProvider';
import type { WindowControls } from '@liminal-hq/folio-chrome/TitleBar/windowControls';
import { MainScreen } from './MainScreen';
import type { AppInfoClient } from '../services/appInfoClient';

const controls: WindowControls = {
	minimize: () => {},
	toggleMaximize: () => {},
	close: () => {},
	setAlwaysOnTop: () => {},
	isMaximized: () => false,
	onMaximizedChange: () => () => {},
};

function renderScreen(client: AppInfoClient) {
	return render(
		<WindowChromeProvider controls={controls}>
			<MainScreen client={client} />
		</WindowChromeProvider>,
	);
}

describe('MainScreen', () => {
	it('shows what the app reports about itself', async () => {
		renderScreen({
			appInfo: async () => ({
				name: 'Folio',
				version: '1.2.3',
				identifier: 'ca.liminalhq.folio',
				platform: 'linux',
			}),
		});
		expect(await screen.findByText('Folio 1.2.3 is running on linux.')).toBeInTheDocument();
		expect(screen.getByText('Application identifier: ca.liminalhq.folio')).toBeInTheDocument();
	});

	it('says so when the app details are not available', async () => {
		renderScreen({
			appInfo: async () => {
				throw new Error('not in Tauri');
			},
		});
		expect(
			await screen.findByText('The application details are not available here.'),
		).toBeInTheDocument();
	});
});
