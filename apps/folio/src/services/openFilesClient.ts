// Hears the books the system asks Folio to open, such as a `.folio` file double-clicked in a file manager
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { isTauri, invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { message } from '@tauri-apps/plugin-dialog';

/** The paths the app was asked to open, kept in Rust until the main window takes them. */
export interface OpenFilesClient {
	/** Takes every path waiting, and forgets them. */
	take(): Promise<string[]>;
	/** Hears that more paths are waiting; resolves to the function that stops listening. */
	onWaiting(listener: () => void): Promise<() => void>;
	/** Tells the person a book could not be opened. */
	report(text: string): Promise<void>;
}

/** The app's `take_open_files` command and `folio://open-files` event. Outside Tauri nothing is asked. */
export const tauriOpenFilesClient: OpenFilesClient = {
	take: async () => (isTauri() ? invoke<string[]>('take_open_files') : []),
	onWaiting: async (listener) => (isTauri() ? listen('folio://open-files', listener) : () => {}),
	report: async (text) => {
		await message(text, { kind: 'error' });
	},
};
