// Tests for the barcode dialog
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import { createFakeBooksClient } from '../books/fakeBooksClient';
import { BarcodeDialog } from './BarcodeDialog';

async function setup() {
	const fake = createFakeBooksClient();
	const made = await fake.client.newBook({});
	const apply = vi.fn();
	render(<BarcodeDialog open onClose={() => undefined} book={made.book} apply={apply} />);
	return { apply };
}

describe('BarcodeDialog', () => {
	it('starts automatic and saves leaving it to KDP', async () => {
		const { apply } = await setup();
		expect(screen.getByRole('radio', { name: /Automatic/ })).toBeChecked();
		await userEvent.click(screen.getByRole('radio', { name: /Let KDP add it/ }));
		await userEvent.click(screen.getByRole('button', { name: 'Save' }));
		expect(apply).toHaveBeenCalledWith({ type: 'SetBarcode', choice: { kind: 'kdp' } });
	});

	it('makes its own from an ISBN-10, with a US price add-on, saving the ISBN as thirteen digits', async () => {
		const { apply } = await setup();
		await userEvent.click(screen.getByRole('radio', { name: /Make my own/ }));
		await userEvent.type(screen.getByLabelText(/Paperback ISBN/), '0-306-40615-2');
		expect(screen.getByRole('status')).toHaveTextContent('9780306406157');
		await userEvent.selectOptions(screen.getByLabelText('Price add-on'), 'usd');
		await userEvent.type(screen.getByLabelText(/^Price \(for example/), '19.99');
		await userEvent.click(screen.getByRole('button', { name: 'Save' }));
		expect(apply).toHaveBeenCalledWith(
			expect.objectContaining({
				type: 'SetDetails',
				details: expect.objectContaining({
					isbn: expect.objectContaining({ paperback: '9780306406157' }),
				}),
			}),
		);
		expect(apply).toHaveBeenLastCalledWith({
			type: 'SetBarcode',
			choice: { kind: 'own', addon: '51999' },
		});
	});

	it('offers the right last digit and will not save a bad ISBN', async () => {
		await setup();
		await userEvent.click(screen.getByRole('radio', { name: /Make my own/ }));
		await userEvent.type(screen.getByLabelText(/Paperback ISBN/), '9780306406158');
		expect(screen.getByRole('alert')).toHaveTextContent('last digit');
		expect(screen.getByRole('button', { name: 'Save' })).toBeDisabled();
		await userEvent.click(screen.getByRole('button', { name: 'Use 9780306406157' }));
		expect(screen.getByRole('button', { name: 'Save' })).toBeEnabled();
	});
});
