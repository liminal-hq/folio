// Tests for the picture quality and size in the Export dialog, with a fake BooksClient
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { act, render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { createFakeBooksClient } from '../books/fakeBooksClient';
import { ExportDialog } from '../app/ExportDialog';
import { createFakeSettingsClient } from '../settings/fakeSettingsClient';
import { SettingsProvider } from '../settings/SettingsProvider';
import { ESTIMATE_PAUSE_MS } from './SizeEstimate';

afterEach(() => vi.useRealTimers());

async function setup(withSettings = false) {
	const fake = createFakeBooksClient();
	const made = await fake.client.newBook({});
	const settings = createFakeSettingsClient();
	const dialog = (
		<ExportDialog
			open
			onClose={() => {}}
			client={fake.client}
			id={made.summary.id}
			autoStart={null}
			onCheck={() => {}}
		/>
	);
	render(withSettings ? <SettingsProvider client={settings}>{dialog}</SettingsProvider> : dialog);
	return { fake, settings, id: made.summary.id };
}

const estimates = (calls: string[]) => calls.filter((c) => c.startsWith('estimateExport('));
const applied = (calls: string[]) => calls.filter((c) => c.includes('SetExportPrefs'));

describe('the picture quality choice', () => {
	it('offers four qualities for the sharing PDF and starts on Standard', async () => {
		await setup();
		expect(await screen.findByRole('radio', { name: /Standard/ })).toBeChecked();
		for (const name of [/Small/, /Standard/, /High/, /Original/]) {
			expect(screen.getByRole('radio', { name })).toBeInTheDocument();
		}
		expect(screen.getByText('For email. Pictures are made smaller.')).toBeInTheDocument();
	});

	it('offers a paperback only the qualities that keep it printable', async () => {
		await setup();
		await userEvent.click(screen.getByRole('checkbox', { name: /Paperback interior/ }));
		await userEvent.click(screen.getByRole('checkbox', { name: /Sharing PDF/ }));
		expect(screen.getByRole('radio', { name: /High/ })).toBeChecked();
		expect(screen.queryByRole('radio', { name: /Small/ })).not.toBeInTheDocument();
	});

	it('sends the choice with the export and keeps it in the book', async () => {
		const { fake } = await setup();
		await userEvent.click(await screen.findByRole('radio', { name: /Small/ }));
		await userEvent.click(screen.getByRole('button', { name: 'Make files' }));
		await screen.findByText('Your files are ready.');
		const call = fake.calls.find((c) => c.startsWith('exportBook('))!;
		expect(call).toContain('"preset":"small"');
		await waitFor(() => expect(applied(fake.calls)).toHaveLength(1));
	});

	it('saves what was changed as the person’s defaults when they ask', async () => {
		const { settings } = await setup(true);
		await userEvent.click(await screen.findByRole('radio', { name: /High/ }));
		await userEvent.click(screen.getByRole('button', { name: 'Make these my defaults' }));
		await waitFor(async () =>
			expect((await settings.get()).settings.export.pdf.sharing.preset).toBe('high'),
		);
	});
});

describe('the size estimate', () => {
	it('says about how big the file will be after a pause', async () => {
		vi.useFakeTimers({ shouldAdvanceTime: true });
		const { fake } = await setup();
		expect(screen.getByText('Working out the size…')).toBeInTheDocument();
		expect(estimates(fake.calls)).toHaveLength(0);
		await act(async () => {
			await vi.advanceTimersByTimeAsync(ESTIMATE_PAUSE_MS + 20);
		});
		expect(await screen.findByText('About 3.8 MB')).toBeInTheDocument();
		expect(estimates(fake.calls)).toHaveLength(1);
	});

	it('waits for the choosing to pause before asking again', async () => {
		vi.useFakeTimers({ shouldAdvanceTime: true });
		const { fake } = await setup();
		await act(async () => {
			await vi.advanceTimersByTimeAsync(ESTIMATE_PAUSE_MS + 20);
		});
		const before = estimates(fake.calls).length;
		await userEvent.click(screen.getByRole('radio', { name: /Small/ }));
		await userEvent.click(screen.getByRole('radio', { name: /High/ }));
		expect(estimates(fake.calls)).toHaveLength(before);
		await act(async () => {
			await vi.advanceTimersByTimeAsync(ESTIMATE_PAUSE_MS + 20);
		});
		expect(estimates(fake.calls)).toHaveLength(before + 1);
	});
});
