// Tests opening a dialog window and waiting for its result
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { describe, expect, it, vi } from 'vitest';
import type { Finished } from '@liminal-hq/plugin-window-dialogs';
import { openDialog } from './openDialog';
import type { DialogsClient } from './dialogsClient';

function fakeClient(open: DialogsClient['open'] = async () => {}) {
	let listener: (finished: Finished) => void = () => {};
	const stop = vi.fn();
	const client: DialogsClient = {
		open: vi.fn(open),
		onFinished: async (next) => {
			listener = next;
			return stop;
		},
		onModal: async () => () => {},
		request: async () => ({ requestId: 1, label: 'dialog-export', payload: null, modal: true }),
		shown: async () => 0,
		finish: async () => {},
		close: async () => {},
		post: async () => {},
		focus: async () => {},
	};
	return { client, stop, finish: (finished: Finished) => listener(finished) };
}

const PAYLOAD = { bookId: 'b', autoStart: null };

describe('openDialog', () => {
	it('resolves with the result the dialog handed back', async () => {
		const { client, finish, stop } = fakeClient();
		const asked = openDialog('Export', PAYLOAD, client);
		await Promise.resolve();
		await Promise.resolve();
		finish({ requestId: 1, label: 'dialog-export', cancelled: false, result: { next: 'check' } });
		expect(await asked).toEqual({ next: 'check' });
		expect(client.open).toHaveBeenCalledWith('Export', PAYLOAD);
		expect(stop).toHaveBeenCalled();
	});

	it('resolves with null when the window was closed', async () => {
		const { client, finish } = fakeClient();
		const asked = openDialog('Export', PAYLOAD, client);
		await Promise.resolve();
		await Promise.resolve();
		finish({ requestId: 1, label: 'dialog-export', cancelled: true, result: null });
		expect(await asked).toBeNull();
	});

	it('ignores how another dialog ended', async () => {
		const { client, finish } = fakeClient();
		let ended = false;
		const asked = openDialog('Export', PAYLOAD, client).then((r) => {
			ended = true;
			return r;
		});
		await Promise.resolve();
		await Promise.resolve();
		finish({ requestId: 9, label: 'dialog-other', cancelled: true, result: null });
		await Promise.resolve();
		expect(ended).toBe(false);
		finish({ requestId: 1, label: 'dialog-export', cancelled: true, result: null });
		await asked;
	});

	it('gives focus back to the element that had it', async () => {
		const button = document.createElement('button');
		document.body.append(button);
		button.focus();
		const { client, finish } = fakeClient();
		const asked = openDialog('Export', PAYLOAD, client);
		await Promise.resolve();
		await Promise.resolve();
		document.body.focus();
		finish({ requestId: 1, label: 'dialog-export', cancelled: true, result: null });
		await asked;
		expect(button).toHaveFocus();
		button.remove();
	});

	it('rejects, and stops listening, when the dialog could not open', async () => {
		const { client, stop } = fakeClient(async () => {
			throw new Error('a modal dialog is open');
		});
		await expect(openDialog('Export', PAYLOAD, client)).rejects.toThrow('a modal dialog is open');
		expect(stop).toHaveBeenCalled();
	});
});
