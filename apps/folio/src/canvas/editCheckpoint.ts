// Asks an open text editor to keep the words typed so far as an undo step of their own
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

/** The event the editor listens for; its detail says whether the editing should end as well. */
export const KEEP_WORDS_EVENT = 'folio:keep-words';

export interface KeepWordsDetail {
	end: boolean;
}

/**
 * Something is about to change the book while words are being typed (a style, a move, an undo). The
 * words typed so far are kept first, so that change lands after them and undo takes them back in the
 * order they happened. With `end`, the editing closes too, as an undo must, since the box would
 * otherwise show words the book no longer holds. Nothing happens when no editor is open.
 */
export function keepWords(end = false): void {
	window.dispatchEvent(new CustomEvent<KeepWordsDetail>(KEEP_WORDS_EVENT, { detail: { end } }));
}
