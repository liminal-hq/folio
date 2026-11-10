// The words being edited as paragraphs of text, and the same edit the layout makes, in plain strings
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { InlineContent } from '@liminal-hq/folio-protocol/generated/InlineContent';
import type { TextPos } from '@liminal-hq/folio-protocol/generated/TextPos';
import { cpLength, cpSlice } from './units';

/** The paragraphs of a frame's own words: one for each line break. There is always at least one. */
export function paragraphsOf(content: InlineContent | null | undefined): string[] {
	const text = (content?.inline ?? []).map((r) => r.text).join('');
	return text.split('\n');
}

/** The words as one string, a line break between paragraphs. */
export function joined(texts: readonly string[]): string {
	return texts.join('\n');
}

/**
 * The words once `text` replaces `from..to`: the same edit Rust makes, so the editor knows the
 * words it will be shown before the book says. `text` may hold line breaks.
 */
export function replaceInTexts(
	texts: readonly string[],
	from: TextPos,
	to: TextPos,
	text: string,
): string[] {
	const head = cpSlice(texts[from.para] ?? '', 0, from.offset);
	const tail = cpSlice(texts[to.para] ?? '', to.offset);
	const middle = (head + text + tail).split('\n');
	return [...texts.slice(0, from.para), ...middle, ...texts.slice(to.para + 1)];
}

/** Where the caret goes once `text` has gone in at `from`. */
export function caretAfterText(from: TextPos, text: string): TextPos {
	const pieces = text.split('\n');
	const last = pieces[pieces.length - 1]!;
	return pieces.length === 1
		? { para: from.para, offset: from.offset + cpLength(last) }
		: { para: from.para + pieces.length - 1, offset: cpLength(last) };
}

/** The words from `from` to `to`, a line break between paragraphs. */
export function textBetween(texts: readonly string[], from: TextPos, to: TextPos): string {
	if (from.para === to.para) return cpSlice(texts[from.para] ?? '', from.offset, to.offset);
	const parts = [cpSlice(texts[from.para] ?? '', from.offset)];
	for (let p = from.para + 1; p < to.para; p += 1) parts.push(texts[p] ?? '');
	parts.push(cpSlice(texts[to.para] ?? '', 0, to.offset));
	return parts.join('\n');
}

/** A position in the words as a character count from their start, line breaks counted (what `StyleRange` uses). */
export function flatOffset(texts: readonly string[], pos: TextPos): number {
	let n = 0;
	for (let p = 0; p < pos.para; p += 1) n += cpLength(texts[p] ?? '') + 1;
	return n + pos.offset;
}

/** Puts two positions in order. */
export function ordered(a: TextPos, b: TextPos): [TextPos, TextPos] {
	return a.para < b.para || (a.para === b.para && a.offset <= b.offset) ? [a, b] : [b, a];
}

/** Whether two places are the same. */
export function samePos(a: TextPos, b: TextPos): boolean {
	return a.para === b.para && a.offset === b.offset;
}

/** The place after the last word. */
export function endOf(texts: readonly string[]): TextPos {
	const para = Math.max(0, texts.length - 1);
	return { para, offset: cpLength(texts[para] ?? '') };
}

/** A place pulled inside the words. */
export function clampPos(texts: readonly string[], pos: TextPos): TextPos {
	const para = Math.min(Math.max(0, pos.para), Math.max(0, texts.length - 1));
	return { para, offset: Math.min(Math.max(0, pos.offset), cpLength(texts[para] ?? '')) };
}
