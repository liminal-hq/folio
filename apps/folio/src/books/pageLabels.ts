// What a book's sections call each page: its number as written, and the name of its section
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { Spread } from '@liminal-hq/folio-protocol/generated/Spread';

export interface PageLabel {
	/** "7", "vii", or empty when the page's section shows no numbers. */
	label: string;
	/** The name of the section the page is in; empty before the first section. */
	section: string;
}

const ROMAN: [number, string][] = [
	[1000, 'm'],
	[900, 'cm'],
	[500, 'd'],
	[400, 'cd'],
	[100, 'c'],
	[90, 'xc'],
	[50, 'l'],
	[40, 'xl'],
	[10, 'x'],
	[9, 'ix'],
	[5, 'v'],
	[4, 'iv'],
	[1, 'i'],
];

/** `n` in lower-case Roman numerals for 1 to 3999; larger numbers stay Arabic. */
export function roman(n: number): string {
	if (n < 1 || n > 3999) return String(n);
	let rest = n;
	let out = '';
	for (const [value, symbol] of ROMAN) {
		while (rest >= value) {
			out += symbol;
			rest -= value;
		}
	}
	return out;
}

/**
 * The label of every page of every spread. The cover is not counted and has none. The twin of
 * `page_labels` in `folio-model`, and both are tested against `labelCases.json`.
 */
export function pageLabels(spreads: Spread[]): PageLabel[][] {
	let count = 0;
	let current: NonNullable<Spread['section']> | null = null;
	return spreads.map((spread) => {
		if (spread.kind === 'cover') return spread.pages.map(() => ({ label: '', section: '' }));
		if (spread.section) {
			current = spread.section;
			if (spread.section.start != null) count = spread.section.start - 1;
		}
		return spread.pages.map(() => {
			count += 1;
			const numbering = current?.numbering ?? 'arabic';
			return {
				label: numbering === 'arabic' ? String(count) : numbering === 'roman' ? roman(count) : '',
				section: current?.name ?? '',
			};
		});
	});
}
