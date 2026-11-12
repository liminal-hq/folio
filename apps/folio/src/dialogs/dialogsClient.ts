// The page's side of the dialog windows: open one, hear how it ended, and (in a dialog) answer
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { invoke } from '@tauri-apps/api/core';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import {
	dialogRequest,
	dialogShown,
	finishDialog,
	onFinished,
	onMessage,
	postToOpener,
	type DialogRequest,
	type Finished,
	type Posted,
} from '@liminal-hq/plugin-window-dialogs';
import type { DialogKind } from '@liminal-hq/folio-protocol/generated/DialogKind';

/** What the page needs of the dialog windows, behind an interface so tests can stand in. */
export interface DialogsClient {
	/** Opens the dialog window of a kind for this window. */
	open(kind: DialogKind, payload: unknown): Promise<void>;
	/** Hears how the dialogs this window opened ended. */
	onFinished(listener: (finished: Finished) => void): Promise<() => void>;
	/** Hears messages the dialogs this window opened post to it. */
	onMessage(listener: (posted: Posted) => void): Promise<() => void>;
	/** In a dialog: what it was opened with. */
	request(): Promise<DialogRequest>;
	/** In a dialog: says it has painted. */
	shown(): Promise<number>;
	/** In a dialog: ends with a result, which the opener hears. */
	finish(result: unknown): Promise<void>;
	/** In a dialog: closes without a result, which the opener hears as a cancellation. */
	close(): Promise<void>;
	/** In a dialog: a message to the window that opened it, while the dialog stays open. */
	post(message: unknown): Promise<void>;
}

/** The app's own `open_dialog` command and the window-dialogs plugin. */
export const tauriDialogsClient: DialogsClient = {
	open: (kind, payload) => invoke<void>('open_dialog', { kind, payload }),
	onFinished: (listener) => onFinished(listener),
	onMessage: (listener) => onMessage(listener),
	request: () => dialogRequest(),
	shown: () => dialogShown(),
	finish: (result) => finishDialog(result),
	close: () => getCurrentWebviewWindow().close(),
	post: (message) => postToOpener(message),
};
