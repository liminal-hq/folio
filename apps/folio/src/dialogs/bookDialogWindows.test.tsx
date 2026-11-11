// Tests the book dialogs as dialog windows: each works on the book it was opened for and closes after saving
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { WindowChromeProvider } from '@liminal-hq/folio-chrome/WindowChromeProvider/WindowChromeProvider';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import type { ReactNode } from 'react';
import { describe, expect, it, vi } from 'vitest';
import { BarcodeDialogWindow } from '../app/BarcodeDialogWindow';
import { BookDetailsDialogWindow } from '../app/BookDetailsDialogWindow';
import { BookSizeDialogWindow } from '../app/BookSizeDialogWindow';
import { MarginsDialogWindow } from '../app/MarginsDialogWindow';
import { PageBackgroundDialogWindow } from '../app/PageBackgroundDialogWindow';
import { BooksProvider } from '../books/BooksProvider';
import { createFakeBooksClient } from '../books/fakeBooksClient';
import { createFakeSettingsClient } from '../settings/fakeSettingsClient';
import { SettingsProvider } from '../settings/SettingsProvider';
import type { DialogSession } from './DialogHost';
import { DialogWindowProvider } from './DialogSurface';

const controls = {
	minimize: vi.fn(),
	toggleMaximize: vi.fn(),
	close: vi.fn(),
	setAlwaysOnTop: vi.fn(),
	isMaximized: vi.fn().mockResolvedValue(false),
	onMaximizedChange: () => () => {},
};

async function setup(host: (props: { session: DialogSession }) => ReactNode) {
	const fake = createFakeBooksClient();
	const made = await fake.client.newBook({ title: 'Pip' });
	const close = vi.fn();
	const session = { close, finish: vi.fn(), client: {} } as unknown as DialogSession;
	render(
		<SettingsProvider client={createFakeSettingsClient()}>
			<WindowChromeProvider controls={controls}>
				<DialogWindowProvider>
					<BooksProvider client={fake.client} dialogs={{} as never} pin={made.summary.id}>
						{host({ session })}
					</BooksProvider>
				</DialogWindowProvider>
			</WindowChromeProvider>
		</SettingsProvider>,
	);
	return { fake, close };
}

describe('the book dialogs in their own windows', () => {
	it('shows Book details for the pinned book and closes once the change is made', async () => {
		const { fake, close } = await setup((p) => <BookDetailsDialogWindow {...p} />);
		const title = await screen.findByRole('textbox', { name: 'Title' });
		expect(title).toHaveValue('Pip');
		await userEvent.clear(title);
		await userEvent.type(title, 'Pip and Pop');
		await userEvent.click(screen.getByRole('button', { name: 'Save' }));
		await waitFor(() => expect(close).toHaveBeenCalled());
		expect(fake.calls.some((c) => c.startsWith('apply(') && c.includes('SetDetails'))).toBe(true);
	});

	it.each([
		['Barcode', (p: { session: DialogSession }) => <BarcodeDialogWindow {...p} />],
		['Book size', (p: { session: DialogSession }) => <BookSizeDialogWindow {...p} />],
		['Margins', (p: { session: DialogSession }) => <MarginsDialogWindow {...p} />],
	])('shows the %s dialog and closes on Esc', async (name, host) => {
		const { close } = await setup(host);
		expect(await screen.findByRole('dialog', { name })).toBeInTheDocument();
		// A length field keeps Esc for undoing its own edit, so press it from the buttons.
		screen.getByRole('button', { name: 'Cancel' }).focus();
		await userEvent.keyboard('{Escape}');
		await waitFor(() => expect(close).toHaveBeenCalled());
	});

	it('rewinds the book and closes when the page background is cancelled', async () => {
		const fake = createFakeBooksClient();
		const made = await fake.client.newBook({ title: 'Pip' });
		const spread = made.book.spreads[0]!.id;
		const close = vi.fn();
		const session = { close, finish: vi.fn(), client: {} } as unknown as DialogSession;
		render(
			<SettingsProvider client={createFakeSettingsClient()}>
				<WindowChromeProvider controls={controls}>
					<DialogWindowProvider>
						<BooksProvider client={fake.client} dialogs={{} as never} pin={made.summary.id}>
							<PageBackgroundDialogWindow
								payload={{ bookId: made.summary.id, spread, page: null, scope: null }}
								session={session}
							/>
						</BooksProvider>
					</DialogWindowProvider>
				</WindowChromeProvider>
			</SettingsProvider>,
		);
		expect(await screen.findByRole('dialog', { name: 'Page background' })).toBeInTheDocument();
		await userEvent.click(screen.getByRole('button', { name: 'Cancel' }));
		await waitFor(() => expect(close).toHaveBeenCalled());
		expect(fake.calls.some((c) => c.startsWith('historyMark'))).toBe(true);
	});
});
