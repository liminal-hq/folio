// Tests for the Spread templates dialog
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import { createFakeBooksClient } from '../books/fakeBooksClient';
import { SpreadTemplatesDialog } from './SpreadTemplatesDialog';

async function open(onChoose = vi.fn().mockResolvedValue(undefined)) {
	const fake = createFakeBooksClient();
	const made = await fake.client.newBook({});
	render(
		<SpreadTemplatesDialog
			open
			onClose={() => undefined}
			client={fake.client}
			id={made.summary.id}
			onChoose={onChoose}
		/>,
	);
	return { fake, made, onChoose };
}

describe('SpreadTemplatesDialog', () => {
	it('draws the templates the plugin makes and opens on the book’s own group', async () => {
		const { fake, made } = await open();
		expect(await screen.findByRole('button', { name: /Full-bleed spread/ })).toBeInTheDocument();
		expect(fake.calls).toContain(`templateGallery("${made.summary.id}")`);
		expect(screen.getByRole('button', { name: 'Picture book' })).toHaveAttribute(
			'aria-pressed',
			'true',
		);
		// Another group's templates are not shown until it is chosen.
		expect(screen.queryByRole('button', { name: /Lesson and worksheet/ })).toBeNull();
		await userEvent.click(screen.getByRole('button', { name: 'Textbook' }));
		expect(screen.getByRole('button', { name: /Lesson and worksheet/ })).toBeInTheDocument();
		await userEvent.click(screen.getByRole('button', { name: 'All' }));
		expect(screen.getByRole('button', { name: /Poem, centred/ })).toBeInTheDocument();
	});

	it('hands the chosen template over once', async () => {
		const { onChoose } = await open();
		await userEvent.click(await screen.findByRole('button', { name: /Picture and words/ }));
		expect(onChoose).toHaveBeenCalledTimes(1);
		expect(onChoose).toHaveBeenCalledWith('pb-split', 'Picture and words');
	});

	it('says why when the template could not be added', async () => {
		await open(vi.fn().mockRejectedValue(new Error('No spread after the cover.')));
		await userEvent.click(await screen.findByRole('button', { name: /Full-bleed spread/ }));
		expect(await screen.findByRole('alert')).toHaveTextContent('No spread after the cover.');
	});
});
