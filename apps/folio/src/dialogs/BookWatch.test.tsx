// Tests that a dialog window left open beside the book says so when the book changed, and closes with it
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { WindowChromeProvider } from '@liminal-hq/folio-chrome/WindowChromeProvider/WindowChromeProvider';
import { act, render, screen, waitFor } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { BookSizeDialogWindow } from '../app/BookSizeDialogWindow';
import { PageBackgroundDialogWindow } from '../app/PageBackgroundDialogWindow';
import { BooksProvider } from '../books/BooksProvider';
import { createFakeBooksClient } from '../books/fakeBooksClient';
import { createFakeSettingsClient } from '../settings/fakeSettingsClient';
import { SettingsProvider } from '../settings/SettingsProvider';
import { BookWatch, type DialogSession } from './DialogHost';
import { DialogWindowProvider } from './DialogSurface';

const controls = {
	minimize: vi.fn(),
	toggleMaximize: vi.fn(),
	close: vi.fn(),
	setAlwaysOnTop: vi.fn(),
	isMaximized: vi.fn().mockResolvedValue(false),
	onMaximizedChange: () => () => {},
};

async function setup(kind: 'BookSize' | 'PageBackground') {
	const fake = createFakeBooksClient();
	const made = await fake.client.newBook({ title: 'Pip' });
	const close = vi.fn();
	const session = { close, finish: vi.fn(), client: {} } as unknown as DialogSession;
	render(
		<SettingsProvider client={createFakeSettingsClient()}>
			<WindowChromeProvider controls={controls}>
				<DialogWindowProvider>
					<BooksProvider client={fake.client} dialogs={{} as never} pin={made.summary.id}>
						<BookWatch kind={kind} session={session}>
							{kind === 'BookSize' ? (
								<BookSizeDialogWindow session={session} />
							) : (
								<PageBackgroundDialogWindow
									payload={{
										bookId: made.summary.id,
										spread: made.book.spreads[0]!.id,
										page: null,
										scope: null,
									}}
									session={session}
								/>
							)}
						</BookWatch>
					</BooksProvider>
				</DialogWindowProvider>
			</WindowChromeProvider>
		</SettingsProvider>,
	);
	return { fake, made, close };
}

describe('a dialog window beside a book that goes on changing', () => {
	it('tells a form that keeps a draft that the book changed, and not before', async () => {
		const { fake, made } = await setup('BookSize');
		await screen.findByRole('dialog', { name: 'Book size' });
		expect(screen.queryByRole('status')).not.toBeInTheDocument();
		await act(async () => {
			await fake.client.apply(made.summary.id, {
				type: 'SetLanguage',
				language: 'fr-CA',
			} as never);
		});
		expect(await screen.findByRole('status')).toHaveTextContent(/book has changed/i);
	});

	it('does not say so for a dialog that changes the book as it goes', async () => {
		const { fake, made } = await setup('PageBackground');
		await screen.findByRole('dialog');
		await act(async () => {
			await fake.client.apply(made.summary.id, {
				type: 'SetLanguage',
				language: 'fr-CA',
			} as never);
		});
		expect(screen.queryByRole('status')).not.toBeInTheDocument();
	});

	it('closes when its book is closed', async () => {
		const { fake, made, close } = await setup('BookSize');
		await screen.findByRole('dialog', { name: 'Book size' });
		await act(async () => {
			await fake.client.closeBook(made.summary.id);
		});
		await waitFor(() => expect(close).toHaveBeenCalled());
	});
});
