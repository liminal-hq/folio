// Lets the main window ask whether it may close, so a book with changes is not lost
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { isTauri } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';

/** Hears the window being asked to close, by its ✕, Alt+F4 or the Quit command. */
export interface CloseRequests {
	/** `allow` answers whether the window may close; resolves to the function that stops listening. */
	onCloseRequested(allow: () => Promise<boolean>): Promise<() => void>;
}

/**
 * The window's own close request: while `allow` is thinking the window stays, and `false` keeps it.
 * Outside a Tauri window (a plain browser, a test) there is no window to ask, so nothing is heard.
 */
export const tauriCloseRequests: CloseRequests = {
	onCloseRequested: async (allow) =>
		isTauri()
			? getCurrentWindow().onCloseRequested(async (event) => {
					if (!(await allow())) event.preventDefault();
				})
			: () => {},
};
