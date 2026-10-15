// Tests for the Pages group: the page list, and the background of the page being shown
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { Book } from '@liminal-hq/folio-protocol/generated/Book';
import { render, screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import { PagesPanel } from './PagesPanel';

const colour = (c: string) => ({ type: 'colour', colour: c });

const book = {
	masterPages: [],
	spreads: [
		{ id: 'c', kind: 'cover', pages: [{ id: 'pc', master: null }], frames: [] },
		{
			id: 'a',
			kind: 'spread',
			pages: [
				{ id: 'p1', master: null },
				{ id: 'p2', master: null, bg: colour('#112233') },
			],
			frames: [],
			bgSpan: undefined,
		},
	],
} as unknown as Book;

function setup(shown = 1) {
	const props = { onShow: vi.fn(), onChange: vi.fn(), onApply: vi.fn() };
	render(<PagesPanel book={book} labels={['Cover', 'Pages 1–2']} shown={shown} {...props} />);
	return props;
}

describe('PagesPanel', () => {
	it('lists the spreads with the one shown marked, and shows another when it is chosen', async () => {
		const p = setup();
		expect(screen.getByRole('button', { name: 'Pages 1–2' })).toHaveAttribute(
			'aria-current',
			'true',
		);
		expect(screen.getByRole('button', { name: 'Cover' })).not.toHaveAttribute('aria-current');
		await userEvent.click(screen.getByRole('button', { name: 'Cover' }));
		expect(p.onShow).toHaveBeenCalledWith(0);
	});

	it('says in words what each place on the spread has', () => {
		setup();
		const background = within(screen.getByRole('region', { name: 'Page background' }));
		const right = background.getByText('Right page').closest('li')!;
		expect(within(right).getByText('Colour')).toBeInTheDocument();
		const left = background.getByText('Left page').closest('li')!;
		expect(within(left).getByText('None')).toBeInTheDocument();
	});

	it('opens the dialog at a place, and removes what a place has', async () => {
		const p = setup();
		await userEvent.click(
			screen.getByRole('button', { name: 'Change the background of Right page' }),
		);
		expect(p.onChange).toHaveBeenCalledWith('page:p2');
		await userEvent.click(
			screen.getByRole('button', { name: 'Remove the background of Right page' }),
		);
		expect(p.onApply).toHaveBeenCalledWith({
			type: 'SetBackground',
			target: { type: 'page', spread: 'a', page: 'p2' },
			bg: null,
		});
	});

	it('has nothing to remove where there is no background', () => {
		setup();
		expect(
			screen.getByRole('button', { name: 'Remove the background of Left page' }),
		).toBeDisabled();
	});

	it('offers the cover its own page only', () => {
		setup(0);
		expect(
			screen.getByRole('button', { name: 'Change the background of This page' }),
		).toBeInTheDocument();
		expect(screen.queryByRole('button', { name: /Every page/ })).toBeNull();
	});
});
