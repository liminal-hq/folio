// Asks the open book's window to show the Export dialog, for the menu command and its key
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

/** The event the book's window listens for. */
export const EXPORT_EVENT = 'folio:export';

/** Opens the Export dialog on the book being shown. Nothing happens when no book is open. */
export function requestExport(): void {
	window.dispatchEvent(new CustomEvent(EXPORT_EVENT));
}
