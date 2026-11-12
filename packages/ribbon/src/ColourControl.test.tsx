// Tests for the ribbon's colour control: it opens the picker, keeps the editor's focus, and reports the colour
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { fireEvent, render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import { Ribbon } from './Ribbon';
import type { RibbonModel } from './types';

const d = 'M1 1h14';

function modelWith(value: string): RibbonModel {
	return {
		tabs: [
			{
				id: 'home',
				label: 'Home',
				groups: [
					{
						id: 'font',
						label: 'Font',
						items: [
							{
								id: 'colour',
								label: 'Text colour',
								d,
								kind: 'colour',
								value,
								swatch: value || '#1c1b20',
								colour: {
									inherited: { value: '#1c1b20', name: 'Ink' },
									swatches: [],
									pinned: [
										{ value: '#000000', name: 'Black' },
										{ value: '#ffffff', name: 'White' },
									],
									theme: [{ value: '#5b2a4a', name: 'Plum' }],
									recents: [],
									pageColour: '#ffffff',
									contrast: 'text',
									mode: 'easy',
								},
							},
						],
					},
				],
			},
		],
	};
}

function setup(value = '') {
	const onCommand = vi.fn();
	const view = render(<Ribbon model={modelWith(value)} mode="easy" onCommand={onCommand} />);
	return { onCommand, ...view };
}

describe('the colour control', () => {
	it('opens the picker on a click and reports a chosen colour, then closes', async () => {
		const { onCommand } = setup();
		const trigger = screen.getByRole('button', { name: 'Text colour' });
		expect(trigger).toHaveAttribute('aria-haspopup', 'dialog');
		expect(trigger).toHaveAttribute('aria-expanded', 'false');
		await userEvent.click(trigger);
		expect(trigger).toHaveAttribute('aria-expanded', 'true');
		await userEvent.click(screen.getByRole('button', { name: 'Black, #000000' }));
		expect(onCommand).toHaveBeenCalledWith(
			expect.objectContaining({ id: 'colour', value: '#000000' }),
		);
		expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
	});

	it('reports an empty value to go back to the style colour', async () => {
		const { onCommand } = setup('#5b2a4a');
		await userEvent.click(screen.getByRole('button', { name: 'Text colour' }));
		await userEvent.click(screen.getByRole('button', { name: 'Reset' }));
		expect(onCommand).toHaveBeenCalledWith(expect.objectContaining({ id: 'colour', value: '' }));
	});

	it('leaves the focus where it was on a press, so the words stay selected', async () => {
		setup();
		const outside = document.createElement('textarea');
		document.body.append(outside);
		outside.focus();
		const trigger = screen.getByRole('button', { name: 'Text colour' });
		const press = fireEvent.mouseDown(trigger);
		// `preventDefault` on the press is what keeps the focus in the editor.
		expect(press).toBe(false);
		await userEvent.click(trigger);
		const swatch = screen.getByRole('button', { name: 'Plum, #5b2a4a' });
		expect(fireEvent.mouseDown(swatch)).toBe(false);
		expect(outside).toHaveFocus();
		outside.remove();
	});

	it('marks the picker as part of the editing so the text editor is not ended', async () => {
		setup();
		await userEvent.click(screen.getByRole('button', { name: 'Text colour' }));
		expect(screen.getByRole('dialog')).toHaveAttribute('data-keeps-editor');
	});

	it('closes on Escape and gives the focus back to the button when it was in the picker', async () => {
		setup();
		const trigger = screen.getByRole('button', { name: 'Text colour' });
		trigger.focus();
		await userEvent.keyboard('{Enter}');
		expect(screen.getByRole('dialog')).toBeInTheDocument();
		// A keyboard opener moves the focus into the picker.
		expect(screen.getByRole('dialog').contains(document.activeElement)).toBe(true);
		await userEvent.keyboard('{Escape}');
		expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
		expect(trigger).toHaveFocus();
	});

	it('is unavailable without picker data', () => {
		const m = modelWith('');
		delete m.tabs[0]!.groups[0]!.items![0]!.colour;
		render(<Ribbon model={m} mode="easy" onCommand={vi.fn()} />);
		expect(screen.getByRole('button', { name: 'Text colour' })).toBeDisabled();
	});
});
