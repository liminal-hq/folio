// Tests for the cover sheet preview
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import { createFakeBooksClient } from '../books/fakeBooksClient';
import { CoverSheetDialog } from './CoverSheetDialog';

describe('CoverSheetDialog', () => {
	it('draws the sheet the plugin makes', async () => {
		const fake = createFakeBooksClient();
		const made = await fake.client.newBook({});
		render(
			<CoverSheetDialog
				open
				onClose={() => undefined}
				client={fake.client}
				id={made.summary.id}
				revision={0}
				book={made.book}
				onSave={() => undefined}
				onPutDescription={() => undefined}
			/>,
		);
		expect(
			await screen.findByRole('img', { name: 'The paperback cover sheet' }),
		).toBeInTheDocument();
		expect(fake.calls).toContain(`previewCoverSheet("${made.summary.id}")`);
	});

	it('is not drawn while closed', async () => {
		const fake = createFakeBooksClient();
		const made = await fake.client.newBook({});
		render(
			<CoverSheetDialog
				open={false}
				onClose={() => undefined}
				client={fake.client}
				id={made.summary.id}
				revision={0}
				book={made.book}
				onSave={() => undefined}
				onPutDescription={() => undefined}
			/>,
		);
		expect(fake.calls.some((c) => c.startsWith('previewCoverSheet'))).toBe(false);
	});

	it('shows the guides over the sheet, and hides them when asked', async () => {
		const fake = createFakeBooksClient();
		const made = await fake.client.newBook({});
		render(
			<CoverSheetDialog
				open
				onClose={() => undefined}
				client={fake.client}
				id={made.summary.id}
				revision={0}
				book={made.book}
				onSave={() => undefined}
				onPutDescription={() => undefined}
			/>,
		);
		expect(await screen.findByTestId('cover-guides')).toBeInTheDocument();
		await userEvent.click(screen.getByRole('checkbox', { name: /Show the folds/ }));
		expect(screen.queryByTestId('cover-guides')).toBeNull();
	});

	it('keeps new words for the back cover, the spine and the front as the book details', async () => {
		const fake = createFakeBooksClient();
		const made = await fake.client.newBook({});
		const onSave = vi.fn();
		render(
			<CoverSheetDialog
				open
				onClose={() => undefined}
				client={fake.client}
				id={made.summary.id}
				revision={0}
				book={made.book}
				onSave={onSave}
				onPutDescription={() => undefined}
			/>,
		);
		const update = await screen.findByRole('button', { name: 'Update the cover' });
		expect(update).toBeDisabled();
		const blurb = screen.getByRole('textbox', { name: /^Description/ });
		await userEvent.clear(blurb);
		await userEvent.type(blurb, 'A cosy tale.');
		await userEvent.click(update);
		expect(onSave).toHaveBeenCalledTimes(1);
		expect(onSave.mock.calls[0]![0]).toMatchObject({ description: 'A cosy tale.' });
	});

	it('puts the description back on the back cover when the frame no longer holds it', async () => {
		const fake = createFakeBooksClient();
		const made = await fake.client.newBook({});
		const book = { ...made.book, metadata: { ...made.book.metadata, description: 'A cosy tale.' } };
		const put = vi.fn();
		const { rerender } = render(
			<CoverSheetDialog
				open
				onClose={() => undefined}
				client={fake.client}
				id={made.summary.id}
				revision={0}
				book={book}
				onSave={() => undefined}
				onPutDescription={put}
			/>,
		);
		await userEvent.click(await screen.findByRole('button', { name: /Put the description/ }));
		expect(put).toHaveBeenCalledWith('A cosy tale.');
		// Once the frame holds it, there is nothing to put back.
		const [cover, ...rest] = book.spreads;
		const frame = {
			type: 'text',
			id: 'blurb',
			box: { x: 0, y: 0, w: 10, h: 10 },
			content: { inline: [{ text: 'A cosy tale.' }] },
		};
		rerender(
			<CoverSheetDialog
				open
				onClose={() => undefined}
				client={fake.client}
				id={made.summary.id}
				revision={0}
				book={{ ...book, spreads: [{ ...cover!, frames: [frame as never] }, ...rest] }}
				onSave={() => undefined}
				onPutDescription={put}
			/>,
		);
		expect(screen.queryByRole('button', { name: /Put the description/ })).toBeNull();
	});
});
