// Moving the caret: by grapheme and word through the words, by line through the layout
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { FrameLayout } from '@liminal-hq/folio-protocol/generated/FrameLayout';
import type { TextPos } from '@liminal-hq/folio-protocol/generated/TextPos';
import { hitTest, lineOf, textSpan, xOnLine } from './caretGeometry';
import { boundaries, cpLength } from './units';

/** The words being edited, a string for each paragraph. */
export type Paragraphs = readonly string[];

/** Left (`-1`) or right (`1`) by a grapheme, over a paragraph break when the end is reached. */
export function moveGrapheme(texts: Paragraphs, pos: TextPos, dir: -1 | 1): TextPos {
	const text = texts[pos.para] ?? '';
	const marks = boundaries(text, 'grapheme');
	if (dir < 0) {
		const before = [...marks].reverse().find((m) => m < pos.offset);
		if (before !== undefined) return { para: pos.para, offset: before };
		return pos.para > 0 ? { para: pos.para - 1, offset: cpLength(texts[pos.para - 1] ?? '') } : pos;
	}
	const after = marks.find((m) => m > pos.offset);
	if (after !== undefined) return { para: pos.para, offset: after };
	return pos.para + 1 < texts.length ? { para: pos.para + 1, offset: 0 } : pos;
}

/** Where a word begins or ends: the next one to the right, the start of this or the last to the left. */
export function moveWord(texts: Paragraphs, pos: TextPos, dir: -1 | 1): TextPos {
	const text = texts[pos.para] ?? '';
	const words = wordStops(text);
	if (dir < 0) {
		const before = [...words.starts].reverse().find((m) => m < pos.offset);
		if (before !== undefined) return { para: pos.para, offset: before };
		return pos.para > 0
			? moveWord(texts, { para: pos.para - 1, offset: cpLength(texts[pos.para - 1] ?? '') }, -1)
			: pos;
	}
	const after = words.ends.find((m) => m > pos.offset);
	if (after !== undefined) return { para: pos.para, offset: after };
	return pos.para + 1 < texts.length ? { para: pos.para + 1, offset: 0 } : pos;
}

/** Where words start and end, spaces and punctuation skipped over, with the paragraph's ends as stops. */
function wordStops(text: string): { starts: number[]; ends: number[] } {
	const marks = boundaries(text, 'word');
	const total = cpLength(text);
	const chars = [...text];
	const starts = new Set<number>([0]);
	const ends = new Set<number>([total]);
	for (let i = 0; i < marks.length - 1; i += 1) {
		const from = marks[i]!;
		const to = marks[i + 1]!;
		// Only stretches that hold a letter or digit are words.
		if (/[\p{L}\p{N}]/u.test(chars.slice(from, to).join(''))) {
			starts.add(from);
			ends.add(to);
		}
	}
	const sorted = (set: Set<number>) => [...set].sort((a, b) => a - b);
	return { starts: sorted(starts), ends: sorted(ends) };
}

/** The range of the word at `offset` of a paragraph, for a double click; a space or mark selects itself. */
export function wordAt(text: string, offset: number): { start: number; end: number } {
	const marks = boundaries(text, 'word');
	for (let i = 0; i < marks.length - 1; i += 1) {
		if (offset >= marks[i]! && offset < marks[i + 1]!)
			return { start: marks[i]!, end: marks[i + 1]! };
	}
	const total = cpLength(text);
	const last = marks[marks.length - 2] ?? total;
	return { start: Math.min(last, total), end: total };
}

/** Home or End: the start or end of the words on the caret's line. */
export function moveLineEdge(layout: FrameLayout, pos: TextPos, edge: 'home' | 'end'): TextPos {
	const line = lineOf(layout, pos, edge === 'end' ? 'before' : 'after');
	if (!line) return pos;
	const span = textSpan(line);
	return { para: pos.para, offset: edge === 'home' ? span.start : span.end };
}

/**
 * Up or down one line, keeping to `goalX` (the x the caret was at before it went up or down, so a
 * run of presses does not drift). At the first or last line it goes to the start or end of the text.
 */
export function moveVertical(
	layout: FrameLayout,
	pos: TextPos,
	dir: -1 | 1,
	goalX?: number,
): { pos: TextPos; goalX: number } {
	const line = lineOf(layout, pos);
	if (!line) return { pos, goalX: goalX ?? 0 };
	const x = goalX ?? xOnLine(line, pos.offset);
	const lines = layout.lines.filter((l) => l.paragraph != null);
	const at = lines.indexOf(line);
	const target = lines[at + dir];
	if (!target) {
		const span = textSpan(line);
		return {
			pos: { para: line.paragraph ?? pos.para, offset: dir < 0 ? span.start : span.end },
			goalX: x,
		};
	}
	const hit = hitTest(layout, x, target.top + target.height / 2);
	return { pos: hit ?? pos, goalX: x };
}

/** Whether `a` comes before `b`. */
export function before(a: TextPos, b: TextPos): boolean {
	return a.para < b.para || (a.para === b.para && a.offset < b.offset);
}
