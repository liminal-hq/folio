// Tests for the stage header's Marks menu
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { render, screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import { MarksMenu } from './MarksMenu';
import { DEFAULT_MARK_COLOURS, DEFAULT_MARKS, MARK_PALETTES } from './marksModel';

function setup() {
	const onMarks = vi.fn();
	const onColour = vi.fn();
	render(
		<MarksMenu
			marks={DEFAULT_MARKS}
			colours={DEFAULT_MARK_COLOURS}
			onMarks={onMarks}
			onColour={onColour}
		/>,
	);
	return { onMarks, onColour };
}

describe('the Marks menu', () => {
	it('opens from its button and closes on Escape or a press outside', async () => {
		setup();
		expect(screen.queryByRole('dialog')).toBeNull();
		await userEvent.click(screen.getByRole('button', { name: 'Marks' }));
		expect(screen.getByRole('dialog', { name: 'Page marks' })).toBeInTheDocument();
		await userEvent.keyboard('{Escape}');
		expect(screen.queryByRole('dialog')).toBeNull();
		await userEvent.click(screen.getByRole('button', { name: 'Marks' }));
		await userEvent.click(document.body);
		expect(screen.queryByRole('dialog')).toBeNull();
	});

	it('switches a mark on and off', async () => {
		const { onMarks } = setup();
		await userEvent.click(screen.getByRole('button', { name: 'Marks' }));
		const dialog = screen.getByRole('dialog');
		expect(within(dialog).getByRole('checkbox', { name: 'Margins' })).toBeChecked();
		await userEvent.click(within(dialog).getByRole('checkbox', { name: 'Column grid' }));
		expect(onMarks).toHaveBeenCalledWith({ columns: true });
	});

	it('offers four colours for each mark and says which is in use', async () => {
		const { onColour } = setup();
		await userEvent.click(screen.getByRole('button', { name: 'Marks' }));
		const group = screen.getByRole('group', { name: 'Baseline grid: Colour' });
		const swatches = within(group).getAllByRole('button');
		expect(swatches).toHaveLength(4);
		expect(swatches[0]).toHaveAttribute('aria-pressed', 'true');
		await userEvent.click(swatches[2]!);
		expect(onColour).toHaveBeenCalledWith('baseline', 2);
		expect(MARK_PALETTES.baseline).toHaveLength(4);
	});
});
