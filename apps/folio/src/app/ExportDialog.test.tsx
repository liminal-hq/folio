// Tests for the Export dialog: choosing what to make, making it, and showing where it went
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { render, screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import { createFakeBooksClient } from '../books/fakeBooksClient';
import { ExportDialog } from './ExportDialog';

async function setup(autoStart: ('sharingPdf' | 'kdpInterior')[] | null = null) {
	const fake = createFakeBooksClient();
	const made = await fake.client.newBook({});
	const onCheck = vi.fn();
	const onClose = vi.fn();
	render(
		<ExportDialog
			open
			onClose={onClose}
			client={fake.client}
			id={made.summary.id}
			autoStart={autoStart}
			onCheck={onCheck}
		/>,
	);
	return { fake, onCheck, onClose };
}

const exported = (calls: string[]) => calls.filter((c) => c.startsWith('exportBook('));

describe('the Export dialog', () => {
	it('offers the kinds of file, with the sharing PDF chosen to begin with', async () => {
		await setup();
		expect(screen.getByRole('checkbox', { name: /Sharing PDF/ })).toBeChecked();
		expect(
			screen.getByRole('checkbox', { name: /Paperback interior for Amazon KDP/ }),
		).not.toBeChecked();
	});

	it('makes the chosen files and lists them with a way to show them', async () => {
		const { fake } = await setup();
		await userEvent.click(screen.getByRole('checkbox', { name: /Paperback interior/ }));
		await userEvent.click(screen.getByRole('button', { name: 'Make files' }));
		expect(await screen.findByText('Your files are ready.')).toBeInTheDocument();
		expect(exported(fake.calls)[0]).toContain('["sharingPdf","kdpInterior"]');
		const files = screen.getByRole('region', { name: 'The files that were made' });
		expect(within(files).getByText('pip-share.pdf')).toBeInTheDocument();
		expect(within(files).getByText('pip-paperback-interior.pdf')).toBeInTheDocument();
		await userEvent.click(within(files).getAllByRole('button', { name: 'Show in folder' })[0]!);
		expect(fake.calls).toContain('revealPath("/home/mom/Books/Pip/exports/pip-share.pdf")');
	});

	it('offers the Kindle and Kobo books and lists both Kobo files', async () => {
		await setup();
		await userEvent.click(screen.getByRole('checkbox', { name: /Kobo \(EPUB and KEPUB\)/ }));
		await userEvent.click(screen.getByRole('checkbox', { name: /Kindle \(EPUB\)/ }));
		await userEvent.click(screen.getByRole('button', { name: 'Make files' }));
		const files = await screen.findByRole('region', { name: 'The files that were made' });
		for (const name of ['pip-kindle.epub', 'pip.epub', 'pip.kepub.epub']) {
			expect(within(files).getByText(name)).toBeInTheDocument();
		}
	});

	it('cannot make nothing', async () => {
		await setup();
		await userEvent.click(screen.getByRole('checkbox', { name: /Sharing PDF/ }));
		expect(screen.getByRole('button', { name: 'Make files' })).toBeDisabled();
	});

	it('says what went wrong when the files cannot be made', async () => {
		const { fake } = await setup();
		fake.refuseNext('exportBook', 'export', 'the folder is full');
		await userEvent.click(screen.getByRole('button', { name: 'Make files' }));
		expect(await screen.findByRole('alert')).toHaveTextContent('the folder is full');
		expect(screen.getByRole('button', { name: 'Make files' })).toBeEnabled();
	});

	it('makes the sharing PDF at once for Quick PDF', async () => {
		const { fake } = await setup(['sharingPdf']);
		expect(await screen.findByText('Your files are ready.')).toBeInTheDocument();
		expect(exported(fake.calls)).toHaveLength(1);
	});

	it('says how many things wait on the person and offers to check the book first', async () => {
		const fake = createFakeBooksClient();
		fake.preflightItems.push(
			{ kind: 'oversetText', severity: 'error', spread: 's', frame: 'f' },
			{ kind: 'fontMissing', severity: 'warning', spread: 's', frame: 'f', family: 'Zed' },
		);
		const made = await fake.client.newBook({});
		const onCheck = vi.fn();
		const onClose = vi.fn();
		render(
			<ExportDialog
				open
				onClose={onClose}
				client={fake.client}
				id={made.summary.id}
				onCheck={onCheck}
			/>,
		);
		expect(await screen.findByText(/One thing is waiting on you/)).toBeInTheDocument();
		await userEvent.click(screen.getByRole('button', { name: 'Check the book first' }));
		expect(onCheck).toHaveBeenCalled();
		expect(onClose).toHaveBeenCalled();
	});
});
