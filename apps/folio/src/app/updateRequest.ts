// Asks the main window to show the update dialog, for the chip and the Help menu item
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

/** The event the main window listens for. */
export const UPDATE_EVENT = 'folio:updates';

/** Opens the update dialog; `check` also looks for a newer version, as the Help item does. */
export function requestUpdateDialog(check = false): void {
	window.dispatchEvent(new CustomEvent(UPDATE_EVENT, { detail: { check } }));
}
