// Opens a dialog as a window and waits for its result
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import {
	dialogLabel,
	type DialogPayloads,
	type DialogResults,
	type OpenableKind,
} from './dialogKinds';
import { tauriDialogsClient, type DialogsClient } from './dialogsClient';

/**
 * Opens the dialog window of `kind` and resolves with what it handed back, or `null` when it was
 * closed without a result. Focus goes back to the element that had it when the dialog ends. Rejects
 * when the dialog could not open (this window has another modal dialog open already).
 */
export async function openDialog<K extends OpenableKind>(
	kind: K,
	payload: DialogPayloads[K],
	client: DialogsClient = tauriDialogsClient,
): Promise<DialogResults[K] | null> {
	const label = dialogLabel(kind);
	const opener = document.activeElement instanceof HTMLElement ? document.activeElement : null;
	let settle: (value: DialogResults[K] | null) => void = () => {};
	const done = new Promise<DialogResults[K] | null>((resolve) => (settle = resolve));
	const stop = await client.onFinished((finished) => {
		if (finished.label !== label) return;
		settle(finished.cancelled ? null : (finished.result as DialogResults[K]));
	});
	try {
		await client.open(kind, payload);
	} catch (reason) {
		stop();
		throw reason;
	}
	try {
		return await done;
	} finally {
		stop();
		if (opener?.isConnected) opener.focus();
	}
}
