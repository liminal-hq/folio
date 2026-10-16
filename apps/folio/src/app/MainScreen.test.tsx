// Tests for the main window's screen: the welcome screen, starting a book, and working with it
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { WindowControls } from '@liminal-hq/folio-chrome/TitleBar/windowControls';
import { WindowChromeProvider } from '@liminal-hq/folio-chrome/WindowChromeProvider/WindowChromeProvider';
import { act, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it } from 'vitest';
import { BooksProvider } from '../books/BooksProvider';
import { createFakeBooksClient } from '../books/fakeBooksClient';
import type { FileDialogs } from '../books/dialogs';
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

function setup(picks: { open?: string | null; save?: string | null } = {}) {
	const settings = createFakeSettingsClient();
	const books = createFakeBooksClient();
	const dialogs: FileDialogs = {
		pickBook: async () => picks.open ?? null,
		pickSavePath: async () => picks.save ?? null,
		pickImage: async () => null,
	};
	render(
		<SettingsProvider client={settings}>
			<WindowChromeProvider controls={controls}>
				<BooksProvider client={books.client} dialogs={dialogs}>
					<MainScreen client={info} />
				</BooksProvider>
			</WindowChromeProvider>
		</SettingsProvider>,
	);
	return { settings, books };
}

const startBook = async () => {
	await userEvent.click(await screen.findByRole('button', { name: /^Start / }));
	await screen.findByRole('toolbar', { name: 'Toolbar' });
};

describe('the welcome screen', () => {
	it('opens first, with the six kinds, the questions and the places to pick up', async () => {
		setup();
		expect(
			await screen.findByRole('heading', { name: 'What are you making?' }),
		).toBeInTheDocument();
		const kinds = await screen.findAllByRole('radio', {
			name: /book|Novel|Textbook|Poetry|Memoir|Illustrated/,
		});
		expect(kinds.length).toBeGreaterThanOrEqual(6);
		expect(screen.getByRole('radiogroup', { name: 'Book size' })).toBeInTheDocument();
		expect(
			screen.getByRole('radiogroup', { name: 'How do you like to work?' }),
		).toBeInTheDocument();
		expect(
			screen.getByRole('radiogroup', { name: 'Would you like Jax’s help?' }),
		).toBeInTheDocument();
		expect(screen.getByRole('heading', { name: 'Pick up where you left off' })).toBeInTheDocument();
		expect(screen.getByText('Books you open or save will be here.')).toBeInTheDocument();
		expect(screen.queryByRole('toolbar', { name: 'Toolbar' })).not.toBeInTheDocument();
		expect(screen.getByText('No book open')).toBeInTheDocument();
	});

	it('offers the sizes of the kind chosen', async () => {
		setup();
		const novel = await screen.findByRole('radio', { name: /Novel/ });
		await userEvent.click(novel);
		const sizes = screen.getByRole('radiogroup', { name: 'Book size' });
		expect(within(sizes).getByRole('radio', { name: '5 × 8 in' })).toBeChecked();
		expect(within(sizes).getByRole('radio', { name: '6 × 9 in' })).toBeInTheDocument();
		expect(screen.getByRole('button', { name: 'Start Novel' })).toBeInTheDocument();
	});

	it('starts the kind and size chosen, and sets how she likes to work', async () => {
		const { settings, books } = setup();
		await userEvent.click(await screen.findByRole('radio', { name: /Novel/ }));
		const sizes = screen.getByRole('radiogroup', { name: 'Book size' });
		await userEvent.click(within(sizes).getByRole('radio', { name: '6 × 9 in' }));
		await userEvent.click(screen.getByRole('radio', { name: 'Expert' }));
		await userEvent.click(screen.getByRole('radio', { name: 'No thanks' }));
		await userEvent.click(screen.getByRole('button', { name: 'Start Novel' }));

		await screen.findByRole('toolbar', { name: 'Toolbar' });
		expect(books.calls).toContain('newBook({"kind":"novel","trimIndex":1})');
		const now = (await settings.get()).settings;
		expect(now.mode).toBe('expert');
		expect(now.jax).toBe('off');
	});

	it('lists the recent books and opens one when it is chosen', async () => {
		const { settings, books } = setup();
		await settings.update({ recentBooks: ['/home/mom/Books/Pip/Pip.folio'] });
		const row = await screen.findByRole('button', { name: /Pip/ });
		expect(within(row).getByText('/home/mom/Books/Pip')).toBeInTheDocument();
		await userEvent.click(row);
		await screen.findByRole('toolbar', { name: 'Toolbar' });
		expect(books.calls).toContain('openBook("/home/mom/Books/Pip/Pip.folio")');
	});

	it('opens a book from the picker, and says what is not ready yet', async () => {
		const { books } = setup({ open: '/books/other.folio' });
		expect(await screen.findByRole('button', { name: /Import words/ })).toBeDisabled();
		expect(screen.getByRole('button', { name: /Open a layout/ })).toBeDisabled();
		await userEvent.click(screen.getByRole('button', { name: 'Open a Folio book…' }));
		await screen.findByRole('toolbar', { name: 'Toolbar' });
		expect(books.calls).toContain('openBook("/books/other.folio")');
	});
});

/** The ribbon's tabs, apart from the sidebar's own. */
const ribbonTabs = () => screen.getByRole('tablist', { name: 'Ribbon tabs' });

describe('a book in the main window', () => {
	it('shows its title, size, length and that it has not been saved', async () => {
		setup();
		await startBook();
		expect(screen.getByRole('contentinfo', { name: 'Status' })).toHaveTextContent(
			'Picture book · 8.5 × 8.5 in',
		);
		expect(screen.getByRole('contentinfo', { name: 'Status' })).toHaveTextContent('Not saved yet');
		expect(screen.getByRole('heading', { name: /^Spread / })).toBeInTheDocument();
		expect(screen.getByRole('group', { name: 'Page' })).toBeInTheDocument();
		expect(screen.getByRole('list', { name: 'Spreads' })).toBeInTheDocument();
		expect(await screen.findByText('Folio 1.2.3')).toBeInTheDocument();
	});

	it('shows the spread the shelf selects, and zooms it', async () => {
		const { books } = setup();
		await startBook();
		const shelf = screen.getByRole('list', { name: 'Spreads' });
		const items = within(shelf).getAllByRole('button');
		expect(items.length).toBeGreaterThan(1);
		expect(items[0]).toHaveAttribute('aria-current', 'true');
		await userEvent.click(items[1]!);
		expect(items[1]).toHaveAttribute('aria-current', 'true');
		expect(screen.getByRole('heading', { name: /^Spread / })).toHaveTextContent(/Page/);
		await waitFor(() =>
			expect(books.calls.filter((c) => c.startsWith('layoutSpread')).length).toBeGreaterThan(1),
		);
		const before = screen.getByRole('group', { name: 'Zoom' }).querySelector('output')!.textContent;
		await userEvent.click(screen.getByRole('button', { name: 'Zoom in' }));
		expect(
			screen.getByRole('group', { name: 'Zoom' }).querySelector('output')!.textContent,
		).not.toBe(before);
		await userEvent.click(screen.getByRole('button', { name: 'Fit' }));
		expect(screen.getByRole('button', { name: 'Fit' })).toHaveAttribute('aria-pressed', 'true');
	});

	it('saves with Ctrl+S, asking where the first time', async () => {
		setup({ save: '/books/Pip/Pip' });
		await startBook();
		fireEvent.keyDown(window, { key: 's', ctrlKey: true });
		await waitFor(() =>
			expect(screen.getByRole('contentinfo', { name: 'Status' })).toHaveTextContent(
				'Saved · Pip.folio',
			),
		);
	});

	it('goes back to the welcome screen when an untouched new book is closed', async () => {
		setup();
		await startBook();
		fireEvent.keyDown(window, { key: 'w', ctrlKey: true });
		expect(
			await screen.findByRole('heading', { name: 'What are you making?' }),
		).toBeInTheDocument();
	});

	it('asks about changes before closing, and keeps the book on cancel', async () => {
		const { books } = setup({ save: '/books/Pip/Pip' });
		await startBook();
		const id = (await books.client.list())[0]!.id;
		await act(async () => {
			await books.client.apply(id, { type: 'MoveFrame' } as never);
		});
		fireEvent.keyDown(window, { key: 'w', ctrlKey: true });
		const dialog = await screen.findByRole('dialog', { name: /Save changes to/ });
		await userEvent.click(within(dialog).getByRole('button', { name: 'Cancel' }));
		await waitFor(() => expect(screen.queryByRole('dialog')).not.toBeInTheDocument());
		expect(screen.getByRole('toolbar', { name: 'Toolbar' })).toBeInTheDocument();

		fireEvent.keyDown(window, { key: 'w', ctrlKey: true });
		const again = await screen.findByRole('dialog', { name: /Save changes to/ });
		await userEvent.click(within(again).getByRole('button', { name: 'Don’t save' }));
		expect(
			await screen.findByRole('heading', { name: 'What are you making?' }),
		).toBeInTheDocument();
	});

	it('undoes and redoes with the keys once there is something to undo', async () => {
		const { books } = setup();
		await startBook();
		const id = (await books.client.list())[0]!.id;
		await act(async () => {
			await books.client.apply(id, { type: 'MoveFrame' } as never);
		});
		fireEvent.keyDown(window, { key: 'z', ctrlKey: true });
		await waitFor(() => expect(books.calls.some((c) => c.startsWith('undo('))).toBe(true));
		fireEvent.keyDown(window, { key: 'z', ctrlKey: true, shiftKey: true });
		await waitFor(() => expect(books.calls.some((c) => c.startsWith('redo('))).toBe(true));
	});

	it('hides the menu bar by default and shows it when the setting turns on', async () => {
		const { settings } = setup();
		await startBook();
		expect(screen.queryByRole('menubar')).not.toBeInTheDocument();
		await settings.update({ interface: { menuBar: true } });
		expect(await screen.findByRole('menubar', { name: 'Menu bar' })).toBeInTheDocument();
	});

	it('switches between Easy and Expert through the settings', async () => {
		const { settings } = setup();
		await startBook();
		const toolbar = screen.getByRole('group', { name: 'Toolbar' });
		await userEvent.click(within(toolbar).getByRole('radio', { name: 'Expert' }));
		await waitFor(async () => expect((await settings.get()).settings.mode).toBe('expert'));
	});

	it('shows the ribbon with its tabs, and Expert reveals the expert groups', async () => {
		setup();
		await startBook();
		expect(
			within(ribbonTabs())
				.getAllByRole('tab')
				.map((tab) => tab.textContent),
		).toEqual(['Home', 'Insert', 'Pages', 'Layout', 'View', 'Export']);
		await userEvent.click(within(ribbonTabs()).getByRole('tab', { name: 'Pages' }));
		expect(screen.queryByRole('group', { name: 'Masters' })).not.toBeInTheDocument();
		await userEvent.click(screen.getByRole('radio', { name: 'Expert' }));
		expect(await screen.findByRole('group', { name: 'Masters' })).toBeInTheDocument();
	});

	it('switches to the classic toolbar when the setting says so, and back with its ribbon button', async () => {
		const { settings } = setup();
		await startBook();
		await settings.update({ interface: { toolbar: 'classic' } });
		const toolbar = await screen.findByRole('toolbar', { name: 'Toolbar' });
		expect(screen.queryByRole('tablist', { name: 'Ribbon tabs' })).not.toBeInTheDocument();
		await userEvent.click(within(toolbar).getByRole('button', { name: 'Switch to the ribbon' }));
		await waitFor(async () =>
			expect((await settings.get()).settings.interface.toolbar).toBe('ribbon'),
		);
		expect(await screen.findByRole('tablist', { name: 'Ribbon tabs' })).toBeInTheDocument();
	});

	it('runs undo from the ribbon once there is something to undo', async () => {
		const { books } = setup();
		await startBook();
		expect(screen.getByRole('button', { name: 'Undo' })).toBeDisabled();
		const id = (await books.client.list())[0]!.id;
		await act(async () => {
			await books.client.apply(id, { type: 'MoveFrame' } as never);
		});
		await userEvent.click(await screen.findByRole('button', { name: 'Undo' }));
		await waitFor(() => expect(books.calls.some((c) => c.startsWith('undo('))).toBe(true));
	});

	it('adds a text box from the ribbon, and turns on the text controls for a selected text frame', async () => {
		const { books } = setup();
		await startBook();
		await userEvent.click(screen.getByRole('button', { name: 'Text box' }));
		await waitFor(() =>
			expect(books.calls.some((c) => c.startsWith('apply(') && c.includes('AddFrame'))).toBe(true),
		);
		expect(screen.getByRole('button', { name: 'Bold' })).toBeDisabled();
		await userEvent.click(screen.getByRole('button', { name: /^Page 1/ }));
		// Pressing each frame in turn, one of the sample's frames is text and wakes the text controls.
		let woke = false;
		for (const option of screen.getAllByRole('option')) {
			fireEvent.pointerDown(option, { button: 0 });
			fireEvent.pointerUp(option);
			if (!(screen.getByRole('button', { name: 'Bold' }) as HTMLButtonElement).disabled)
				woke = true;
		}
		expect(woke).toBe(true);
	});

	it('zooms the stage from the ribbon', async () => {
		setup();
		await startBook();
		const level = () => screen.getByLabelText(/^Zoom \d+%/).textContent;
		const before = level();
		await userEvent.click(within(ribbonTabs()).getByRole('tab', { name: 'View' }));
		await userEvent.click(
			within(screen.getByRole('tabpanel', { name: 'View' })).getByRole('button', {
				name: 'Zoom in',
			}),
		);
		await waitFor(() => expect(level()).not.toBe(before));
	});

	it('keeps the page marks in the settings, so they hold for the next book', async () => {
		const { settings } = setup();
		await startBook();
		await userEvent.click(screen.getByRole('checkbox', { name: 'Columns' }));
		await waitFor(async () => expect((await settings.get()).settings.marks.columns.on).toBe(true));
		await userEvent.click(screen.getByRole('button', { name: 'Marks' }));
		await userEvent.click(screen.getByRole('button', { name: 'Margins colour 3' }));
		await waitFor(async () => expect((await settings.get()).settings.marks.margins.colour).toBe(2));
	});
});
