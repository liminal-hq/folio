// Tests for the ribbon: tabs, modes, controls, menus, tooltips and the keyboard
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { act, fireEvent, render, screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { Ribbon, type RibbonProps } from './Ribbon';
import type { RibbonModel } from './types';
import { TOOLTIP_DELAY_MS, TOOLTIP_WARM_DELAY_MS } from './useRibbonTooltip';

const d = 'M1 1h14';
const model: RibbonModel = {
	file: {
		label: 'File',
		items: [
			['new', 'New book'],
			['open', 'Open'],
		],
	},
	tabs: [
		{
			id: 'home',
			label: 'Home',
			groups: [
				{
					id: 'history',
					label: 'History',
					items: [
						{ id: 'undo', label: 'Undo', d, key: 'Ctrl+Z', desc: 'Take back the last change' },
						{ id: 'redo', label: 'Redo', d, disabled: true },
					],
				},
				{
					id: 'text',
					label: 'Text',
					items: [
						{ id: 'alignL', label: 'Left', d, kind: 'toggle', radio: 'align' },
						{ id: 'alignC', label: 'Centre', d, kind: 'toggle', radio: 'align' },
						{ id: 'justify', label: 'Justify', d, kind: 'toggle', radio: 'align', expert: true },
						{
							id: 'spacing',
							label: 'Line spacing',
							d,
							kind: 'menu',
							options: [
								['1', 'Single'],
								['2', 'Double'],
							],
						},
						{
							id: 'font',
							label: 'Font',
							d,
							kind: 'select',
							options: [
								['a', 'Literata'],
								['b', 'Inter'],
							],
							value: 'a',
						},
						{ id: 'size', label: 'Size', d, kind: 'stepper', value: 12, min: 8, max: 14, step: 2 },
					],
				},
				{
					id: 'masters',
					label: 'Masters',
					expert: true,
					items: [{ id: 'master', label: 'Master pages', d }],
				},
			],
		},
		{
			id: 'insert',
			label: 'Insert',
			groups: [
				{
					id: 'frames',
					label: 'Frames',
					items: [{ id: 'text', label: 'Text box', d, size: 'large' }],
				},
			],
		},
		{
			id: 'picture',
			label: 'Picture',
			contextual: true,
			groups: [{ id: 'pic', label: 'Picture', items: [{ id: 'replace', label: 'Replace', d }] }],
		},
	],
};

function setup(props: Partial<RibbonProps> = {}) {
	const onCommand = vi.fn();
	const view = render(<Ribbon model={model} mode="easy" onCommand={onCommand} {...props} />);
	return { onCommand, ...view };
}

describe('Ribbon', () => {
	it('is a labelled region whose toolbar holds only the controls, not the tabs', () => {
		setup();
		expect(screen.getByRole('group', { name: 'Ribbon' })).toBeInTheDocument();
		const toolbar = screen.getByRole('toolbar', { name: 'Ribbon controls' });
		expect(within(toolbar).queryByRole('tab')).toBeNull();
		expect(within(toolbar).getByRole('group', { name: 'History' })).toBeInTheDocument();
		expect(screen.getByRole('tabpanel')).toContainElement(toolbar);
	});

	it('is a ribbon with tabs and a panel of named groups', () => {
		setup();
		expect(screen.getAllByRole('tab').map((t) => t.textContent)).toEqual(['Home', 'Insert']);
		expect(screen.getByRole('group', { name: 'History' })).toBeInTheDocument();
		expect(screen.getByRole('tab', { name: 'Home' })).toHaveAttribute('aria-selected', 'true');
	});

	it('hides expert groups and items in Easy and shows them in Expert', () => {
		const easy = setup();
		expect(screen.queryByRole('button', { name: 'Justify' })).toBeNull();
		expect(screen.queryByRole('group', { name: 'Masters' })).toBeNull();
		easy.unmount();
		setup({ mode: 'expert' });
		expect(screen.getByRole('button', { name: 'Justify' })).toBeInTheDocument();
		expect(screen.getByRole('group', { name: 'Masters' })).toBeInTheDocument();
	});

	it('shows contextual tabs only when listed', () => {
		const { rerender, onCommand } = setup();
		expect(screen.queryByRole('tab', { name: 'Picture' })).toBeNull();
		rerender(<Ribbon model={model} mode="easy" onCommand={onCommand} contextual={['picture']} />);
		expect(screen.getByRole('tab', { name: 'Picture' })).toBeInTheDocument();
	});

	it('falls back to the first tab when the active contextual tab goes away', async () => {
		const { rerender, onCommand } = setup({ contextual: ['picture'] });
		await userEvent.click(screen.getByRole('tab', { name: 'Picture' }));
		expect(screen.getByRole('button', { name: 'Replace' })).toBeInTheDocument();
		rerender(<Ribbon model={model} mode="easy" onCommand={onCommand} />);
		expect(screen.getByRole('tab', { name: 'Home' })).toHaveAttribute('aria-selected', 'true');
	});

	it('switches tabs and reports it', async () => {
		const { onCommand } = setup();
		await userEvent.click(screen.getByRole('tab', { name: 'Insert' }));
		expect(screen.getByRole('button', { name: 'Text box' })).toBeInTheDocument();
		expect(onCommand).toHaveBeenCalledWith({ id: 'tab', value: 'insert' });
	});

	it('reports a button press with its item', async () => {
		const { onCommand } = setup();
		await userEvent.click(screen.getByRole('button', { name: 'Undo' }));
		expect(onCommand).toHaveBeenCalledWith(expect.objectContaining({ id: 'undo' }));
	});

	it('does not fire a disabled button', async () => {
		const { onCommand } = setup();
		await userEvent.click(screen.getByRole('button', { name: 'Redo' }));
		expect(onCommand).not.toHaveBeenCalled();
	});

	it('toggles, exposing aria-pressed and the new state', async () => {
		const { onCommand } = setup();
		const left = screen.getByRole('button', { name: 'Left' });
		expect(left).toHaveAttribute('aria-pressed', 'false');
		await userEvent.click(left);
		expect(left).toHaveAttribute('aria-pressed', 'true');
		expect(onCommand).toHaveBeenCalledWith(expect.objectContaining({ id: 'alignL', value: true }));
	});

	it('makes toggles that share a radio key mutually exclusive', async () => {
		setup();
		await userEvent.click(screen.getByRole('button', { name: 'Left' }));
		await userEvent.click(screen.getByRole('button', { name: 'Centre' }));
		expect(screen.getByRole('button', { name: 'Left' })).toHaveAttribute('aria-pressed', 'false');
		expect(screen.getByRole('button', { name: 'Centre' })).toHaveAttribute('aria-pressed', 'true');
	});

	it('lets the app own a toggle through `active`', async () => {
		const owned: RibbonModel = {
			tabs: [
				{
					id: 't',
					label: 'T',
					groups: [
						{
							id: 'g',
							label: 'G',
							items: [{ id: 'bold', label: 'Bold', d, kind: 'toggle', active: true }],
						},
					],
				},
			],
		};
		const onCommand = vi.fn();
		render(<Ribbon model={owned} mode="easy" onCommand={onCommand} />);
		await userEvent.click(screen.getByRole('button', { name: 'Bold' }));
		expect(screen.getByRole('button', { name: 'Bold' })).toHaveAttribute('aria-pressed', 'true');
		expect(onCommand).toHaveBeenCalledWith(expect.objectContaining({ id: 'bold', value: false }));
	});

	it('opens a menu and reports the chosen option', async () => {
		const { onCommand } = setup();
		await userEvent.click(screen.getByRole('button', { name: 'Line spacing' }));
		await userEvent.click(screen.getByRole('menuitem', { name: 'Double' }));
		expect(onCommand).toHaveBeenCalledWith(expect.objectContaining({ id: 'spacing', value: '2' }));
		expect(screen.queryByRole('menu')).toBeNull();
	});

	it('opens the File menu and reports the choice', async () => {
		const { onCommand } = setup();
		await userEvent.click(screen.getByRole('button', { name: 'File' }));
		await userEvent.click(screen.getByRole('menuitem', { name: 'Open' }));
		expect(onCommand).toHaveBeenCalledWith({ id: 'file', value: 'open' });
	});

	it('reports a select change', async () => {
		const { onCommand } = setup();
		await userEvent.selectOptions(screen.getByRole('combobox', { name: 'Font' }), 'b');
		expect(onCommand).toHaveBeenCalledWith(expect.objectContaining({ id: 'font', value: 'b' }));
	});

	it('steps within the stepper bounds', async () => {
		const { onCommand } = setup();
		const size = screen.getByRole('group', { name: 'Size' });
		await userEvent.click(within(size).getByRole('button', { name: 'Increase Size' }));
		expect(onCommand).toHaveBeenCalledWith(expect.objectContaining({ id: 'size', value: 14 }));
		await userEvent.click(within(size).getByRole('button', { name: 'Decrease Size' }));
		expect(onCommand).toHaveBeenLastCalledWith(expect.objectContaining({ id: 'size', value: 10 }));
	});

	it('reports a mode change from the Easy and Expert switch', async () => {
		const { onCommand } = setup();
		expect(screen.getByRole('radio', { name: 'Easy' })).toBeChecked();
		await userEvent.click(screen.getByRole('radio', { name: 'Expert' }));
		expect(onCommand).toHaveBeenCalledWith({ id: 'mode', value: 'expert' });
	});

	it('keeps focus where it was when a button is pressed with the pointer', () => {
		setup();
		const undo = screen.getByRole('button', { name: 'Undo' });
		expect(fireEvent.mouseDown(undo)).toBe(false);
	});

	it('hides labels in icon-only mode but keeps the accessible name', () => {
		setup({ showLabels: false });
		const undo = screen.getByRole('button', { name: 'Undo' });
		expect(undo.textContent).toBe('');
	});

	describe('keyboard', () => {
		it('moves between tabs with the arrow keys', async () => {
			const { onCommand } = setup();
			screen.getByRole('tab', { name: 'Home' }).focus();
			await userEvent.keyboard('{ArrowRight}');
			expect(screen.getByRole('tab', { name: 'Insert' })).toHaveFocus();
			expect(onCommand).toHaveBeenCalledWith({ id: 'tab', value: 'insert' });
			await userEvent.keyboard('{ArrowRight}');
			expect(screen.getByRole('tab', { name: 'Home' })).toHaveFocus();
		});

		it('moves into the panel and across its controls, and Esc goes back to the tab', async () => {
			setup();
			screen.getByRole('tab', { name: 'Home' }).focus();
			await userEvent.keyboard('{ArrowDown}');
			expect(screen.getByRole('button', { name: 'Undo' })).toHaveFocus();
			await userEvent.keyboard('{ArrowRight}');
			expect(screen.getByRole('button', { name: 'Left' })).toHaveFocus();
			await userEvent.keyboard('{Escape}');
			expect(screen.getByRole('tab', { name: 'Home' })).toHaveFocus();
		});

		it('keeps one control in the panel as the Tab stop and moves it with the arrows', async () => {
			setup();
			const panel = screen.getByRole('tabpanel');
			const stops = () =>
				within(panel)
					.getAllByRole('button')
					.filter((b) => b.tabIndex === 0 && !(b as HTMLButtonElement).disabled);
			expect(stops()).toHaveLength(1);
			expect(stops()[0]).toHaveAccessibleName('Undo');
			screen.getByRole('button', { name: 'Undo' }).focus();
			await userEvent.keyboard('{ArrowRight}');
			expect(stops()).toHaveLength(1);
			expect(stops()[0]).toHaveAccessibleName('Left');
		});

		it('activates a focused button with Enter', async () => {
			const { onCommand } = setup();
			screen.getByRole('button', { name: 'Undo' }).focus();
			await userEvent.keyboard('{Enter}');
			expect(onCommand).toHaveBeenCalledWith(expect.objectContaining({ id: 'undo' }));
		});
	});

	describe('variants', () => {
		it('draws a simplified ribbon as one row without group labels', () => {
			setup({ variant: 'simplified' });
			expect(screen.queryByRole('tabpanel')).toBeNull();
			expect(screen.getByRole('button', { name: 'Undo' }).textContent).toBe('');
		});

		it('keeps a collapsed ribbon to its tabs until a tab is clicked, and Esc closes it', async () => {
			setup({ variant: 'collapsed' });
			expect(screen.queryByRole('tabpanel')).toBeNull();
			await userEvent.click(screen.getByRole('tab', { name: 'Insert' }));
			expect(screen.getByRole('tabpanel')).toBeInTheDocument();
			await userEvent.keyboard('{Tab}{Escape}');
			expect(screen.queryByRole('tabpanel')).toBeNull();
		});

		it('does not toggle a collapsed ribbon when arrowing across tabs, only when activating', async () => {
			setup({ variant: 'collapsed' });
			screen.getByRole('tab', { name: 'Home' }).focus();
			await userEvent.keyboard('{ArrowRight}');
			expect(screen.getByRole('tab', { name: 'Insert' })).toHaveFocus();
			expect(screen.queryByRole('tabpanel')).toBeNull();
			await userEvent.keyboard('{Enter}');
			expect(screen.getByRole('tabpanel')).toBeInTheDocument();
			await userEvent.keyboard('{ArrowLeft}');
			expect(screen.getByRole('tabpanel')).toBeInTheDocument();
		});

		it('keeps a pinned collapsed ribbon open', async () => {
			setup({ variant: 'collapsed' });
			await userEvent.click(screen.getByRole('button', { name: 'Keep the ribbon open' }));
			expect(screen.getByRole('tabpanel')).toBeInTheDocument();
		});

		it('drops group labels in the compact density', () => {
			setup({ density: 'compact' });
			expect(screen.queryByText('History')).toBeNull();
			expect(screen.getByRole('group', { name: 'History' })).toBeInTheDocument();
		});
	});

	it('forgets a contextual tab that went away, so it is not restored when it returns', async () => {
		const { rerender, onCommand } = setup({ contextual: ['picture'] });
		await userEvent.click(screen.getByRole('tab', { name: 'Picture' }));
		rerender(<Ribbon model={model} mode="easy" onCommand={onCommand} />);
		rerender(<Ribbon model={model} mode="easy" onCommand={onCommand} contextual={['picture']} />);
		expect(screen.getByRole('tab', { name: 'Home' })).toHaveAttribute('aria-selected', 'true');
	});

	it('keeps ribbon-held state per control, so two controls sharing an id are independent', async () => {
		const twin = (id: string) => ({
			id,
			label: id,
			groups: [
				{
					id: `${id}-a`,
					label: `${id} A`,
					items: [{ id: 'bold', label: `Bold ${id}A`, d, kind: 'toggle' as const }],
				},
				{
					id: `${id}-b`,
					label: `${id} B`,
					items: [{ id: 'bold', label: `Bold ${id}B`, d, kind: 'toggle' as const }],
				},
			],
		});
		setup({ model: { tabs: [twin('x')] } });
		await userEvent.click(screen.getByRole('button', { name: 'Bold xA' }));
		expect(screen.getByRole('button', { name: 'Bold xA' })).toHaveAttribute('aria-pressed', 'true');
		expect(screen.getByRole('button', { name: 'Bold xB' })).toHaveAttribute(
			'aria-pressed',
			'false',
		);
	});

	describe('tooltips', () => {
		beforeEach(() => vi.useFakeTimers({ shouldAdvanceTime: false }));
		afterEach(() => vi.useRealTimers());

		it('shows the name, shortcut and description after the delay', () => {
			setup();
			fireEvent.pointerEnter(screen.getByRole('button', { name: 'Undo' }));
			expect(screen.queryByRole('tooltip')).toBeNull();
			act(() => void vi.advanceTimersByTime(TOOLTIP_DELAY_MS));
			const tip = screen.getByRole('tooltip');
			expect(tip).toHaveTextContent('Undo');
			expect(tip).toHaveTextContent('Ctrl+Z');
			expect(tip).toHaveTextContent('Take back the last change');
		});

		it('appears quickly when moving between buttons', () => {
			setup();
			fireEvent.pointerEnter(screen.getByRole('button', { name: 'Undo' }));
			act(() => void vi.advanceTimersByTime(TOOLTIP_DELAY_MS));
			fireEvent.pointerLeave(screen.getByRole('button', { name: 'Undo' }));
			fireEvent.pointerEnter(screen.getByRole('button', { name: 'Left' }));
			act(() => void vi.advanceTimersByTime(TOOLTIP_WARM_DELAY_MS));
			expect(screen.getByRole('tooltip')).toHaveTextContent('Left');
		});

		it('does not show when the pointer leaves first', () => {
			setup();
			const undo = screen.getByRole('button', { name: 'Undo' });
			fireEvent.pointerEnter(undo);
			fireEvent.pointerLeave(undo);
			act(() => void vi.advanceTimersByTime(TOOLTIP_DELAY_MS * 2));
			expect(screen.queryByRole('tooltip')).toBeNull();
		});

		it('hides when a control is pressed', () => {
			setup();
			const undo = screen.getByRole('button', { name: 'Undo' });
			fireEvent.pointerEnter(undo);
			act(() => void vi.advanceTimersByTime(TOOLTIP_DELAY_MS));
			expect(screen.getByRole('tooltip')).toBeInTheDocument();
			fireEvent.click(undo);
			expect(screen.queryByRole('tooltip')).toBeNull();
		});

		it('hides when a control opens a menu', () => {
			setup();
			const spacing = screen.getByRole('button', { name: 'Line spacing' });
			fireEvent.pointerEnter(spacing);
			act(() => void vi.advanceTimersByTime(TOOLTIP_DELAY_MS));
			fireEvent.click(spacing);
			expect(screen.queryByRole('tooltip')).toBeNull();
			expect(screen.getByRole('menu')).toBeInTheDocument();
		});

		it('hides on Escape', () => {
			setup();
			fireEvent.pointerEnter(screen.getByRole('button', { name: 'Undo' }));
			act(() => void vi.advanceTimersByTime(TOOLTIP_DELAY_MS));
			fireEvent.keyDown(document.body, { key: 'Escape' });
			expect(screen.queryByRole('tooltip')).toBeNull();
		});

		it('does not show for a disabled anchor', () => {
			setup();
			fireEvent.pointerEnter(screen.getByRole('button', { name: 'Redo' }));
			act(() => void vi.advanceTimersByTime(TOOLTIP_DELAY_MS));
			expect(screen.queryByRole('tooltip')).toBeNull();
		});

		it('does not show, or throw, when the anchor unmounts before the delay', () => {
			const { rerender, onCommand } = setup({ contextual: ['picture'] });
			fireEvent.click(screen.getByRole('tab', { name: 'Picture' }));
			fireEvent.pointerEnter(screen.getByRole('button', { name: 'Replace' }));
			rerender(<Ribbon model={model} mode="easy" onCommand={onCommand} />);
			act(() => void vi.advanceTimersByTime(TOOLTIP_DELAY_MS));
			expect(screen.queryByRole('tooltip')).toBeNull();
		});

		it('hides when its anchor unmounts while shown', async () => {
			const { rerender, onCommand } = setup({ contextual: ['picture'] });
			fireEvent.click(screen.getByRole('tab', { name: 'Picture' }));
			fireEvent.pointerEnter(screen.getByRole('button', { name: 'Replace' }));
			act(() => void vi.advanceTimersByTime(TOOLTIP_DELAY_MS));
			expect(screen.getByRole('tooltip')).toBeInTheDocument();
			rerender(<Ribbon model={model} mode="easy" onCommand={onCommand} />);
			await act(async () => {
				await Promise.resolve();
			});
			expect(screen.queryByRole('tooltip')).toBeNull();
		});
	});
});
