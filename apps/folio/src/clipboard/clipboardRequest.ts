// Asks the open book's window to copy, cut or paste, for the menu commands
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

/** The event the book's window listens for. */
export const CLIPBOARD_EVENT = 'folio:clipboard';

export type ClipboardKind = 'copy' | 'cut' | 'paste';

/** Copies, cuts or pastes page objects in the book being shown. Nothing happens with no book open. */
export function requestClipboard(kind: ClipboardKind): void {
	window.dispatchEvent(new CustomEvent<ClipboardKind>(CLIPBOARD_EVENT, { detail: kind }));
}
