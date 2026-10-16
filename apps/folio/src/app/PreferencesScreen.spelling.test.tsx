// Tests for the Spelling page of Preferences: the switch, the dictionaries and the words added
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { WindowControls } from '@liminal-hq/folio-chrome/TitleBar/windowControls';
import { WindowChromeProvider } from '@liminal-hq/folio-chrome/WindowChromeProvider/WindowChromeProvider';
import type { DictionaryProgress } from '@liminal-hq/folio-protocol/generated/DictionaryProgress';
import { act, render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it } from 'vitest';
import { createFakeBooksClient } from '../books/fakeBooksClient';
import { createFakeSettingsClient } from '../settings/fakeSettingsClient';
import { SettingsProvider } from '../settings/SettingsProvider';
import { PreferencesScreen } from './PreferencesScreen';
import { DictionaryApiContext, type DictionaryApi } from './useDictionaries';

const controls: WindowControls = {
	minimize: () => {},
	toggleMaximize: () => {},
	close: () => {},
	setAlwaysOnTop: () => {},
	isMaximized: () => false,
	onMaximizedChange: () => () => {},
};

async function openSpelling() {
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
	await userEvent.click(await screen.findByRole('button', { name: 'Spelling' }));
	return { settings, books };
}

describe('the Spelling page', () => {
	it('turns spelling off and on', async () => {
		const { settings } = await openSpelling();
		await userEvent.click(screen.getByRole('switch', { name: 'Check spelling while typing' }));
		await waitFor(async () => expect((await settings.get()).settings.spelling.enabled).toBe(false));
	});

	it('lists the dictionary that comes with Folio and those that can be downloaded', async () => {
		await openSpelling();
		expect(await screen.findByText('English (Canada)')).toBeInTheDocument();
		expect(screen.getByRole('button', { name: /Included/ })).toBeDisabled();
		expect(await screen.findByText('French')).toBeInTheDocument();
		expect(screen.getByText('1.4 MB to download · MPL-2.0')).toBeInTheDocument();
	});

	it('downloads a dictionary and then offers to remove it', async () => {
		const { books } = await openSpelling();
		await userEvent.click(
			await screen.findByRole('button', { name: 'Download', description: /French/ }),
		);
		await waitFor(() => expect(books.calls).toContain('downloadDictionary("fr")'));
		const remove = await screen.findByRole('button', { name: /^Remove/, description: /French/ });
		await userEvent.click(remove);
		await waitFor(() => expect(books.calls).toContain('removeDictionary("fr")'));
	});

	it('says why a download failed and leaves the list as it was', async () => {
		const { books } = await openSpelling();
		books.refuseNext('downloadDictionary', 'dictionary', 'the file has the wrong hash');
		await userEvent.click(
			await screen.findByRole('button', { name: 'Download', description: /French/ }),
		);
		expect(await screen.findByRole('alert')).toHaveTextContent(
			'The dictionary could not be changed: the file has the wrong hash',
		);
		expect(screen.getAllByRole('button', { name: /^Download/ }).length).toBeGreaterThan(0);
	});

	it('lists the words added to the dictionary and removes one', async () => {
		const { settings } = await openSpelling();
		await settings.update({ spelling: { personal: ['Zorblat', 'Pip'] } } as never);
		expect(await screen.findByText('Zorblat')).toBeInTheDocument();
		await userEvent.click(
			screen.getAllByRole('button', { name: /^Remove/, description: /Zorblat/ })[0]!,
		);
		await waitFor(async () =>
			expect((await settings.get()).settings.spelling.personal).toEqual(['Pip']),
		);
	});

	it('shows how far a download has got behind its row, and takes the bar away when it is done', async () => {
		const books = createFakeBooksClient();
		let hear: ((progress: DictionaryProgress) => void) | undefined;
		let finish: (() => void) | undefined;
		const api: DictionaryApi = {
			...books.client,
			onDictionaryProgress: async (listener) => {
				hear = listener;
				return () => {};
			},
			downloadDictionary: () =>
				new Promise((resolve) => {
					finish = () => resolve(books.client.dictionaries());
				}),
		};
		render(
			<DictionaryApiContext.Provider value={api}>
				<SettingsProvider client={createFakeSettingsClient()}>
					<WindowChromeProvider controls={controls}>
						<PreferencesScreen />
					</WindowChromeProvider>
				</SettingsProvider>
			</DictionaryApiContext.Provider>,
		);
		await userEvent.click(await screen.findByRole('button', { name: 'Spelling' }));
		expect(screen.queryByRole('progressbar')).toBeNull();

		await userEvent.click(
			await screen.findByRole('button', { name: 'Download', description: /French/ }),
		);
		await waitFor(() => expect(hear).toBeDefined());
		act(() => hear!({ id: 'fr', received: 350, total: 1000 }));
		const bar = await screen.findByRole('progressbar', {
			name: 'Downloading the French dictionary',
		});
		expect(bar).toHaveAttribute('value', '0.35');
		// Only the dictionary being fetched has one.
		expect(screen.getAllByRole('progressbar')).toHaveLength(1);

		act(() => hear!({ id: 'fr', received: 1000, total: 1000 }));
		expect(bar).toHaveAttribute('value', '1');
		await act(async () => finish!());
		await waitFor(() => expect(screen.queryByRole('progressbar')).toBeNull());
	});
});
