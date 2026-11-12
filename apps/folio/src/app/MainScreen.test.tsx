// Tests for the main window's screen: the welcome screen, starting a book, and working with it
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { WindowControls } from '@liminal-hq/folio-chrome/TitleBar/windowControls';
import { WindowChromeProvider } from '@liminal-hq/folio-chrome/WindowChromeProvider/WindowChromeProvider';
import { act, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import { BooksProvider } from '../books/BooksProvider';
import { createFakeBooksClient } from '../books/fakeBooksClient';
import type { FileDialogs } from '../books/dialogs';
import type { AppInfoClient } from '../services/appInfoClient';
import type { CloseRequests } from '../services/closeRequests';
import type { OpenFilesClient } from '../services/openFilesClient';
import { createFakeSettingsClient } from '../settings/fakeSettingsClient';
import { SettingsProvider } from '../settings/SettingsProvider';
import { MainScreen } from './MainScreen';
import { WindowsContext } from './WindowsContext';

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

/** A window that can be asked to close, as the ✕, Alt+F4 and Quit ask it. */
function closableWindow() {
	let allow: (() => Promise<boolean>) | undefined;
	const requests: CloseRequests = {
		onCloseRequested: async (guard) => {
			allow = guard;
			return () => {
				allow = undefined;
			};
		},
	};
	return { requests, askToClose: () => allow!() };
}

function setup(
	picks: { open?: string | null; save?: string | null } = {},
	closing?: CloseRequests,
	openFiles?: OpenFilesClient,
) {
	const settings = createFakeSettingsClient();
	const books = createFakeBooksClient();
	const windows = { open: vi.fn(async () => {}), openBookTools: vi.fn(async () => {}) };
	const dialogs: FileDialogs = {
		pickBook: async () => picks.open ?? null,
		pickSavePath: async () => picks.save ?? null,
		pickImage: async () => null,
		pickFolder: async () => null,
		pickText: async () => null,
	};
	render(
		<SettingsProvider client={settings}>
			<WindowChromeProvider controls={controls}>
				<BooksProvider client={books.client} dialogs={dialogs}>
					<WindowsContext.Provider value={windows}>
						<MainScreen
							client={info}
							{...(closing ? { closeRequests: closing } : {})}
							{...(openFiles ? { openFilesClient: openFiles } : {})}
						/>
					</WindowsContext.Provider>
				</BooksProvider>
			</WindowChromeProvider>
		</SettingsProvider>,
	);
	return { settings, books, windows };
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

	it('opens the New book wizard from Guide me', async () => {
		const { windows } = setup();
		await userEvent.click(await screen.findByRole('button', { name: 'Guide me…' }));
		expect(windows.open).toHaveBeenCalledWith('WizardNew');
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

describe('work a crash left behind', () => {
	const left = {
		id: 'r1',
		title: 'The Otter Who Couldn’t Sleep',
		path: null,
		savedAt: 1_700_000_000,
	};

	it('is offered on the start screen, and restored as a book that is still unsaved', async () => {
		const { books } = setup();
		books.recoveries.push(left);
		expect(
			await screen.findByRole('heading', { name: 'Unsaved work from last time' }),
		).toBeInTheDocument();
		expect(screen.getByText(/Never saved/)).toBeInTheDocument();
		await userEvent.click(
			screen.getByRole('button', { name: 'Restore The Otter Who Couldn’t Sleep' }),
		);
		await screen.findByRole('toolbar', { name: 'Toolbar' });
		expect(screen.getByText('Not saved yet')).toBeInTheDocument();
		expect(books.recoveries).toEqual([]);
	});

	it('can be given up, and the offer goes with it', async () => {
		const { books } = setup();
		books.recoveries.push(left);
		await userEvent.click(
			await screen.findByRole('button', {
				name: 'Discard the unsaved work on The Otter Who Couldn’t Sleep',
			}),
		);
		await waitFor(() =>
			expect(
				screen.queryByRole('heading', { name: 'Unsaved work from last time' }),
			).not.toBeInTheDocument(),
		);
		expect(books.recoveries).toEqual([]);
	});

	it('says nothing when there is nothing to offer', async () => {
		setup();
		await screen.findByRole('heading', { name: 'What are you making?' });
		expect(
			screen.queryByRole('heading', { name: 'Unsaved work from last time' }),
		).not.toBeInTheDocument();
	});
});

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

	it('keeps the sidebar group the writer opened while a frame stays selected', async () => {
		setup();
		await startBook();
		await userEvent.click(screen.getByRole('radio', { name: 'Expert' }));
		const shelf = within(screen.getByRole('list', { name: 'Spreads' })).getAllByRole('button');
		await userEvent.click(shelf[1]!);
		const hits = () => Array.from(document.querySelectorAll('[data-hit]'));
		await waitFor(() => expect(hits().length).toBeGreaterThan(0));
		const frames = hits();
		fireEvent.pointerDown(frames[0]!, { button: 0, clientX: 10, clientY: 10 });
		fireEvent.pointerUp(frames[0]!, { button: 0, clientX: 10, clientY: 10 });
		await waitFor(() =>
			expect(hits().filter((h) => h.getAttribute('aria-selected') === 'true').length).toBe(1),
		);
		const sidebar = screen.getByRole('complementary', { name: 'Sidebar' });
		expect(within(sidebar).getByRole('tab', { name: 'Properties' })).toHaveAttribute(
			'aria-selected',
			'true',
		);
		for (const name of ['Fonts', 'Pages', 'Swatches']) {
			await userEvent.click(within(sidebar).getByRole('tab', { name }));
			await act(async () => {
				await new Promise((r) => setTimeout(r, 20));
			});
			expect(within(sidebar).getByRole('tab', { name })).toHaveAttribute('aria-selected', 'true');
			expect(hits().filter((h) => h.getAttribute('aria-selected') === 'true').length).toBe(1);
		}
	});

	it('opens the Book size and Margins dialogs and goes to the cover from the Pages tab', async () => {
		setup();
		await startBook();
		await userEvent.click(within(ribbonTabs()).getByRole('tab', { name: 'Pages' }));
		const shelf = within(screen.getByRole('list', { name: 'Spreads' })).getAllByRole('button');
		await userEvent.click(shelf[1]!);
		expect(shelf[1]).toHaveAttribute('aria-current', 'true');
		const book = screen.getByRole('group', { name: 'Book' });
		await userEvent.click(within(book).getByRole('button', { name: 'Cover' }));
		expect(shelf[0]).toHaveAttribute('aria-current', 'true');
		await userEvent.click(within(book).getByRole('button', { name: 'Book size' }));
		const size = await screen.findByRole('dialog', { name: 'Book size' });
		await userEvent.click(within(size).getByRole('button', { name: 'Cancel' }));
		await userEvent.click(within(book).getByRole('button', { name: 'Margins' }));
		expect(await screen.findByRole('dialog', { name: 'Margins' })).toBeInTheDocument();
	});

	it('opens a Book tool from the Tools tab', async () => {
		const { windows } = setup();
		await startBook();
		await userEvent.click(within(ribbonTabs()).getByRole('tab', { name: 'Tools' }));
		await userEvent.click(screen.getByRole('button', { name: 'Keywords' }));
		await userEvent.click(screen.getByRole('button', { name: 'Alt text' }));
		expect(windows.openBookTools.mock.calls).toEqual([
			['publishing', 'publishing.keywords'],
			['pictures', 'pictures.alt'],
		]);
		expect(screen.getByRole('button', { name: 'Find & replace' })).toBeEnabled();
	});

	it('reads the fonts the book uses without the Fonts group being open', async () => {
		const { books } = setup();
		await startBook();
		await waitFor(() => expect(books.calls.some((c) => c.startsWith('fontsInUse('))).toBe(true));
		expect(screen.getByRole('tab', { name: 'Properties', selected: true })).toBeInTheDocument();
	});

	it('copies, pastes and cuts the selected frames with the keys, without the webview’s own copy', async () => {
		const { books } = setup();
		await startBook();
		await userEvent.click(
			within(screen.getByRole('list', { name: 'Spreads' })).getAllByRole('button')[1]!,
		);
		const hits = () => Array.from(document.querySelectorAll('[data-hit]'));
		await waitFor(() => expect(hits().length).toBeGreaterThan(0));
		fireEvent.pointerDown(hits()[0]!, { button: 0, clientX: 10, clientY: 10 });
		fireEvent.pointerUp(hits()[0]!, { button: 0, clientX: 10, clientY: 10 });
		const applied = (type: string) =>
			books.calls.filter((c) => c.startsWith('apply(') && c.includes(type)).length;

		// Copy is the program’s, so the webview is not asked to copy.
		expect(fireEvent.keyDown(document.body, { key: 'c', ctrlKey: true })).toBe(false);
		expect(applied('AddFrame')).toBe(0);
		// What the system clipboard then holds is the words of the copy, which is how a paste knows it.
		const copied = await waitFor(async () => {
			const words = await navigator.clipboard.readText();
			expect(words).not.toBe('');
			return words;
		});
		const holdingCopy = {
			clipboardData: {
				getData: (type: string) => (type === 'text/plain' ? copied : ''),
				files: [],
			},
		};
		fireEvent.paste(window, holdingCopy);
		await waitFor(() => expect(applied('AddFrame')).toBe(1));
		fireEvent.paste(window, holdingCopy);
		await waitFor(() => expect(applied('AddFrame')).toBe(2));

		// Words copied in another program are not taken for the frames.
		fireEvent.paste(window, {
			clipboardData: {
				getData: (type: string) => (type === 'text/plain' ? 'Elsewhere' : ''),
				files: [],
				types: [],
			},
		});
		expect(applied('AddFrame')).toBe(2);

		// Cut copies and takes the frame off the page. (The fake book does not grow the pasted frames, so
		// the first one is chosen again.)
		fireEvent.pointerDown(hits()[0]!, { button: 0, clientX: 10, clientY: 10 });
		fireEvent.pointerUp(hits()[0]!, { button: 0, clientX: 10, clientY: 10 });
		const before = applied('DeleteFrame');
		expect(fireEvent.keyDown(document.body, { key: 'x', ctrlKey: true })).toBe(false);
		await waitFor(() => expect(applied('DeleteFrame')).toBeGreaterThan(before));
	});

	it('copies and pastes with the Home tab’s Clipboard buttons', async () => {
		const { books } = setup();
		await startBook();
		await userEvent.click(
			within(screen.getByRole('list', { name: 'Spreads' })).getAllByRole('button')[1]!,
		);
		const group = screen.getByRole('group', { name: 'Clipboard' });
		expect(within(group).getByRole('button', { name: 'Copy' })).toBeDisabled();
		expect(within(group).getByRole('button', { name: 'Paste' })).toBeEnabled();
		const hits = () => Array.from(document.querySelectorAll('[data-hit]'));
		await waitFor(() => expect(hits().length).toBeGreaterThan(0));
		fireEvent.pointerDown(hits()[0]!, { button: 0, clientX: 10, clientY: 10 });
		fireEvent.pointerUp(hits()[0]!, { button: 0, clientX: 10, clientY: 10 });
		await userEvent.click(await within(group).findByRole('button', { name: 'Copy' }));
		await userEvent.click(within(group).getByRole('button', { name: 'Paste' }));
		const added = () => books.calls.filter((c) => c.includes('AddFrame')).length;
		await waitFor(() => expect(added()).toBe(1));
		await userEvent.click(within(group).getByRole('button', { name: 'Paste special' }));
		await userEvent.click(await screen.findByRole('menuitem', { name: 'Paste in place' }));
		await waitFor(() => expect(added()).toBe(2));
	});

	it('pastes a picture from the clipboard as a picture frame holding it', async () => {
		const { books } = setup();
		await startBook();
		const png = new File([new Uint8Array([137, 80, 78, 71, 13, 10, 26, 10])], 'shot.png', {
			type: 'image/png',
		});
		fireEvent.paste(window, {
			clipboardData: { getData: () => '', files: [png], types: ['Files'] },
		});
		await waitFor(() =>
			expect(books.calls.some((c) => c.startsWith('importImageBytes('))).toBe(true),
		);
		expect(books.calls.find((c) => c.startsWith('importImageBytes('))).toContain('"png"');
		await waitFor(() =>
			expect(
				books.calls.filter((c) => c.startsWith('apply(')).map((c) => c.match(/Add|SetImage/)?.[0]),
			).toEqual(['Add', 'SetImage']),
		);
	});

	it('says so when a pasted picture is refused', async () => {
		const { books } = setup();
		await startBook();
		const spy = vi.spyOn(books.client, 'importImageBytes').mockRejectedValue({ kind: 'image' });
		const png = new File([new Uint8Array([1, 2, 3])], 'bad.png', { type: 'image/png' });
		fireEvent.paste(window, {
			clipboardData: { getData: () => '', files: [png], types: ['Files'] },
		});
		expect(await screen.findByText(/could not be pasted/)).toBeInTheDocument();
		spy.mockRestore();
	});

	it('pastes only the words into a new text box with Ctrl+Shift+V', async () => {
		const { books } = setup();
		await startBook();
		const words = {
			clipboardData: {
				getData: (type: string) => (type === 'text/plain' ? 'Plain words' : ''),
				files: [],
			},
		};
		expect(fireEvent.keyDown(document.body, { key: 'V', ctrlKey: true, shiftKey: true })).toBe(
			true,
		);
		fireEvent.paste(window, words);
		await waitFor(() =>
			expect(books.calls.some((c) => c.startsWith('apply(') && c.includes('AddFrame'))).toBe(true),
		);
		// A plain paste of words is not repeated by the clipboard read that waits for the paste event.
		await new Promise((r) => setTimeout(r, 300));
		expect(books.calls.filter((c) => c.includes('AddFrame'))).toHaveLength(1);
		// An ordinary paste of words from another program is left alone.
		fireEvent.paste(window, words);
		expect(books.calls.filter((c) => c.includes('AddFrame'))).toHaveLength(1);
	});

	it('lists the ways to paste in the Paste special menu', async () => {
		setup();
		await startBook();
		await userEvent.click(
			within(screen.getByRole('list', { name: 'Spreads' })).getAllByRole('button')[1]!,
		);
		const hits = () => Array.from(document.querySelectorAll('[data-hit]'));
		await waitFor(() => expect(hits().length).toBeGreaterThan(0));
		// A text frame, if the spread has one.
		const text = hits().find((h) => /text/i.test(h.getAttribute('aria-label') ?? '')) ?? hits()[0]!;
		fireEvent.pointerDown(text, { button: 0, clientX: 10, clientY: 10 });
		fireEvent.pointerUp(text, { button: 0, clientX: 10, clientY: 10 });
		await userEvent.click(screen.getByRole('button', { name: 'Copy' }));
		const group = screen.getByRole('group', { name: 'Clipboard' });
		await userEvent.click(within(group).getByRole('button', { name: 'Paste special' }));
		const items = (await screen.findAllByRole('menuitem')).map((i) => i.textContent);
		expect(items).toContain('Paste text only');
		expect(items).toContain('Paste in place');
	});

	it('groups the frames selected with Ctrl+G and from the Layout tab’s Group button', async () => {
		const { books } = setup();
		await startBook();
		await userEvent.click(
			within(screen.getByRole('list', { name: 'Spreads' })).getAllByRole('button')[1]!,
		);
		const hits = () => Array.from(document.querySelectorAll('[data-hit]'));
		await waitFor(() => expect(hits().length).toBeGreaterThan(0));
		const page = document.querySelector('svg[role="group"]')!;
		// The picture and the words are free; the third frame carries a story and is left out.
		fireEvent.pointerDown(hits()[0]!, { button: 0, clientX: 10, clientY: 10 });
		fireEvent.pointerUp(hits()[0]!, { button: 0, clientX: 10, clientY: 10 });
		fireEvent.pointerDown(hits()[1]!, { button: 0, shiftKey: true, clientX: 10, clientY: 10 });
		fireEvent.pointerUp(hits()[1]!, { button: 0, shiftKey: true, clientX: 10, clientY: 10 });
		const grouped = () =>
			books.calls.filter((c) => c.startsWith('apply(') && c.includes('GroupFrames')).length;
		fireEvent.keyDown(page, { key: 'g', ctrlKey: true });
		await waitFor(() => expect(grouped()).toBe(1));
		await userEvent.click(within(ribbonTabs()).getByRole('tab', { name: 'Layout' }));
		await userEvent.click(screen.getByRole('button', { name: 'Group' }));
		await waitFor(() => expect(grouped()).toBe(2));
	});

	it('leaves copy, cut and paste to a field being typed in', async () => {
		setup();
		await startBook();
		render(<input aria-label="Name" />);
		const input = screen.getByLabelText('Name');
		expect(fireEvent.keyDown(input, { key: 'c', ctrlKey: true })).toBe(true);
		expect(fireEvent.keyDown(input, { key: 'x', ctrlKey: true })).toBe(true);
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

	it('asks about changes when the window is asked to close, and lets it go only when answered', async () => {
		const closing = closableWindow();
		const { books } = setup({ save: '/books/Pip/Pip' }, closing.requests);
		await startBook();
		const id = (await books.client.list())[0]!.id;
		await act(async () => {
			await books.client.apply(id, { type: 'MoveFrame' } as never);
		});

		let answer!: Promise<boolean>;
		act(() => {
			answer = closing.askToClose();
		});
		const dialog = await screen.findByRole('dialog', { name: /Save changes to/ });
		await userEvent.click(within(dialog).getByRole('button', { name: 'Cancel' }));
		expect(await answer).toBe(false);
		expect(screen.getByRole('toolbar', { name: 'Toolbar' })).toBeInTheDocument();

		act(() => {
			answer = closing.askToClose();
		});
		const again = await screen.findByRole('dialog', { name: /Save changes to/ });
		await userEvent.click(within(again).getByRole('button', { name: 'Don’t save' }));
		expect(await answer).toBe(true);
	});

	it('lets the window close at once when no book has changes', async () => {
		const closing = closableWindow();
		setup({}, closing.requests);
		await startBook();
		let answer!: Promise<boolean>;
		act(() => {
			answer = closing.askToClose();
		});
		expect(await answer).toBe(true);
		expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
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
		).toEqual(['Home', 'Insert', 'Pages', 'Layout', 'View', 'Tools', 'Export']);
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

/** A stand-in for the hand-off from Rust: paths waiting at start-up, and ones that arrive later. */
function openFilesFrom(waiting: string[]) {
	let wake: (() => void) | undefined;
	const report = vi.fn(async () => {});
	const client: OpenFilesClient = {
		take: async () => waiting.splice(0),
		onWaiting: async (listener) => {
			wake = listener;
			return () => {
				wake = undefined;
			};
		},
		report,
	};
	return {
		client,
		report,
		arrive: (...paths: string[]) => {
			waiting.push(...paths);
			act(() => wake?.());
		},
	};
}

describe('books the system asks Folio to open', () => {
	it('opens the book that was double-clicked on a cold start', async () => {
		const files = openFilesFrom(['C:\\My Books\\Naïve café.folio']);
		const { books } = setup({}, undefined, files.client);
		await screen.findByRole('toolbar', { name: 'Toolbar' });
		expect(books.calls).toContain('openBook("C:\\\\My Books\\\\Naïve café.folio")');
		expect(files.report).not.toHaveBeenCalled();
	});

	it('opens a book asked for while Folio is already running', async () => {
		const files = openFilesFrom([]);
		const { books } = setup({}, undefined, files.client);
		await screen.findByRole('heading', { name: 'What are you making?' });
		files.arrive('/home/me/second.folio');
		await screen.findByRole('toolbar', { name: 'Toolbar' });
		expect(books.calls).toContain('openBook("/home/me/second.folio")');
	});

	it('opens several books in the order asked', async () => {
		const files = openFilesFrom([]);
		const { books } = setup({}, undefined, files.client);
		await screen.findByRole('heading', { name: 'What are you making?' });
		files.arrive('/b/one.folio', '/b/two.folio');
		await waitFor(() =>
			expect(books.calls.filter((c) => c.startsWith('openBook'))).toHaveLength(2),
		);
		const opened = books.calls.filter((c) => c.startsWith('openBook'));
		expect(opened).toEqual(['openBook("/b/one.folio")', 'openBook("/b/two.folio")']);
	});

	it('says why a book could not be opened rather than staying quietly on the start page', async () => {
		const files = openFilesFrom(['/b/broken.folio']);
		const fake = createFakeBooksClient();
		fake.refuseNext('openBook', 'notFolio', 'that is not a Folio book');
		const settings = createFakeSettingsClient();
		render(
			<SettingsProvider client={settings}>
				<WindowChromeProvider controls={controls}>
					<BooksProvider
						client={fake.client}
						dialogs={{
							pickBook: async () => null,
							pickSavePath: async () => null,
							pickImage: async () => null,
							pickFolder: async () => null,
							pickText: async () => null,
						}}
					>
						<WindowsContext.Provider
							value={{ open: vi.fn(async () => {}), openBookTools: vi.fn(async () => {}) }}
						>
							<MainScreen client={info} openFilesClient={files.client} />
						</WindowsContext.Provider>
					</BooksProvider>
				</WindowChromeProvider>
			</SettingsProvider>,
		);
		await waitFor(() => expect(files.report).toHaveBeenCalledTimes(1));
		expect(files.report).toHaveBeenCalledWith(expect.stringContaining('/b/broken.folio'));
		expect(files.report).toHaveBeenCalledWith(expect.stringContaining('that is not a Folio book'));
	});
});

describe('find and replace', () => {
	const match = (from: number) => ({
		target: { type: 'frame' as const, frame: 'a' },
		para: 0,
		from,
		to: from + 3,
		spread: 'nowhere',
		context: 'a cat and a cat',
		locked: false,
	});

	it('opens with Ctrl+F, counts the matches, steps through them and closes with Escape', async () => {
		const { books } = setup();
		books.found.push(match(2), match(12));
		await startBook();
		expect(screen.queryByRole('search')).not.toBeInTheDocument();
		fireEvent.keyDown(window, { key: 'f', ctrlKey: true });
		const bar = await screen.findByRole('search', { name: 'Find in the book' });
		const field = within(bar).getByRole('textbox', { name: 'Find' });
		expect(field).toHaveFocus();
		await userEvent.type(field, 'cat');
		await within(bar).findByText('1 of 2');
		expect(books.calls.some((c) => c.startsWith('find('))).toBe(true);
		await userEvent.click(within(bar).getByRole('button', { name: 'Next match' }));
		expect(within(bar).getByText('2 of 2')).toBeInTheDocument();
		fireEvent.keyDown(field, { key: 'Enter', shiftKey: true });
		expect(within(bar).getByText('1 of 2')).toBeInTheDocument();
		fireEvent.keyDown(field, { key: 'Escape' });
		expect(screen.queryByRole('search')).not.toBeInTheDocument();
	});

	it('takes the page to the spread of the match it is on', async () => {
		const { books } = setup();
		await startBook();
		const [summary] = await books.client.list();
		const spreads = (await books.client.snapshot(summary!.id)).book.spreads;
		expect(spreads.length).toBeGreaterThan(2);
		const page = () =>
			[...document.querySelectorAll('li')].findIndex(
				(li) => li.querySelector('button[aria-current="true"]') !== null,
			);
		const before = page();
		books.found.push({ ...match(2), spread: spreads[2]!.id });
		fireEvent.keyDown(window, { key: 'f', ctrlKey: true });
		const bar = await screen.findByRole('search', { name: 'Find in the book' });
		await userEvent.type(within(bar).getByRole('textbox', { name: 'Find' }), 'cat');
		await within(bar).findByText('1 of 1');
		expect(before).toBeGreaterThanOrEqual(0);
		await waitFor(() => expect(page()).toBe(2));
	});

	it('opens the replace bar with Ctrl+H and replaces all the matches', async () => {
		const { books } = setup();
		books.found.push(match(2), match(12));
		await startBook();
		fireEvent.keyDown(window, { key: 'h', ctrlKey: true });
		const bar = await screen.findByRole('search', { name: 'Find and replace in the book' });
		await userEvent.type(within(bar).getByRole('textbox', { name: 'Find' }), 'cat');
		await within(bar).findByText('1 of 2');
		await userEvent.type(within(bar).getByRole('textbox', { name: 'Replace with' }), 'dog');
		await userEvent.click(within(bar).getByRole('button', { name: 'Replace all' }));
		expect(await within(bar).findByText('Replaced 2.')).toBeInTheDocument();
		expect(books.calls.some((c) => /replace\(.*"cat", "dog", "all"\)/.test(c))).toBe(true);
	});

	it('opens from the Find & replace button of the ribbon, and from the Edit menu command', async () => {
		setup();
		await startBook();
		await userEvent.click(await screen.findByRole('button', { name: /Find & replace/ }));
		expect(await screen.findByRole('search', { name: 'Find in the book' })).toBeInTheDocument();
	});

	it('shows the options only in Expert', async () => {
		setup();
		await startBook();
		fireEvent.keyDown(window, { key: 'f', ctrlKey: true });
		const bar = await screen.findByRole('search', { name: 'Find in the book' });
		expect(within(bar).queryByRole('button', { name: 'Options' })).not.toBeInTheDocument();
		const toolbar = screen.getByRole('group', { name: 'Toolbar' });
		await userEvent.click(within(toolbar).getByRole('radio', { name: 'Expert' }));
		await userEvent.click(await within(bar).findByRole('button', { name: 'Options' }));
		expect(within(bar).getByRole('checkbox', { name: 'Regular expression' })).toBeInTheDocument();
	});
});

describe('the ribbon acting on words selected in the on-page editor', () => {
	/** Opens the editor on the first text frame of Page 1 that has words, with Expert on. */
	const openEditor = async () => {
		const { books } = setup();
		await startBook();
		await userEvent.click(screen.getByRole('radio', { name: 'Expert' }));
		await userEvent.click(screen.getByRole('button', { name: /^Page 1/ }));
		await waitFor(() => expect(screen.getAllByRole('option').length).toBeGreaterThan(0));
		const bold = () => screen.getByRole('button', { name: 'Bold' }) as HTMLButtonElement;
		const frame = screen.getAllByRole('option').find((option) => {
			fireEvent.pointerDown(option, { button: 0 });
			fireEvent.pointerUp(option);
			return !bold().disabled;
		})!;
		fireEvent.doubleClick(frame);
		const surface = await waitFor(() => {
			const found = document.querySelector('[data-text-surface]');
			expect(found).not.toBeNull();
			return found as HTMLElement;
		});
		const sink = surface.querySelector('textarea')!;
		return { books, frame, surface, sink, bold };
	};

	/** A press on a ribbon control as WebKitGTK sends it: the blur it causes does not say where to. */
	const pressOnRibbon = (control: HTMLElement, sink: HTMLElement) => {
		fireEvent.pointerDown(control, { button: 0 });
		const kept = !fireEvent.mouseDown(control, { button: 0 });
		if (!kept) fireEvent.blur(sink, { relatedTarget: null });
		fireEvent.pointerUp(control, { button: 0 });
		fireEvent.mouseUp(control, { button: 0 });
		return kept;
	};

	it('keeps the frame selected and the controls live when words are picked in the editor', async () => {
		const { frame, surface, sink, bold } = await openEditor();
		// A press in the words lands on the editor's own box, over the page.
		fireEvent.pointerDown(surface, { button: 0 });
		fireEvent.mouseDown(surface, { button: 0 });
		fireEvent.keyDown(sink, { key: 'a', ctrlKey: true });
		expect(frame).toHaveAttribute('aria-selected', 'true');
		expect(bold()).toBeEnabled();
		expect(screen.getByRole('button', { name: 'Text colour' })).toBeEnabled();
		expect(screen.getByRole('button', { name: 'Copy' })).toBeEnabled();
		expect(screen.getByRole('button', { name: 'Cut' })).toBeEnabled();
	});

	it('styles the selected words from the ribbon, though the blur does not say where the focus went', async () => {
		const { books, surface, sink, bold } = await openEditor();
		fireEvent.pointerDown(surface, { button: 0 });
		fireEvent.keyDown(sink, { key: 'a', ctrlKey: true });
		const styled = () =>
			books.calls.filter((c) => c.includes('StyleRange') || c.includes('StyleText')).length;

		expect(pressOnRibbon(bold(), sink)).toBe(true);
		fireEvent.click(bold());
		await waitFor(() => expect(styled()).toBe(1));
		expect(document.querySelector('[data-text-surface]')).not.toBeNull();

		// The colour picker opens without taking the focus from the words, and a colour chosen in it
		// styles the selected words with the editor still open.
		const colour = screen.getByRole('button', { name: 'Text colour' });
		expect(pressOnRibbon(colour, sink)).toBe(true);
		fireEvent.click(colour);
		const picker = screen.getByRole('dialog');
		expect(picker).toHaveAttribute('data-keeps-editor');
		expect(document.querySelector('[data-text-surface]')).not.toBeNull();
		fireEvent.click(within(picker).getByRole('button', { name: 'Black, #000000' }));
		await waitFor(() => expect(styled()).toBe(2));
		expect(document.querySelector('[data-text-surface]')).not.toBeNull();

		// A press outside the editor and the ribbon still ends the editing.
		fireEvent.pointerDown(document.body, { button: 0 });
		fireEvent.blur(document.activeElement ?? sink, { relatedTarget: null });
		fireEvent.blur(sink, { relatedTarget: null });
		await waitFor(() => expect(document.querySelector('[data-text-surface]')).toBeNull());
	});

	it('copies, cuts and pastes the words with the Clipboard buttons, as the keys do', async () => {
		const { books, surface, sink } = await openEditor();
		fireEvent.pointerDown(surface, { button: 0 });
		fireEvent.keyDown(sink, { key: 'a', ctrlKey: true });
		// The webview sends the copy and paste events a button asks for, as it does for the keys.
		const held = new Map<string, string>();
		const clipboardData = {
			setData: (type: string, value: string) => held.set(type, value),
			getData: (type: string) => held.get(type) ?? '',
		};
		const exec = vi.fn((command: string) => {
			if (command === 'copy') fireEvent.copy(sink, { clipboardData });
			if (command === 'paste') fireEvent.paste(sink, { clipboardData });
			return false;
		});
		Object.defineProperty(document, 'execCommand', { value: exec, configurable: true });
		try {
			const group = screen.getByRole('group', { name: 'Clipboard' });
			pressOnRibbon(within(group).getByRole('button', { name: 'Copy' }), sink);
			fireEvent.click(within(group).getByRole('button', { name: 'Copy' }));
			await waitFor(() => expect(held.get('text/plain')).toBeTruthy());

			const edits = () => books.calls.filter((c) => c.startsWith('editText(')).length;
			pressOnRibbon(within(group).getByRole('button', { name: 'Paste' }), sink);
			fireEvent.click(within(group).getByRole('button', { name: 'Paste' }));
			await waitFor(() => expect(edits()).toBe(1));
			expect(
				screen.queryByText('There was nothing on the clipboard that could be pasted there.'),
			).not.toBeInTheDocument();
			// The words go back into the editor, not onto the page as a new frame.
			expect(books.calls.some((c) => c.includes('AddFrame'))).toBe(false);
		} finally {
			Reflect.deleteProperty(document, 'execCommand');
		}
	});
});

describe('the title bar quick actions', () => {
	const strip = () => screen.getByRole('toolbar', { name: 'Quick actions' });

	it('sits right after the app menu, with Save dimmed and New and Open ready when no book is open', async () => {
		setup();
		await screen.findByRole('heading', { name: 'What are you making?' });
		const menu = screen.getByRole('button', { name: 'Folio' });
		expect(menu.parentElement?.nextElementSibling).toBe(strip());
		expect(within(strip()).getByRole('button', { name: 'Save (Ctrl+S)' })).toHaveAttribute(
			'aria-disabled',
			'true',
		);
		expect(within(strip()).getByRole('button', { name: 'New book (Ctrl+N)' })).not.toHaveAttribute(
			'aria-disabled',
		);
		expect(within(strip()).getByRole('button', { name: 'Open (Ctrl+O)' })).not.toHaveAttribute(
			'aria-disabled',
		);
	});

	it('opens the New book wizard, as File → New does', async () => {
		const { windows } = setup();
		await userEvent.click(await screen.findByRole('button', { name: 'New book (Ctrl+N)' }));
		expect(windows.open).toHaveBeenCalledWith('WizardNew');
	});

	it('does nothing from a disabled Save', async () => {
		const { books } = setup({ save: '/books/Pip/Pip' });
		await screen.findByRole('heading', { name: 'What are you making?' });
		await userEvent.click(screen.getByRole('button', { name: 'Save (Ctrl+S)' }));
		expect(await books.client.list()).toHaveLength(0);
	});

	it('enables Save with a dot on a new book, and saves asking where, like File → Save', async () => {
		setup({ save: '/books/Pip/Pip' });
		await startBook();
		const save = within(strip()).getByRole('button', { name: 'Save, unsaved changes (Ctrl+S)' });
		expect(save).not.toHaveAttribute('aria-disabled');
		expect(save.querySelector('[class*="badge"]')).not.toBeNull();
		await userEvent.click(save);
		await waitFor(() =>
			expect(screen.getByRole('contentinfo', { name: 'Status' })).toHaveTextContent(
				'Saved · Pip.folio',
			),
		);
		const saved = within(strip()).getByRole('button', { name: 'Save (Ctrl+S)' });
		expect(saved.querySelector('[class*="badge"]')).toBeNull();
	});

	it('shows the dot again when a saved book changes', async () => {
		const { books } = setup({ save: '/books/Pip/Pip' });
		await startBook();
		await userEvent.click(within(strip()).getByRole('button', { name: /^Save/ }));
		await screen.findByText('Saved · Pip.folio');
		const id = (await books.client.list())[0]!.id;
		await act(async () => {
			await books.client.apply(id, { type: 'MoveFrame' } as never);
		});
		expect(
			await within(strip()).findByRole('button', { name: 'Save, unsaved changes (Ctrl+S)' }),
		).toBeInTheDocument();
	});

	it('is reached from the app menu button with Tab', async () => {
		setup();
		await screen.findByRole('heading', { name: 'What are you making?' });
		screen.getByRole('button', { name: 'Folio' }).focus();
		await userEvent.tab();
		// Save is dimmed but stays in the tab order, so the strip's one tab stop is its first button.
		expect(screen.getByRole('button', { name: 'Save (Ctrl+S)' })).toHaveFocus();
	});
});
