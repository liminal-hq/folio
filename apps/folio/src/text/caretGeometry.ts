// Where a caret is, what a selection covers and what a click hits, from the lines Rust laid out
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { FrameLayout } from '@liminal-hq/folio-protocol/generated/FrameLayout';
import type { LayoutLine } from '@liminal-hq/folio-protocol/generated/LayoutLine';
import type { TextPos } from '@liminal-hq/folio-protocol/generated/TextPos';
import { cpLength } from './units';

/** A rectangle in points from the frame's top-left, where the lines are placed. */
export interface Box {
	x: number;
	y: number;
	w: number;
	h: number;
}

/** The place of one code point boundary on a line. */
interface Stop {
	offset: number;
	x: number;
}

/** The runs of a line that hold words of the paragraph (a hyphen drawn at a line's end does not). */
function textRuns(line: LayoutLine) {
	return line.runs.filter((r) => !r.hyphen);
}

/**
 * Every code point boundary of the words a line shows, with its x from the frame's left edge, in
 * order. The spaces between two runs are given evenly spaced stops across the gap.
 */
export function lineStops(line: LayoutLine): Stop[] {
	const runs = textRuns(line);
	const out: Stop[] = [];
	let last: { end: number; right: number } | null = null;
	for (const run of runs) {
		const runStops = run.stops ?? [];
		const n = Math.max(0, runStops.length - 1);
		if (last && run.start > last.end) {
			const gap = run.start - last.end;
			const from = last.right;
			const to = line.x + run.x;
			for (let k = 1; k < gap; k += 1) {
				out.push({ offset: last.end + k, x: from + ((to - from) * k) / gap });
			}
		}
		for (let i = 0; i <= n; i += 1) {
			// A boundary two runs share keeps the first.
			if (out.length > 0 && out[out.length - 1]!.offset === run.start + i) continue;
			out.push({ offset: run.start + i, x: line.x + run.x + (runStops[i] ?? run.width) });
		}
		last = { end: run.start + n, right: line.x + run.x + run.width };
	}
	return out;
}

/** The first and last offset of the words a line shows; a blank line has its own start twice. */
export function textSpan(line: LayoutLine): { start: number; end: number } {
	const stops = lineStops(line);
	if (stops.length === 0) return { start: line.start, end: line.start };
	return { start: stops[0]!.offset, end: stops[stops.length - 1]!.offset };
}

/** The x of a boundary of a line, clamped to the words it shows. */
export function xOnLine(line: LayoutLine, offset: number): number {
	const stops = lineStops(line);
	if (stops.length === 0) return line.x;
	if (offset <= stops[0]!.offset) return stops[0]!.x;
	for (const stop of stops) if (stop.offset >= offset) return stop.x;
	return stops[stops.length - 1]!.x;
}

/**
 * The line a position is on. At the boundary between two lines of a paragraph the position is at
 * the start of the later one (`affinity: 'after'`) or the end of the earlier (`'before'`).
 */
export function lineOf(
	layout: FrameLayout,
	pos: TextPos,
	affinity: 'before' | 'after' = 'after',
): LayoutLine | undefined {
	const own = layout.lines.filter((l) => l.paragraph === pos.para);
	if (own.length === 0) return undefined;
	const inside = own.filter((l) => pos.offset >= l.start && pos.offset < l.end);
	const closing = own.find((l) => l.end === pos.offset && l.start < l.end);
	if (affinity === 'before' && closing) return closing;
	return inside[0] ?? closing ?? own.find((l) => l.start === pos.offset) ?? own[own.length - 1];
}

/** Where the caret goes: a thin box as tall as the line, in the frame's points. */
export function caretRect(
	layout: FrameLayout,
	pos: TextPos,
	affinity: 'before' | 'after' = 'after',
): Box | undefined {
	const line = lineOf(layout, pos, affinity);
	if (!line) return undefined;
	return { x: xOnLine(line, pos.offset), y: line.top, w: 0, h: line.height };
}

const compare = (a: TextPos, b: TextPos) => a.para - b.para || a.offset - b.offset;

/** The boxes a selection from `from` to `to` covers, one for each line it touches. */
export function selectionRects(layout: FrameLayout, from: TextPos, to: TextPos): Box[] {
	const [a, b] = compare(from, to) <= 0 ? [from, to] : [to, from];
	if (compare(a, b) === 0) return [];
	const out: Box[] = [];
	for (const line of layout.lines) {
		const p = line.paragraph ?? -1;
		if (p < a.para || p > b.para) continue;
		const span = textSpan(line);
		const lo = p === a.para ? Math.max(a.offset, span.start) : span.start;
		const hi = p === b.para ? Math.min(b.offset, span.end) : span.end;
		const continues = p < b.para && line.end >= span.end && isLastOfParagraph(layout, line);
		if (lo >= hi && !continues) continue;
		const x0 = xOnLine(line, lo);
		let x1 = xOnLine(line, hi);
		// A line break inside the selection is shown as a little extra width after the last word.
		if (continues) x1 += Math.max(2, line.height * 0.25);
		if (x1 <= x0) continue;
		out.push({ x: x0, y: line.top, w: x1 - x0, h: line.height });
	}
	return out;
}

function isLastOfParagraph(layout: FrameLayout, line: LayoutLine): boolean {
	const own = layout.lines.filter((l) => l.paragraph === line.paragraph);
	return own[own.length - 1] === line;
}

/** The line a y falls on, else the nearest one. */
function lineAtY(layout: FrameLayout, y: number): LayoutLine | undefined {
	let best: LayoutLine | undefined;
	let bestGap = Infinity;
	for (const line of layout.lines) {
		if (line.paragraph == null) continue;
		const gap =
			y < line.top ? line.top - y : y > line.top + line.height ? y - line.top - line.height : 0;
		if (gap < bestGap) {
			best = line;
			bestGap = gap;
		}
	}
	return best;
}

/**
 * The position under a point, in the frame's points. `snap` moves an offset to a grapheme boundary
 * of the paragraph's own words, which only the editor has.
 */
export function hitTest(
	layout: FrameLayout,
	x: number,
	y: number,
	snap?: (para: number, offset: number) => number,
): TextPos | undefined {
	const line = lineAtY(layout, y);
	return line ? hitOnLine(line, x, snap) : undefined;
}

/** The position on one line nearest to `x`. */
export function hitOnLine(
	line: LayoutLine,
	x: number,
	snap?: (para: number, offset: number) => number,
): TextPos | undefined {
	if (line.paragraph == null) return undefined;
	const stops = lineStops(line);
	let offset = line.start;
	if (stops.length > 0) {
		let best = stops[0]!;
		for (const stop of stops) {
			if (Math.abs(stop.x - x) < Math.abs(best.x - x)) best = stop;
		}
		offset = best.offset;
	}
	return { para: line.paragraph, offset: snap ? snap(line.paragraph, offset) : offset };
}

/** A misspelt word: the paragraph it is in and where it starts and ends, in code points. */
export interface Misspelt {
	para: number;
	start: number;
	end: number;
}

/** Every misspelt word the layout marks, in reading order; a word broken across lines is one. */
export function misspellings(layout: FrameLayout): Misspelt[] {
	const out: Misspelt[] = [];
	for (const line of layout.lines) {
		if (line.paragraph == null) continue;
		for (const run of line.runs) {
			for (const [from, to] of run.miss ?? []) {
				const word = { para: line.paragraph, start: run.start + from, end: run.start + to };
				const last = out[out.length - 1];
				if (last && last.para === word.para && last.end === word.start) last.end = word.end;
				else out.push(word);
			}
		}
	}
	return out;
}

/** The length of a paragraph as the layout knows it: where its last line ends. */
export function paragraphLength(layout: FrameLayout, para: number): number {
	const own = layout.lines.filter((l) => l.paragraph === para);
	return own.length === 0 ? 0 : own[own.length - 1]!.end;
}

/** The words of a layout's paragraph as far as it shows them, for tests and tools. */
export function shownText(line: LayoutLine): string {
	return textRuns(line)
		.map((r) => r.text)
		.join('');
}

export { cpLength };
