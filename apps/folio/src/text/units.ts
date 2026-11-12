// Counting text in code points, the unit the wire uses, and finding grapheme and word boundaries
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

/** The number of code points in `text`. */
export function cpLength(text: string): number {
	let n = 0;
	for (const _ of text) n += 1;
	return n;
}

/** The UTF-16 index of code point `cp` of `text`; past the end it is the text's length. */
export function cpToUtf16(text: string, cp: number): number {
	if (cp <= 0) return 0;
	let index = 0;
	let seen = 0;
	for (const ch of text) {
		if (seen === cp) return index;
		index += ch.length;
		seen += 1;
	}
	return index;
}

/** The code point at UTF-16 index `index` of `text`. */
export function utf16ToCp(text: string, index: number): number {
	return cpLength(text.slice(0, Math.max(0, index)));
}

/** The code points `from..to` of `text`. */
export function cpSlice(text: string, from: number, to?: number): string {
	const start = cpToUtf16(text, from);
	return to === undefined ? text.slice(start) : text.slice(start, cpToUtf16(text, to));
}

type Segmenter = {
	segment(text: string): Iterable<{ index: number; segment: string; isWordLike?: boolean }>;
};

const segmenters: Record<'grapheme' | 'word', Segmenter | null | undefined> = {
	grapheme: undefined,
	word: undefined,
};

function segmenter(granularity: 'grapheme' | 'word'): Segmenter | null {
	if (segmenters[granularity] === undefined) {
		const Intl_ = Intl as unknown as {
			Segmenter?: new (locale: string | undefined, options: { granularity: string }) => Segmenter;
		};
		segmenters[granularity] = Intl_.Segmenter
			? new Intl_.Segmenter(undefined, { granularity })
			: null;
	}
	return segmenters[granularity] ?? null;
}

/**
 * The code point offsets at which a grapheme (or a word) starts or ends, from 0 to the text's
 * length, in order. Without `Intl.Segmenter` every code point is a boundary.
 */
export function boundaries(text: string, granularity: 'grapheme' | 'word'): number[] {
	const total = cpLength(text);
	const seg = segmenter(granularity);
	if (!seg) return Array.from({ length: total + 1 }, (_, i) => i);
	const out: number[] = [];
	let cp = 0;
	for (const part of seg.segment(text)) {
		out.push(cp);
		cp += cpLength(part.segment);
	}
	out.push(total);
	return out;
}

/** The grapheme boundary nearest `cp`, the one before it on a tie (`prefer` says otherwise). */
export function snapToGrapheme(
	text: string,
	cp: number,
	prefer: 'before' | 'after' = 'before',
): number {
	const marks = boundaries(text, 'grapheme');
	let best = marks[0] ?? 0;
	for (const mark of marks) {
		const d = Math.abs(mark - cp);
		const bd = Math.abs(best - cp);
		if (d < bd || (d === bd && prefer === 'after')) best = mark;
		if (mark > cp) break;
	}
	return best;
}
