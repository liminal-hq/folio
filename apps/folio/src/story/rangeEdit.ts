// A stretch of a story's paragraphs edited as one box of text, and the intents that make it so
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { Intent } from '@liminal-hq/folio-protocol/generated/Intent';
import type { StoryPara } from '@liminal-hq/folio-protocol/generated/StoryPara';

/** The words of a stretch of paragraphs in one box: a line for each paragraph. */
export function boxOf(paragraphs: readonly StoryPara[], from: number, count: number): string {
	return paragraphs
		.slice(from, from + count)
		.map((p) => p.inline.map((r) => r.text).join(''))
		.join('\n');
}

/** The paragraphs a box of text holds: a new one wherever there is a new line. */
export function linesOf(text: string): string[] {
	return text.split(/\r?\n/);
}

/**
 * The intents that turn the paragraphs `before` (which start at `from` in the story `flow`) into
 * `after`. Paragraphs that kept their number of lines are changed in place, so their look is kept;
 * where lines were added or taken away the stretch that differs is replaced by plain paragraphs in
 * the style of the ones they stand in for (`styles` has the style of each of `before`).
 */
export function rangeIntents(
	flow: string,
	from: number,
	before: readonly string[],
	after: readonly string[],
	styles: readonly (string | null | undefined)[] = [],
): Intent[] {
	if (before.length === after.length) {
		const out: Intent[] = [];
		after.forEach((text, i) => {
			if (text !== before[i]) out.push({ type: 'SetParagraphText', flow, para: from + i, text });
		});
		return out;
	}
	let head = 0;
	while (head < before.length && head < after.length && before[head] === after[head]) head += 1;
	let tail = 0;
	while (
		tail < before.length - head &&
		tail < after.length - head &&
		before[before.length - 1 - tail] === after[after.length - 1 - tail]
	)
		tail += 1;
	const gone = before.length - head - tail;
	const made = after.slice(head, after.length - tail);
	const styleAt = (i: number) => styles[Math.min(head + i, styles.length - 1)] ?? null;
	return [
		{
			type: 'SpliceStory',
			flow,
			at: from + head,
			remove: gone,
			insert: made.map((text, i) => ({
				style: styleAt(i),
				inline: [{ text, style: null, bold: false, italic: false }],
			})),
		},
	];
}
