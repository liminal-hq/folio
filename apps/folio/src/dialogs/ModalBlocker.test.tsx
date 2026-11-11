// Tests the cover that blocks a window while a dialog window is open
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { act, render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import type { ModalState } from '@liminal-hq/plugin-window-dialogs';
import type { DialogsClient } from './dialogsClient';
import { ModalBlocker } from './ModalBlocker';

function setup() {
	let emit: (state: ModalState) => void = () => {};
	const focus = vi.fn(async () => {});
	const client = {
		onModal: async (listener: (state: ModalState) => void) => {
			emit = listener;
			return () => {};
		},
		focus,
	} as unknown as DialogsClient;
	render(<ModalBlocker client={client} />);
	return { focus, emit: (state: ModalState) => act(() => emit(state)) };
}

describe('the modal blocker', () => {
	it('covers the window while a dialog has it blocked, and lets go after', async () => {
		const { emit } = setup();
		await act(async () => {});
		expect(screen.queryByTestId('modal-blocker')).not.toBeInTheDocument();
		emit({ open: true, label: 'dialog-export', native: false });
		expect(screen.getByTestId('modal-blocker')).toBeInTheDocument();
		emit({ open: false, label: 'dialog-export', native: false });
		expect(screen.queryByTestId('modal-blocker')).not.toBeInTheDocument();
	});

	it('brings the dialog forward when the cover is pressed', async () => {
		const { emit, focus } = setup();
		await act(async () => {});
		emit({ open: true, label: 'dialog-export', native: false });
		await userEvent.click(screen.getByTestId('modal-blocker'));
		expect(focus).toHaveBeenCalled();
	});
});
