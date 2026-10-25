// Tests for the Book details dialog
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { Details } from '@liminal-hq/folio-protocol/generated/Details';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import { createFakeBooksClient } from '../books/fakeBooksClient';
import { BookDetailsDialog } from './BookDetailsDialog';

async function setup() {
	const fake = createFakeBooksClient();
	const made = await fake.client.newBook({});
	const onSave = vi.fn<(details: Details) => void>();
	const onClose = vi.fn();
	const onPaper = vi.fn();
	render(
		<BookDetailsDialog open onClose={onClose} book={made.book} onSave={onSave} onPaper={onPaper} />,
	);
	return { onSave, onClose, onPaper };
}

describe('BookDetailsDialog', () => {
	it('starts from the book and saves what was changed as one set of details', async () => {
		const { onSave, onClose } = await setup();
		const title = screen.getByLabelText(/^Title/);
		await userEvent.clear(title);
		await userEvent.type(title, 'Tea at the Lighthouse');
		await userEvent.type(screen.getByLabelText(/Paperback ISBN/), '978-0-306-40615-7');
		await userEvent.click(screen.getByRole('button', { name: 'Save' }));
		expect(onSave).toHaveBeenCalledOnce();
		const details = onSave.mock.calls[0]![0];
		expect(details.title).toBe('Tea at the Lighthouse');
		expect(details.isbn.paperback).toBe('978-0-306-40615-7');
		expect(details.isbn.hardcover).toBeNull();
		expect(onClose).toHaveBeenCalled();
	});

	it('flags a bad ISBN as it is typed and will not save it, or an empty title', async () => {
		const { onSave } = await setup();
		await userEvent.type(screen.getByLabelText(/Paperback ISBN/), '978-0-306-40615-8');
		expect(screen.getByRole('alert')).toHaveTextContent('not a valid ISBN-13');
		expect(screen.getByRole('button', { name: 'Save' })).toBeDisabled();
		await userEvent.clear(screen.getByLabelText(/Paperback ISBN/));
		await userEvent.clear(screen.getByLabelText(/^Title/));
		expect(screen.getByRole('button', { name: 'Save' })).toBeDisabled();
		expect(onSave).not.toHaveBeenCalled();
	});

	it('sets the paper as its own step, and only when it changed', async () => {
		const { onPaper } = await setup();
		await userEvent.click(screen.getByRole('button', { name: 'Save' }));
		expect(onPaper).not.toHaveBeenCalled();
	});

	it('passes a changed paper on', async () => {
		const { onPaper } = await setup();
		await userEvent.selectOptions(screen.getByRole('combobox', { name: /^Paper/ }), 'cream');
		await userEvent.click(screen.getByRole('button', { name: 'Save' }));
		expect(onPaper).toHaveBeenCalledWith('cream');
	});
});
