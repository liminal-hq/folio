// Tests for the colour picker: swatches, typed colours and none
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import { FAKE_PAINTS } from '../books/fakeBooksClient';
import { ColourPicker } from './ColourPicker';

const setup = (value: string | null = '#ffffff', allowNone = true) => {
	const onPick = vi.fn();
	render(
		<ColourPicker
			label="Page colour"
			value={value}
			swatches={FAKE_PAINTS.swatches}
			allowNone={allowNone}
			onPick={onPick}
		/>,
	);
	return onPick;
};

describe('ColourPicker', () => {
	it('picks a swatch at once and marks the one in use', async () => {
		const onPick = setup('#f6efdc');
		expect(screen.getByRole('button', { name: 'Cream' })).toHaveAttribute('aria-pressed', 'true');
		await userEvent.click(screen.getByRole('button', { name: 'Slate' }));
		expect(onPick).toHaveBeenCalledWith('#1f3a5f');
	});

	it('keeps a typed colour on Enter, in its stored form', async () => {
		const onPick = setup();
		const box = screen.getByRole('textbox');
		await userEvent.clear(box);
		await userEvent.type(box, 'Fa0{Enter}');
		expect(onPick).toHaveBeenCalledWith('#ffaa00');
	});

	it('refuses a typo, says so, and puts the colour back on Escape', async () => {
		const onPick = setup();
		const box = screen.getByRole('textbox');
		await userEvent.clear(box);
		await userEvent.type(box, '#12');
		expect(screen.getByRole('alert')).toBeInTheDocument();
		await userEvent.type(box, '{Enter}');
		expect(onPick).not.toHaveBeenCalled();
		await userEvent.type(box, '{Escape}');
		expect(box).toHaveValue('#ffffff');
	});

	it('offers none only when the fill can be cleared', async () => {
		const onPick = setup('#ffffff', true);
		await userEvent.click(screen.getByRole('button', { name: 'None' }));
		expect(onPick).toHaveBeenCalledWith(null);
	});

	it('leaves out none when it cannot be cleared', () => {
		setup('#ffffff', false);
		expect(screen.queryByRole('button', { name: 'None' })).toBeNull();
	});
});
