// Tests for the Export page of Preferences: the picture settings and the folder
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { WindowControls } from '@liminal-hq/folio-chrome/TitleBar/windowControls';
import { WindowChromeProvider } from '@liminal-hq/folio-chrome/WindowChromeProvider/WindowChromeProvider';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it } from 'vitest';
import { createFakeBooksClient } from '../books/fakeBooksClient';
import { createFakeSettingsClient } from '../settings/fakeSettingsClient';
import { SettingsProvider } from '../settings/SettingsProvider';
import { PreferencesScreen } from './PreferencesScreen';
import { DictionaryApiContext } from './useDictionaries';

const controls: WindowControls = {
	minimize: () => {},
	toggleMaximize: () => {},
	close: () => {},
	setAlwaysOnTop: () => {},
	isMaximized: () => false,
	onMaximizedChange: () => () => {},
};

async function openExport() {
	const settings = createFakeSettingsClient();
	const books = createFakeBooksClient();
	render(
		<DictionaryApiContext.Provider value={books.client}>
			<SettingsProvider client={settings}>
				<WindowChromeProvider controls={controls}>
					<PreferencesScreen />
				</WindowChromeProvider>
			</SettingsProvider>
		</DictionaryApiContext.Provider>,
	);
	await userEvent.click(await screen.findByRole('button', { name: 'Export' }));
	return settings;
}

describe('the Export page', () => {
	it('keeps the picture setting chosen for Sharing PDFs', async () => {
		const settings = await openExport();
		await userEvent.click(await screen.findByRole('radio', { name: 'Small' }));
		await waitFor(async () =>
			expect((await settings.get()).settings.export.pdf.sharing.preset).toBe('small'),
		);
	});

	it('keeps the press setting for the paperback and the print shop together', async () => {
		const settings = await openExport();
		const radios = await screen.findAllByRole('radio', { name: 'Original' });
		await userEvent.click(radios.at(-1)!);
		await waitFor(async () => {
			const pdf = (await settings.get()).settings.export.pdf;
			expect(pdf.kdp.preset).toBe('original');
			expect(pdf.printShop.preset).toBe('original');
		});
	});

	it('keeps a folder for exports', async () => {
		const settings = await openExport();
		const field = await screen.findByRole('textbox', { name: 'Put exports in' });
		await userEvent.type(field, '/home/mom/Books/out{Enter}');
		await waitFor(async () =>
			expect((await settings.get()).settings.export.folder).toBe('/home/mom/Books/out'),
		);
	});
});
