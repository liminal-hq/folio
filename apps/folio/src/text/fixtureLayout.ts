// A layout built by hand for tests: fixed-width letters, one run for each word, lines as given
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { FrameLayout } from '@liminal-hq/folio-protocol/generated/FrameLayout';
import type { LayoutLine } from '@liminal-hq/folio-protocol/generated/LayoutLine';
import type { LayoutRun } from '@liminal-hq/folio-protocol/generated/LayoutRun';

/** The width of every code point, in points. */
export const CHAR_W = 10;
/** The height of every line, in points. */
export const LINE_H = 20;

/**
 * Lays out paragraphs the way the layout does: each paragraph is a list of lines, each line a
 * string of words with single spaces between them. Spaces that end a line are kept in the line's
 * `end`, as the layout tiles a paragraph, and are not drawn.
 */
export function layoutOf(paragraphs: string[][]): FrameLayout {
	const lines: LayoutLine[] = [];
	let top = 0;
	paragraphs.forEach((para, p) => {
		let at = 0;
		para.forEach((text, k) => {
			const start = at;
			const runs: LayoutRun[] = [];
			let offset = start;
			let x = 0;
			for (const word of text.split(' ')) {
				if (word.length === 0) {
					offset += 1;
					x += CHAR_W;
					continue;
				}
				const n = Array.from(word).length;
				runs.push({
					text: word,
					x,
					width: n * CHAR_W,
					family: 'Test',
					size: 12,
					weight: 400,
					italic: false,
					tracking: 0,
					hyphen: false,
					start: offset,
					stops: Array.from({ length: n + 1 }, (_, i) => i * CHAR_W),
				});
				offset += n + 1;
				x += (n + 1) * CHAR_W;
			}
			// The line ends where the next begins: after the space it was broken at.
			const last = k === para.length - 1;
			const shown = Array.from(text).length;
			at = start + shown + (last ? 0 : 1);
			lines.push({
				x: 0,
				top,
				baseline: top + 15,
				width: shown * CHAR_W,
				height: LINE_H,
				runs,
				paragraph: p,
				start: k === 0 ? 0 : start,
				end: at,
			});
			top += LINE_H;
		});
	});
	return { frame: 'f', lines, overset: false };
}
