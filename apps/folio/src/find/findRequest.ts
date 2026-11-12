// Asks the open book's window to show the find bar, for the menu commands and their keys
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

/** The event the book's window listens for. */
export const FIND_EVENT = 'folio:find';

/** Opens the find bar, or with `replace` the find and replace bar. Nothing happens when no book is open. */
export function requestFind(replace = false): void {
	window.dispatchEvent(new CustomEvent(FIND_EVENT, { detail: { replace } }));
}
