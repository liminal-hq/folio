// Tests for the Check book dialog: the issues in groups, the way to each, and the clear message
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { Book } from '@liminal-hq/folio-protocol/generated/Book';
import type { PreflightItem } from '@liminal-hq/folio-protocol/generated/PreflightItem';
import { render, screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import { createFakeBooksClient } from '../books/fakeBooksClient';
import { CheckBookDialog } from './CheckBookDialog';

const book = { spreads: [{ id: 'sp_cover' }, { id: 'sp_1' }] } as unknown as Book;
const labels = ['Cover', 'Pages 1–2'];

const item = (extra: Partial<PreflightItem>): PreflightItem => ({
	kind: 'oversetText',
	severity: 'error',
	spread: 'sp_1',
	frame: 'fr_a',
	...extra,
});

async function setup(items: PreflightItem[]) {
	const fake = createFakeBooksClient();
	const made = await fake.client.newBook({});
	fake.preflightItems.push(...items);
	const onShow = vi.fn();
	const onClose = vi.fn();
	const onIgnore = vi.fn();
	render(
		<CheckBookDialog
			open
			onClose={onClose}
			client={fake.client}
			id={made.summary.id}
			book={book}
			labels={labels}
			onShow={onShow}
			onIgnore={onIgnore}
		/>,
	);
	return { fake, onShow, onClose, onIgnore };
}

describe('the Check book dialog', () => {
	it('says the book looks ready when there is nothing to fix', async () => {
		await setup([]);
		expect(
			await screen.findByText('No issues found. Book looks ready to export.'),
		).toBeInTheDocument();
	});

	it('groups issues as waiting on the person, worth a look and for information, worst first', async () => {
		await setup([
			item({}),
			item({ kind: 'fontMissing', severity: 'warning', family: 'Zed Sans' }),
			item({ kind: 'altMissing', severity: 'note', frame: 'fr_b' }),
		]);
		expect(await screen.findByRole('heading', { name: /Waiting on you/ })).toBeInTheDocument();
		const headings = screen.getAllByRole('heading', { level: 3 }).map((h) => h.textContent);
		expect(headings).toEqual([
			expect.stringContaining('Waiting on you'),
			expect.stringContaining('Worth a look'),
			expect.stringContaining('For your information'),
		]);
		expect(screen.getByText('Some words do not fit on Pages 1–2')).toBeInTheDocument();
		expect(screen.getByText('Zed Sans is not on this computer or in the book')).toBeInTheDocument();
		expect(screen.getByText('A picture on Pages 1–2 has no description')).toBeInTheDocument();
	});

	it('goes to the page and frame of an issue and closes', async () => {
		const { onShow, onClose } = await setup([item({})]);
		const row = (await screen.findByText('Some words do not fit on Pages 1–2')).closest('li')!;
		await userEvent.click(within(row).getByRole('button', { name: 'Show me' }));
		expect(onShow).toHaveBeenCalledWith(1, 'fr_a');
		expect(onClose).toHaveBeenCalled();
	});

	it('says several empty picture frames once, with the count, and goes to the first', async () => {
		const { onShow } = await setup([
			item({ kind: 'picturePlaceholder', frame: 'fr_1', spread: 'sp_cover' }),
			item({ kind: 'picturePlaceholder', frame: 'fr_2' }),
			item({ kind: 'picturePlaceholder', frame: 'fr_3' }),
		]);
		const row = (await screen.findByText('3 picture frames are still empty')).closest('li')!;
		expect(screen.getAllByRole('listitem')).toHaveLength(1);
		expect(screen.getByRole('heading', { level: 3 })).toHaveTextContent(/Waiting on you\s*3/);
		await userEvent.click(within(row).getByRole('button', { name: 'Show me' }));
		expect(onShow).toHaveBeenCalledWith(0, 'fr_1');
	});

	it('leaves an issue alone by its key, and brings an ignored one back', async () => {
		const { fake, onIgnore } = await setup([
			item({ kind: 'fontMissing', severity: 'warning', family: 'Zed Sans', key: 'k-font' }),
			item({ kind: 'oversetText', key: 'k-over', ignored: true }),
		]);
		void fake;
		const row = (
			await screen.findByText('Zed Sans is not on this computer or in the book')
		).closest('li')!;
		await userEvent.click(within(row).getByRole('button', { name: 'Ignore' }));
		expect(onIgnore).toHaveBeenCalledWith(['k-font']);
		const quiet = screen.getByText('Some words do not fit on Pages 1–2').closest('li')!;
		await userEvent.click(within(quiet).getByRole('button', { name: 'Stop ignoring' }));
		expect(onIgnore).toHaveBeenLastCalledWith([]);
		expect(screen.getByRole('heading', { name: /Left alone/ })).toBeInTheDocument();
	});

	it('checks again when asked', async () => {
		const { fake } = await setup([]);
		await screen.findByText('No issues found. Book looks ready to export.');
		fake.preflightItems.push(item({ kind: 'picturePlaceholder', frame: 'fr_c' }));
		await userEvent.click(screen.getByRole('button', { name: 'Check again' }));
		expect(
			await screen.findByText('A picture frame on Pages 1–2 is still empty'),
		).toBeInTheDocument();
		expect(fake.calls.filter((c) => c.startsWith('preflightBook(')).length).toBe(2);
	});
});
