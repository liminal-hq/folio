// Tests for choosing where an export's files go
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it } from 'vitest';
import { ExportDialog } from '../app/ExportDialog';
import { createFakeBooksClient } from '../books/fakeBooksClient';
import { BooksProvider } from '../books/BooksProvider';
import type { FileDialogs } from '../books/dialogs';
import { createFakeSettingsClient } from '../settings/fakeSettingsClient';
import { SettingsProvider } from '../settings/SettingsProvider';

function dialogsPicking(folder: string | null): FileDialogs {
	return {
		pickBook: async () => null,
		pickSavePath: async () => null,
		pickImage: async () => null,
		pickText: async () => null,
		pickFolder: async () => folder,
	};
}

async function setup(picked: string | null) {
	const fake = createFakeBooksClient();
	const made = await fake.client.newBook({ title: 'Pip' } as never);
	const settings = createFakeSettingsClient();
	render(
		<SettingsProvider client={settings}>
			<BooksProvider client={fake.client} dialogs={dialogsPicking(picked)}>
				<ExportDialog
					open
					onClose={() => {}}
					client={fake.client}
					id={made.summary.id}
					autoStart={null}
					onCheck={() => {}}
				/>
			</BooksProvider>
		</SettingsProvider>,
	);
	return { fake, settings };
}

describe('the destination folder', () => {
	it('is the usual one to begin with', async () => {
		await setup(null);
		expect(await screen.findByText('an exports folder beside the book')).toBeInTheDocument();
		expect(screen.queryByRole('button', { name: 'Use the usual folder' })).not.toBeInTheDocument();
	});

	it('is chosen with the system’s folder picker and sent with the export', async () => {
		const { fake } = await setup('/home/mom/Share');
		await userEvent.click(await screen.findByRole('button', { name: 'Choose a folder…' }));
		expect(await screen.findByText('/home/mom/Share')).toBeInTheDocument();
		await userEvent.click(screen.getByRole('button', { name: 'Make files' }));
		await screen.findByText('Your files are ready.');
		const call = fake.calls.find((c) => c.startsWith('exportBook('))!;
		expect(call).toContain('"folder":"/home/mom/Share"');
	});

	it('goes back to the usual folder', async () => {
		await setup('/home/mom/Share');
		await userEvent.click(await screen.findByRole('button', { name: 'Choose a folder…' }));
		await userEvent.click(await screen.findByRole('button', { name: 'Use the usual folder' }));
		expect(await screen.findByText('an exports folder beside the book')).toBeInTheDocument();
	});

	it('becomes the person’s default folder with the other defaults', async () => {
		const { settings } = await setup('/home/mom/Share');
		await userEvent.click(await screen.findByRole('button', { name: 'Choose a folder…' }));
		await userEvent.click(screen.getByRole('radio', { name: /High/ }));
		await userEvent.click(screen.getByRole('button', { name: 'Make these my defaults' }));
		await waitFor(async () =>
			expect((await settings.get()).settings.export.folder).toBe('/home/mom/Share'),
		);
	});
});
