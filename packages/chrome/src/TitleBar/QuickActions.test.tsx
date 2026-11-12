// Tests for the title bar's quick actions strip: placement, keyboard, drag regions and collapse
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { act, fireEvent, render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { WindowChromeProvider } from '../WindowChromeProvider/WindowChromeProvider';
import { AppMenuButton } from './AppMenuButton';
import type { ButtonLayout } from './buttonLayout';
import type { QuickAction } from './QuickActions';
import { TitleBar } from './TitleBar';
import type { WindowControls } from './windowControls';

function controls(): WindowControls {
	return {
		minimize: vi.fn(),
		toggleMaximize: vi.fn(),
		close: vi.fn(),
		startDragging: vi.fn(),
		setAlwaysOnTop: vi.fn(),
		isMaximized: vi.fn(async () => false),
		onMaximizedChange: vi.fn(() => () => {}),
		handlesDoubleClickNatively: true,
	};
}

function actions(overrides: Partial<Record<string, Partial<QuickAction>>> = {}): QuickAction[] {
	return ['save', 'new', 'open'].map((id) => ({
		id,
		label: id[0]!.toUpperCase() + id.slice(1),
		icon: <svg data-testid={`icon-${id}`} />,
		onSelect: vi.fn(),
		...overrides[id],
	}));
}

function renderBar(
	list: QuickAction[],
	options: { buttonLayout?: ButtonLayout; controlsStyle?: 'gnome' | 'win11' } = {},
) {
	return render(
		<WindowChromeProvider controls={controls()}>
			<button type="button">Before</button>
			<TitleBar
				start={<AppMenuButton label="Folio" items={[]} onSelect={() => {}} />}
				quickActions={list}
				center={<span data-testid="title">Title</span>}
				{...options}
			/>
		</WindowChromeProvider>,
	);
}

afterEach(() => {
	vi.unstubAllGlobals();
});

describe('TitleBar quick actions', () => {
	it('renders a labelled toolbar right after the app menu button', () => {
		renderBar(actions());
		const toolbar = screen.getByRole('toolbar', { name: 'Quick actions' });
		const menu = screen.getByRole('button', { name: 'Folio' });
		expect(menu.compareDocumentPosition(toolbar) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
		expect(menu.parentElement?.nextElementSibling).toBe(toolbar);
		expect(toolbar.parentElement).toHaveAttribute('data-group', 'start');
	});

	it('renders nothing when there are no actions', () => {
		renderBar([]);
		expect(screen.queryByRole('toolbar')).toBeNull();
	});

	it('names each button with its shortcut and badge', () => {
		renderBar(
			actions({ save: { shortcut: 'Ctrl+S', badge: true, badgeLabel: 'unsaved changes' } }),
		);
		const save = screen.getByRole('button', { name: 'Save, unsaved changes (Ctrl+S)' });
		expect(save).toHaveAttribute('title', 'Save, unsaved changes (Ctrl+S)');
		expect(save.querySelector('[class*="badge"]')).not.toBeNull();
		expect(
			screen.getByRole('button', { name: 'New' }).querySelector('[class*="badge"]'),
		).toBeNull();
	});

	it('runs an action on click, but not a disabled one', async () => {
		const list = actions({ new: { disabled: true } });
		renderBar(list);
		await userEvent.click(screen.getByRole('button', { name: 'Save' }));
		await userEvent.click(screen.getByRole('button', { name: 'New' }));
		expect(list[0]!.onSelect).toHaveBeenCalledTimes(1);
		expect(list[1]!.onSelect).not.toHaveBeenCalled();
		expect(screen.getByRole('button', { name: 'New' })).toHaveAttribute('aria-disabled', 'true');
	});

	it('reports a toggle as pressed', () => {
		renderBar(actions({ open: { pressed: true } }));
		expect(screen.getByRole('button', { name: 'Open' })).toHaveAttribute('aria-pressed', 'true');
		expect(screen.getByRole('button', { name: 'Save' })).not.toHaveAttribute('aria-pressed');
	});

	it('has one tab stop that roves with the arrow keys, Home and End', async () => {
		const user = userEvent.setup();
		renderBar(actions());
		const [save, nu, open] = ['Save', 'New', 'Open'].map((n) =>
			screen.getByRole('button', { name: n }),
		);
		expect([save, nu, open].map((b) => b!.tabIndex)).toEqual([0, -1, -1]);
		await user.click(screen.getByRole('button', { name: 'Folio' }));
		await user.keyboard('{Escape}');
		save!.focus();
		await user.keyboard('{ArrowRight}');
		expect(nu).toHaveFocus();
		expect([save, nu, open].map((b) => b!.tabIndex)).toEqual([-1, 0, -1]);
		await user.keyboard('{End}');
		expect(open).toHaveFocus();
		await user.keyboard('{ArrowRight}');
		expect(save).toHaveFocus();
		await user.keyboard('{ArrowLeft}');
		expect(open).toHaveFocus();
		await user.keyboard('{Home}');
		expect(save).toHaveFocus();
	});

	it('is reached by Tab from the app menu button, and Tab leaves it in one step', async () => {
		const user = userEvent.setup();
		renderBar(actions());
		screen.getByRole('button', { name: 'Folio' }).focus();
		await user.tab();
		expect(screen.getByRole('button', { name: 'Save' })).toHaveFocus();
		await user.tab();
		expect(screen.getByRole('button', { name: 'Save' })).not.toHaveFocus();
		expect(screen.getByRole('toolbar').contains(document.activeElement)).toBe(false);
	});

	it('returns focus to where it came from on Escape', async () => {
		const user = userEvent.setup();
		renderBar(actions());
		screen.getByRole('button', { name: 'Before' }).focus();
		await user.tab(); // the app menu button
		await user.tab(); // into the toolbar
		expect(screen.getByRole('button', { name: 'Save' })).toHaveFocus();
		await user.keyboard('{Escape}');
		expect(screen.getByRole('button', { name: 'Folio' })).toHaveFocus();
	});

	it('keeps buttons out of the drag region but leaves the gaps draggable', () => {
		renderBar(actions());
		const toolbar = screen.getByRole('toolbar');
		expect(toolbar).toHaveAttribute('data-tauri-drag-region');
		for (const button of toolbar.querySelectorAll('button')) {
			expect(button).not.toHaveAttribute('data-tauri-drag-region');
		}
	});

	it('does not maximise or start a drag when a button is pressed or double-clicked', () => {
		const c = controls();
		render(
			<WindowChromeProvider controls={c}>
				<TitleBar quickActions={actions()} center={<span>Title</span>} />
			</WindowChromeProvider>,
		);
		const save = screen.getByRole('button', { name: 'Save' });
		fireEvent.mouseDown(save, { button: 0, detail: 1 });
		fireEvent.mouseDown(save, { button: 0, detail: 2 });
		fireEvent.mouseDown(save, { button: 0, detail: 3 });
		fireEvent.doubleClick(save, { detail: 2 });
		expect(c.toggleMaximize).not.toHaveBeenCalled();
		expect(c.startDragging).not.toHaveBeenCalled();
	});

	it('still opens the window menu from a right click on a gap in the strip', () => {
		renderBar(actions());
		fireEvent.contextMenu(screen.getByRole('toolbar'));
		expect(screen.getByRole('menu')).toBeInTheDocument();
	});

	it.each([
		['controls at the end', { start: [], end: ['minimise', 'maximise', 'close'] }],
		['controls at the start', { start: ['close', 'minimise', 'maximise'], end: [] }],
	] as [string, ButtonLayout][])('sits after the app menu with %s', (_name, buttonLayout) => {
		renderBar(actions(), { buttonLayout });
		const start = document.querySelector('[data-group="start"]')!;
		const toolbar = screen.getByRole('toolbar');
		expect(toolbar.parentElement).toBe(start);
		const order = Array.from(start.children);
		const menu = screen.getByRole('button', { name: 'Folio' }).parentElement!;
		expect(order.indexOf(toolbar)).toBe(order.indexOf(menu) + 1);
		// The centred title and the window controls are untouched.
		expect(screen.getByTestId('title').parentElement).toHaveAttribute('data-group', 'centre');
		expect(screen.getAllByRole('group', { name: 'Window controls' })).toHaveLength(1);
	});

	describe('narrow bars', () => {
		function stubWidths(client: number, scroll: number) {
			let notify: (() => void) | undefined;
			vi.stubGlobal(
				'ResizeObserver',
				class {
					constructor(callback: () => void) {
						notify = callback;
					}
					observe() {}
					disconnect() {}
				},
			);
			const widths = { client, scroll };
			const group = () => document.querySelector('[data-group="start"]') as HTMLElement;
			return {
				render() {
					const spyClient = vi
						.spyOn(HTMLElement.prototype, 'clientWidth', 'get')
						.mockImplementation(() => widths.client);
					const spyScroll = vi
						.spyOn(HTMLElement.prototype, 'scrollWidth', 'get')
						.mockImplementation(() => widths.scroll);
					renderBar(actions());
					return () => {
						spyClient.mockRestore();
						spyScroll.mockRestore();
					};
				},
				resize(to: number) {
					widths.client = to;
					act(() => notify?.());
				},
				group,
			};
		}

		it('collapses the strip when the start group is too narrow, and brings it back', () => {
			const bar = stubWidths(150, 260);
			const restore = bar.render();
			const toolbar = () => document.querySelector('[data-quick-actions]') as HTMLElement;
			expect(toolbar()).toHaveAttribute('hidden');
			expect(screen.queryByRole('toolbar')).toBeNull();
			// Still collapsed while the group is narrower than the full width.
			bar.resize(200);
			expect(toolbar()).toHaveAttribute('hidden');
			bar.resize(300);
			expect(toolbar()).not.toHaveAttribute('hidden');
			expect(screen.getByRole('toolbar')).toBeInTheDocument();
			restore();
		});

		it('keeps the strip when there is room', () => {
			const bar = stubWidths(400, 260);
			const restore = bar.render();
			expect(screen.getByRole('toolbar')).toBeInTheDocument();
			restore();
		});
	});
});
