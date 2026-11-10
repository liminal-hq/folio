// Tests for the book size and margins dialogs
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { Book } from '@liminal-hq/folio-protocol/generated/Book';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import { BookSizeDialog } from './BookSizeDialog';
import { MarginsDialog } from './MarginsDialog';

const book = {
	trim: { w: 432, h: 648 },
	margins: { top: 54, bottom: 54, inside: 54, outside: 36 },
} as unknown as Book;

const type = async (label: RegExp, text: string) => {
	const field = screen.getByLabelText(label);
	await userEvent.clear(field);
	await userEvent.type(field, `${text}{Enter}`);
};

describe('BookSizeDialog', () => {
	it('starts from the trim and asks for one SetBookGeometry with the new size', async () => {
		const apply = vi.fn();
		const onClose = vi.fn();
		render(<BookSizeDialog open onClose={onClose} book={book} units="inches" apply={apply} />);
		expect(screen.getByLabelText(/Width/)).toHaveValue('6');
		expect(screen.getByLabelText(/Height/)).toHaveValue('9');
		await type(/Width/, '5.5');
		await type(/Height/, '8.5');
		await userEvent.click(screen.getByRole('button', { name: 'Change size' }));
		expect(apply).toHaveBeenCalledTimes(1);
		expect(apply).toHaveBeenCalledWith({
			type: 'SetBookGeometry',
			trim: { w: 396, h: 612 },
			margins: null,
			bleed: null,
			grid: null,
		});
		expect(onClose).toHaveBeenCalled();
	});

	it('reads millimetres when that is the unit, and asks for nothing when nothing changed', async () => {
		const apply = vi.fn();
		render(
			<BookSizeDialog
				open
				onClose={() => undefined}
				book={book}
				units="millimetres"
				apply={apply}
			/>,
		);
		expect(screen.getByLabelText(/Width/)).toHaveValue('152.4');
		await userEvent.click(screen.getByRole('button', { name: 'Change size' }));
		expect(apply).not.toHaveBeenCalled();
	});

	it('refuses a page too narrow for its margins', async () => {
		const apply = vi.fn();
		render(
			<BookSizeDialog open onClose={() => undefined} book={book} units="inches" apply={apply} />,
		);
		await type(/Width/, '1');
		expect(screen.getByRole('alert')).toHaveTextContent('no room');
		expect(screen.getByRole('button', { name: 'Change size' })).toBeDisabled();
	});
});

describe('MarginsDialog', () => {
	it('sets the four margins in one SetBookGeometry', async () => {
		const apply = vi.fn();
		render(
			<MarginsDialog open onClose={() => undefined} book={book} units="inches" apply={apply} />,
		);
		expect(screen.getByLabelText(/Inside margin/)).toHaveValue('0.75');
		await type(/Inside margin/, '0.9');
		await type(/Top margin/, '0.5');
		await userEvent.click(screen.getByRole('button', { name: 'Set margins' }));
		expect(apply).toHaveBeenCalledWith({
			type: 'SetBookGeometry',
			trim: null,
			margins: { top: 36, bottom: 54, inside: 64.8, outside: 36 },
			bleed: null,
			grid: null,
		});
	});

	it('refuses margins that leave no room, and negative ones', async () => {
		const apply = vi.fn();
		render(
			<MarginsDialog open onClose={() => undefined} book={book} units="inches" apply={apply} />,
		);
		await type(/Inside margin/, '5.5');
		expect(screen.getByRole('alert')).toHaveTextContent('no room');
		expect(screen.getByRole('button', { name: 'Set margins' })).toBeDisabled();
		await type(/Inside margin/, '-1');
		expect(screen.getByRole('alert')).toHaveTextContent('below zero');
		expect(apply).not.toHaveBeenCalled();
	});
});
