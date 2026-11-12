// Which stretch of words is selected in the open text editor, for the ribbon's bold, italic and font
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { ParaLook } from '@liminal-hq/folio-protocol/generated/ParaLook';
import type { TextPos } from '@liminal-hq/folio-protocol/generated/TextPos';
import type { TextTarget } from '@liminal-hq/folio-protocol/generated/TextTarget';

/**
 * A selected stretch of the open editor's words. `start` and `end` count characters from the
 * start of the words (a line break is one), not UTF-16 units. For the words of a story or a table
 * cell, which the ribbon styles through `StyleText`, `target`, `from` and `to` say the same stretch
 * by paragraph; `frame` is then the frame the editor was opened on (the table, for a cell).
 */
export interface TextSelection {
	frame: string;
	start: number;
	end: number;
	target?: TextTarget;
	from?: TextPos;
	to?: TextPos;
}

let current: TextSelection | null = null;

/** What the ribbon shows of the words at the caret or in the selection, and when it changes. */
export interface TextLookState {
	selection: TextSelection | null;
	caret: TextCaret | null;
	pending: { frame: string; change: PendingLook } | null;
}

let state: TextLookState = { selection: null, caret: null, pending: null };
const lookListeners = new Set<() => void>();

const samePos = (a?: TextPos, b?: TextPos) => a?.para === b?.para && a?.offset === b?.offset;
const sameTarget = (a?: TextTarget, b?: TextTarget) => JSON.stringify(a) === JSON.stringify(b);

function publish(): void {
	const selection = current && current.end > current.start ? current : null;
	if (state.selection === selection && state.caret === caret && state.pending === pending) return;
	state = { selection, caret, pending };
	lookListeners.forEach((listen) => listen());
}

/** The selection, caret and look set at the caret as one value that changes when any does. */
export function getTextLookState(): TextLookState {
	return state;
}

/** Calls `listen` when the selection, the caret or the look set at it changes (for `useSyncExternalStore`). */
export function subscribeTextLook(listen: () => void): () => void {
	lookListeners.add(listen);
	return () => {
		lookListeners.delete(listen);
	};
}

/** The editor says what is selected, or `null` when it closes. */
export function setTextSelection(selection: TextSelection | null): void {
	const same =
		current === selection ||
		(current !== null &&
			selection !== null &&
			current.frame === selection.frame &&
			current.start === selection.start &&
			current.end === selection.end &&
			sameTarget(current.target, selection.target) &&
			samePos(current.from, selection.from) &&
			samePos(current.to, selection.to));
	if (same) return;
	current = selection;
	publish();
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
	/** For the words of a story or a table cell: whose they are and the caret's place in them. */
	target?: TextTarget;
	pos?: TextPos;
}

let caret: TextCaret | null = null;
let pending: { frame: string; change: PendingLook } | null = null;

/** A look set at the caret for the words typed next. */
export interface PendingLook {
	bold?: boolean;
	italic?: boolean;
	underline?: boolean;
	strike?: boolean;
	/** A size in points, and letter spacing in em thousandths (or `clearTracking` for the style's). */
	size?: number;
	tracking?: number;
	clearTracking?: boolean;
	/** A language tag for the words typed next; empty for the book's. */
	lang?: string;
	font?: string;
	colour?: string;
}

/** The editor says where its caret is when nothing is selected, or `null`. */
export function setTextCaret(next: TextCaret | null): void {
	const same =
		caret === next ||
		(caret !== null &&
			next !== null &&
			caret.frame === next.frame &&
			caret.offset === next.offset &&
			sameTarget(caret.target, next.target) &&
			samePos(caret.pos, next.pos));
	if (same) return;
	caret = next;
	publish();
}

/** Where the caret is in the open editor, when nothing is selected. */
export function getTextCaret(): TextCaret | null {
	return caret;
}

/** Sets a look for the words typed next at the caret, over any set before. */
export function setPendingLook(frame: string, change: PendingLook): void {
	pending = { frame, change: { ...(pending?.frame === frame ? pending.change : {}), ...change } };
	publish();
}

/** The look set at the caret for the words typed next in `frame`, if any. */
export function getPendingLook(frame: string): PendingLook | null {
	return pending?.frame === frame ? pending.change : null;
}

/** Forgets the look set at the caret: the caret moved, or the words went in. */
export function clearPendingLook(): void {
	if (pending === null) return;
	pending = null;
	publish();
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
