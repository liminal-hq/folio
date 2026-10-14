// Tests for the classic toolbar
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import { ClassicToolbar } from './ClassicToolbar';
import type { ToolbarModel } from './types';

const d = 'M1 1h14';
const model: ToolbarModel = {
	groups: [
		[{ id: 'save', label: 'Save', d }],
		[
			{ id: 'bold', label: 'Bold', d, kind: 'toggle' },
			{ id: 'italic', label: 'Italic', d, kind: 'toggle' },
		],
		[{ id: 'style', label: 'Style', d, kind: 'menu', options: [['body', 'Story text']] }],
	],
};

describe('ClassicToolbar', () => {
	it('is a toolbar of icon-only buttons in groups', () => {
		render(<ClassicToolbar model={model} mode="easy" onCommand={vi.fn()} label="Toolbar" />);
		expect(screen.getByRole('toolbar', { name: 'Toolbar' })).toBeInTheDocument();
		expect(screen.getAllByRole('group')).toHaveLength(3);
		expect(screen.getByRole('button', { name: 'Save' }).textContent).toBe('');
	});

	it('reports presses, toggles, menu picks and the mode', async () => {
		const onCommand = vi.fn();
		render(<ClassicToolbar model={model} mode="easy" onCommand={onCommand} />);
		await userEvent.click(screen.getByRole('button', { name: 'Save' }));
		expect(onCommand).toHaveBeenLastCalledWith(expect.objectContaining({ id: 'save' }));
		await userEvent.click(screen.getByRole('button', { name: 'Bold' }));
		expect(screen.getByRole('button', { name: 'Bold' })).toHaveAttribute('aria-pressed', 'true');
		await userEvent.click(screen.getByRole('button', { name: 'Style' }));
		await userEvent.click(screen.getByRole('menuitem', { name: 'Story text' }));
		expect(onCommand).toHaveBeenLastCalledWith(
			expect.objectContaining({ id: 'style', value: 'body' }),
		);
		await userEvent.click(screen.getByRole('radio', { name: 'Expert' }));
		expect(onCommand).toHaveBeenLastCalledWith({ id: 'mode', value: 'expert' });
	});

	it('draws what is passed after the mode switch', () => {
		render(
			<ClassicToolbar
				model={model}
				mode="easy"
				onCommand={vi.fn()}
				end={<button>Export book</button>}
			/>,
		);
		expect(screen.getByRole('button', { name: 'Export book' })).toBeInTheDocument();
	});
});
