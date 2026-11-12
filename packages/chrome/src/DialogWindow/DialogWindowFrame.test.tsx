// Tests the dialog window frame: focus on start, Esc, Enter and the footer
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { render, screen } from '@testing-library/react';
import { createPortal } from 'react-dom';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import { DialogActions, DialogButton } from '../Dialog/DialogActions';
import type { WindowControls } from '../TitleBar/windowControls';
import { WindowChromeProvider } from '../WindowChromeProvider/WindowChromeProvider';
import { DialogWindowFrame } from './DialogWindowFrame';

const controls: WindowControls = {
	minimize: vi.fn(),
	toggleMaximize: vi.fn(),
	close: vi.fn(),
	setAlwaysOnTop: vi.fn(),
	isMaximized: vi.fn().mockResolvedValue(false),
	onMaximizedChange: () => () => {},
};

function setup(extra: { dismissible?: boolean } = {}) {
	const onClose = vi.fn();
	const onDefaultAction = vi.fn();
	render(
		<WindowChromeProvider controls={controls}>
			<DialogWindowFrame
				title="Details"
				titleBar={<div>bar</div>}
				onClose={onClose}
				onDefaultAction={onDefaultAction}
				footer={
					<DialogActions>
						<DialogButton onClick={() => onClose('cancel')}>Cancel</DialogButton>
						<DialogButton variant="primary" onClick={onDefaultAction}>
							Save
						</DialogButton>
					</DialogActions>
				}
				{...extra}
			>
				<label>
					Name
					<input />
				</label>
				<label>
					Notes
					<textarea />
				</label>
			</DialogWindowFrame>
		</WindowChromeProvider>,
	);
	return { onClose, onDefaultAction };
}

describe('the dialog window frame', () => {
	it('is named by its title and starts with focus on the first field', () => {
		setup();
		expect(screen.getByRole('dialog', { name: 'Details' })).toBeInTheDocument();
		expect(screen.getByRole('textbox', { name: 'Name' })).toHaveFocus();
	});

	it('asks to close on Esc, unless the dialog must be answered', async () => {
		const first = setup();
		await userEvent.keyboard('{Escape}');
		expect(first.onClose).toHaveBeenCalledWith('escape');
	});

	it('does nothing on Esc when it is not dismissible', async () => {
		const { onClose } = setup({ dismissible: false });
		await userEvent.keyboard('{Escape}');
		expect(onClose).not.toHaveBeenCalled();
	});

	it('runs the default action on Enter in a field, but not in a text area', async () => {
		const { onDefaultAction } = setup();
		await userEvent.keyboard('{Enter}');
		expect(onDefaultAction).toHaveBeenCalledTimes(1);
		await userEvent.click(screen.getByRole('textbox', { name: 'Notes' }));
		await userEvent.keyboard('{Enter}');
		expect(onDefaultAction).toHaveBeenCalledTimes(1);
	});

	it('keeps Tab inside the dialog', async () => {
		setup();
		await userEvent.tab();
		await userEvent.tab();
		await userEvent.tab();
		await userEvent.tab();
		expect(screen.getByRole('textbox', { name: 'Name' })).toHaveFocus();
	});

	it('leaves the keys of a popover portalled outside the dialog to the popover', async () => {
		const onDefaultAction = vi.fn();
		render(
			<WindowChromeProvider controls={controls}>
				<DialogWindowFrame
					title="Details"
					titleBar={<div>bar</div>}
					onClose={vi.fn()}
					onDefaultAction={onDefaultAction}
					footer={<button>Save</button>}
				>
					<input aria-label="Name" />
					{createPortal(
						<div role="dialog" aria-label="Popover">
							<input aria-label="First" />
							<input aria-label="Second" />
						</div>,
						document.body,
					)}
				</DialogWindowFrame>
			</WindowChromeProvider>,
		);
		screen.getByRole('textbox', { name: 'First' }).focus();
		await userEvent.tab();
		expect(screen.getByRole('textbox', { name: 'Second' })).toHaveFocus();
		await userEvent.keyboard('{Enter}');
		expect(onDefaultAction).not.toHaveBeenCalled();
	});

	it('keeps the body in a bounded column and the buttons in a footer pinned below it', () => {
		setup();
		const body = document.querySelector('[data-dialog-body]') as HTMLElement;
		const column = document.querySelector('[data-dialog-column]') as HTMLElement;
		const footer = document.querySelector('[data-dialog-footer]') as HTMLElement;
		expect(body).toContainElement(column);
		expect(column).toContainElement(screen.getByRole('textbox', { name: 'Name' }));
		// The footer is a sibling after the scrolling body, not inside it, and its buttons share a column.
		expect(footer).not.toBeNull();
		expect(body.contains(footer)).toBe(false);
		expect(body.compareDocumentPosition(footer) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
		expect(footer).toContainElement(screen.getByRole('button', { name: 'Save' }));
		expect(footer.firstElementChild?.className).toBe(column.className);
	});

	it('does nothing when its title bar is double-clicked, so the window is not maximised', async () => {
		const reached = vi.fn();
		render(
			<WindowChromeProvider controls={controls}>
				<DialogWindowFrame
					title="Details"
					titleBar={
						<div
							data-testid="bar"
							onMouseDown={reached}
							onMouseUp={reached}
							onDoubleClick={reached}
						>
							bar
						</div>
					}
					onClose={() => {}}
				>
					body
				</DialogWindowFrame>
			</WindowChromeProvider>,
		);
		await userEvent.dblClick(screen.getByTestId('bar'));
		// The first press of each click gets through (it starts a drag); the second, and the
		// `dblclick` that follows it, do not.
		expect(reached.mock.calls.map(([event]) => event.type)).toEqual(['mousedown', 'mouseup']);
		expect(controls.toggleMaximize).not.toHaveBeenCalled();
	});
});
