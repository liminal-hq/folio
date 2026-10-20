// Asks the open book's window to bring in a text file, for the menu command and the ribbon button
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

/** The event the book's window listens for. */
export const IMPORT_TEXT_EVENT = 'folio:import-text';

/** Asks for a file and brings its words into the book being shown. Nothing happens with no book. */
export function requestImportText(): void {
	window.dispatchEvent(new CustomEvent(IMPORT_TEXT_EVENT));
}
