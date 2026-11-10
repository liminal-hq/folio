// Which stretch of words is selected in the open text editor, for the ribbon's bold, italic and font
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { ParaLook } from '@liminal-hq/folio-protocol/generated/ParaLook';
import type { TextTarget } from '@liminal-hq/folio-protocol/generated/TextTarget';

/** A selected stretch of a frame's words, counted in characters (not UTF-16 units). */
export interface TextSelection {
	frame: string;
	start: number;
	end: number;
}

let current: TextSelection | null = null;

/** The editor says what is selected, or `null` when it closes. */
export function setTextSelection(selection: TextSelection | null): void {
	current = selection;
}

/** What is selected now in the open editor, when something is. */
export function getTextSelection(): TextSelection | null {
	return current && current.end > current.start ? current : null;
}

/** The characters (code points) before UTF-16 position `index` of `text`. */
export function charsBefore(text: string, index: number): number {
	return [...text.slice(0, index)].length;
}

/** The caret in the open editor when nothing is selected: the frame and the characters before it. */
export interface TextCaret {
	frame: string;
	offset: number;
}

let caret: TextCaret | null = null;
let pending: { frame: string; change: PendingLook } | null = null;

/** A look set at the caret for the words typed next: only these four can be. */
export interface PendingLook {
	bold?: boolean;
	italic?: boolean;
	colour?: string;
}

/** The editor says where its caret is when nothing is selected, or `null`. */
export function setTextCaret(next: TextCaret | null): void {
	caret = next;
}

/** Where the caret is in the open editor, when nothing is selected. */
export function getTextCaret(): TextCaret | null {
	return caret;
}

/** Sets a look for the words typed next at the caret, over any set before. */
export function setPendingLook(frame: string, change: PendingLook): void {
	pending = { frame, change: { ...(pending?.frame === frame ? pending.change : {}), ...change } };
}

/** The look set at the caret for the words typed next in `frame`, if any. */
export function getPendingLook(frame: string): PendingLook | null {
	return pending?.frame === frame ? pending.change : null;
}

/** Forgets the look set at the caret: the caret moved, or the words went in. */
export function clearPendingLook(): void {
	pending = null;
}

/**
 * The paragraphs the open editor's caret or selection touches, for the ribbon's paragraph controls:
 * which words they are, the first and last (counted from the first paragraph of those words), and
 * the look each paragraph of the words sets for itself.
 */
export interface TextParagraphs {
	target: TextTarget;
	/** The frame the editor was opened on, which is the one selected. */
	frame: string;
	from: number;
	to: number;
	/** The looks of the words' paragraphs by place; paragraphs past the end set nothing. */
	looks: readonly ParaLook[];
}

let paragraphs: TextParagraphs | null = null;
const listeners = new Set<() => void>();

const sameScope = (a: TextParagraphs | null, b: TextParagraphs | null) =>
	a === b ||
	(a !== null &&
		b !== null &&
		a.frame === b.frame &&
		a.from === b.from &&
		a.to === b.to &&
		JSON.stringify(a.target) === JSON.stringify(b.target) &&
		JSON.stringify(a.looks) === JSON.stringify(b.looks));

/** The editor says which paragraphs its caret or selection is in, or `null` when it closes. */
export function setTextParagraphs(next: TextParagraphs | null): void {
	if (sameScope(paragraphs, next)) return;
	paragraphs = next;
	listeners.forEach((listen) => listen());
}

/** The paragraphs the open editor's caret or selection is in, when an editor is open. */
export function getTextParagraphs(): TextParagraphs | null {
	return paragraphs;
}

/** Calls `listen` when the paragraphs change; answers how to stop (for `useSyncExternalStore`). */
export function subscribeTextParagraphs(listen: () => void): () => void {
	listeners.add(listen);
	return () => {
		listeners.delete(listen);
	};
}
