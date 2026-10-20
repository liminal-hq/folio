// Tests for the Swatches panel
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { Book } from '@liminal-hq/folio-protocol/generated/Book';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import { SwatchesPanel } from './SwatchesPanel';

const book = (swatches: unknown[] = []) =>
	({ styles: { look: '', fontFamily: '', styles: {}, swatches } }) as unknown as Book;

describe('SwatchesPanel', () => {
	it('says there are none, and adds one from a name and four percents', async () => {
		const apply = vi.fn();
		render(<SwatchesPanel book={book()} apply={apply} />);
		expect(screen.getByText('This book has no swatches yet.')).toBeInTheDocument();
		const add = screen.getByRole('button', { name: 'Add swatch' });
		expect(add).toBeDisabled();
		await userEvent.type(screen.getByLabelText('Name'), 'Brand red');
		await userEvent.clear(screen.getByLabelText('M %'));
		await userEvent.type(screen.getByLabelText('M %'), '100');
		await userEvent.clear(screen.getByLabelText('Y %'));
		await userEvent.type(screen.getByLabelText('Y %'), '90');
		await userEvent.click(add);
		expect(apply).toHaveBeenCalledWith({
			type: 'SetSwatch',
			swatch: {
				id: 'brand-red',
				name: 'Brand red',
				display: '#ff0019',
				ink: { c: 0, m: 100, y: 90, k: 0 },
			},
		});
	});

	it('will not add a plate that is not a percent', async () => {
		render(<SwatchesPanel book={book()} apply={vi.fn()} />);
		await userEvent.type(screen.getByLabelText('Name'), 'Odd');
		await userEvent.clear(screen.getByLabelText('K %'));
		await userEvent.type(screen.getByLabelText('K %'), '120');
		expect(screen.getByRole('button', { name: 'Add swatch' })).toBeDisabled();
	});

	it('lists the swatches with their ink, changes one by id and removes one', async () => {
		const apply = vi.fn();
		render(
			<SwatchesPanel
				book={book([
					{ id: 'brand', name: 'Brand', display: '#e60017', ink: { c: 0, m: 100, y: 100, k: 0 } },
					{ id: 'sea', name: 'Sea', display: '#336699' },
				])}
				apply={apply}
			/>,
		);
		expect(screen.getByText('C0 M100 Y100 K0')).toBeInTheDocument();
		expect(screen.getByText('#336699')).toBeInTheDocument();
		await userEvent.click(screen.getByRole('button', { name: 'Change Brand' }));
		await userEvent.clear(screen.getByLabelText('K %'));
		await userEvent.type(screen.getByLabelText('K %'), '10');
		await userEvent.click(screen.getByRole('button', { name: 'Save swatch' }));
		expect(apply).toHaveBeenLastCalledWith({
			type: 'SetSwatch',
			swatch: expect.objectContaining({ id: 'brand', ink: { c: 0, m: 100, y: 100, k: 10 } }),
		});
		await userEvent.click(screen.getByRole('button', { name: 'Remove Sea' }));
		expect(apply).toHaveBeenLastCalledWith({ type: 'RemoveSwatch', id: 'sea' });
	});
});
