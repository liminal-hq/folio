// Tests for the Expert panels of the Export dialog: what each controls and what a target locks
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it } from 'vitest';
import { ExportDialog } from '../app/ExportDialog';
import { createFakeBooksClient } from '../books/fakeBooksClient';
import { DEFAULT_SETTINGS, createFakeSettingsClient } from '../settings/fakeSettingsClient';
import { SettingsProvider } from '../settings/SettingsProvider';
import { lockedBecause } from './exportPrefs';

async function setup(mode: 'easy' | 'expert' = 'expert') {
	const fake = createFakeBooksClient();
	const made = await fake.client.newBook({});
	const settings = createFakeSettingsClient({ ...DEFAULT_SETTINGS, mode });
	render(
		<SettingsProvider client={settings}>
			<ExportDialog
				open
				onClose={() => {}}
				client={fake.client}
				id={made.summary.id}
				autoStart={null}
				onCheck={() => {}}
			/>
		</SettingsProvider>,
	);
	return fake;
}

describe('the Expert panels', () => {
	it('are not there in Easy mode', async () => {
		await setup('easy');
		expect(await screen.findByRole('radio', { name: /Standard/ })).toBeInTheDocument();
		expect(screen.queryByText('More choices')).not.toBeInTheDocument();
	});

	it('offer pictures, navigation, pages and print extras in Expert mode', async () => {
		await setup();
		await userEvent.click(await screen.findByText('More choices'));
		for (const name of [
			'Pixels an inch (at most)',
			'JPEG quality',
			'Compression',
			'Bookmarks',
			'Page numbers as the book shows them',
			'Subject',
			'Each sheet holds',
			'Cover',
			'Crop marks',
		]) {
			expect(screen.getByLabelText(name)).toBeInTheDocument();
		}
		// The sharing PDF has chapter bookmarks to begin with.
		expect(screen.getByLabelText('Bookmarks')).toHaveValue('chapters');
	});

	it('keeps a choice for the export', async () => {
		const fake = await setup();
		await userEvent.click(await screen.findByText('More choices'));
		await userEvent.selectOptions(screen.getByLabelText('Bookmarks'), 'pages');
		await userEvent.type(screen.getByLabelText('JPEG quality'), '70');
		await userEvent.click(screen.getByRole('button', { name: 'Make files' }));
		await screen.findByText('Your files are ready.');
		const call = fake.calls.find((c) => c.startsWith('exportBook('))!;
		expect(call).toContain('"bookmarks":"pages"');
		expect(call).toContain('"jpegQuality":70');
	});

	it('has links on for a Sharing PDF and lets them be turned off', async () => {
		const fake = await setup();
		await userEvent.click(await screen.findByText('More choices'));
		const links = screen.getByLabelText('Links');
		expect(links).toBeChecked();
		await userEvent.click(links);
		await userEvent.click(screen.getByRole('button', { name: 'Make files' }));
		await screen.findByText('Your files are ready.');
		const call = fake.calls.find((c) => c.startsWith('exportBook('))!;
		expect(call).toContain('"links":false');
	});

	it('grey out what a paperback is fixed at and say why', async () => {
		await setup();
		await userEvent.click(screen.getByRole('checkbox', { name: /Paperback interior/ }));
		await userEvent.click(screen.getByRole('checkbox', { name: /Sharing PDF/ }));
		await userEvent.click(await screen.findByText('More choices'));
		const bookmarks = screen.getByLabelText('Bookmarks');
		expect(bookmarks).toBeDisabled();
		expect(bookmarks).toHaveAccessibleDescription(
			'A paperback file carries only the title and author.',
		);
		expect(screen.getByLabelText('Cover')).toBeDisabled();
		expect(screen.getByLabelText('Crop marks')).toBeDisabled();
		await waitFor(() => expect(screen.getByLabelText('Pixels an inch (at most)')).toBeEnabled());
	});
});

describe('what each target locks', () => {
	it('matches the table in the spec', () => {
		expect(lockedBecause('sharing', 'bookmarks')).toBeNull();
		expect(lockedBecause('kdp', 'bookmarks')).not.toBeNull();
		expect(lockedBecause('kdp', 'range')).not.toBeNull();
		expect(lockedBecause('kdp', 'cropMarks')).not.toBeNull();
		expect(lockedBecause('printShop', 'bookmarks')).toBeNull();
		expect(lockedBecause('printShop', 'cropMarks')).toBeNull();
		expect(lockedBecause('printShop', 'links')).not.toBeNull();
		expect(lockedBecause('printShop', 'sheets')).not.toBeNull();
		expect(lockedBecause('sharing', 'cropMarks')).not.toBeNull();
	});
});
