// Tests for the About, Preferences and Book tools screens
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { WindowControls } from '@liminal-hq/folio-chrome/TitleBar/windowControls';
import { WindowChromeProvider } from '@liminal-hq/folio-chrome/WindowChromeProvider/WindowChromeProvider';
import { act, render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import type { ReactNode } from 'react';
import { describe, expect, it, vi } from 'vitest';
import { BooksProvider } from '../books/BooksProvider';
import type { FileDialogs } from '../books/dialogs';
import { createFakeBooksClient } from '../books/fakeBooksClient';
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

const noDialogs: FileDialogs = {
	pickBook: async () => null,
	pickSavePath: async () => null,
	pickImage: async () => null,
	pickFolder: async () => null,
	pickText: async () => null,
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

	it('turns the title bar quick actions off and on', async () => {
		const settings = mount(<PreferencesScreen />);
		const row = await screen.findByRole('switch', { name: 'Show quick actions in the title bar' });
		expect(row).toBeChecked();
		await userEvent.click(row);
		await waitFor(async () =>
			expect((await settings.get()).settings.interface.quickActions).toBe(false),
		);
		await userEvent.click(row);
		await waitFor(async () =>
			expect((await settings.get()).settings.interface.quickActions).toBe(true),
		);
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
	it('opens on the group it was asked for, and says when no book is open', async () => {
		const fake = createFakeBooksClient();
		const tools = {
			area: async () => ({ area: 'pictures', tool: null }),
			onArea: async () => () => {},
		};
		mount(
			<BooksProvider client={fake.client} dialogs={noDialogs} follow>
				<BookToolsScreen client={tools} />
			</BooksProvider>,
		);
		expect(
			await screen.findByRole('tab', { name: 'Pictures', selected: true }),
		).toBeInTheDocument();
		expect(screen.getByText(/Open a book in the main window/)).toBeInTheDocument();
	});

	it('follows the book the main window shows', async () => {
		const fake = createFakeBooksClient();
		const made = await fake.client.newBook({ title: 'Pip' });
		const tools = {
			area: async () => ({ area: 'structure', tool: null }),
			onArea: async () => () => {},
		};
		mount(
			<BooksProvider client={fake.client} dialogs={noDialogs} follow>
				<BookToolsScreen client={tools} />
			</BooksProvider>,
		);
		await waitFor(() => expect(screen.queryByText(/Open a book in the main window/)).toBeNull());
		expect(screen.getByRole('tab', { name: 'Book structure', selected: true })).toBeInTheDocument();
		fake.emit.active(made.summary.id);
		expect(await screen.findByText(/Pip · /)).toBeInTheDocument();
	});

	it('opens on the tool it was asked for, and moves to another it is asked for later', async () => {
		const fake = createFakeBooksClient();
		await fake.client.newBook({ title: 'Pip' });
		let ask: (asked: { area: string; tool: string | null }) => void = () => {};
		const tools = {
			area: async () => ({ area: 'pictures', tool: 'pictures.alt' }),
			onArea: async (listener: typeof ask) => {
				ask = listener;
				return () => {};
			},
		};
		mount(
			<BooksProvider client={fake.client} dialogs={noDialogs} follow>
				<BookToolsScreen client={tools} />
			</BooksProvider>,
		);
		expect(await screen.findByRole('heading', { name: 'Alt text' })).toBeInTheDocument();
		await userEvent.click(screen.getByRole('button', { name: 'Pictures in this book' }));
		expect(screen.getByRole('heading', { name: 'Pictures in this book' })).toBeInTheDocument();
		// Asked for the same tool again after another was picked, it comes back.
		act(() => ask({ area: 'pictures', tool: 'pictures.alt' }));
		expect(await screen.findByRole('heading', { name: 'Alt text' })).toBeInTheDocument();
		act(() => ask({ area: 'publishing', tool: 'publishing.keywords' }));
		expect(await screen.findByRole('heading', { name: 'Keywords' })).toBeInTheDocument();
		// An area alone leaves each area on the tool last shown in it.
		act(() => ask({ area: 'pictures', tool: null }));
		expect(await screen.findByRole('heading', { name: 'Alt text' })).toBeInTheDocument();
	});
});

describe('windows without a book', () => {
	it('have no quick actions in their title bars', () => {
		mount(<AboutScreen client={info} />);
		expect(screen.queryByRole('toolbar', { name: 'Quick actions' })).not.toBeInTheDocument();
	});

	it('have none in Preferences either', async () => {
		mount(<PreferencesScreen />);
		await screen.findByRole('switch', { name: 'Show the menu bar' });
		expect(screen.queryByRole('toolbar', { name: 'Quick actions' })).not.toBeInTheDocument();
	});
});
