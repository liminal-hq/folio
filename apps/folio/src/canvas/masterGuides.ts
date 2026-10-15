// The ruler guides a spread's pages get from their masters, and whether a spread's own can go on every page
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { Book } from '@liminal-hq/folio-protocol/generated/Book';
import type { Guide } from '@liminal-hq/folio-protocol/generated/Guide';
import type { Spread } from '@liminal-hq/folio-protocol/generated/Spread';
import { isMasterSpread } from './masterView';

/** A guide that comes from a master, placed on the spread, and the page it shows on. */
export interface MasterGuide {
	axis: 'x' | 'y';
	/** From the first page's top-left, in points, like a spread's own guides. */
	pos: number;
	/** The page of the spread it is on, from 0. */
	page: number;
	master: string;
}

/**
 * The guides of the masters a spread's pages use, where each lands on the spread: a vertical one is
 * a position on its page, so it moves by the pages before it. A master shown for editing has its
 * guides as its own, so it gets none here.
 */
export function masterGuidesOn(book: Book, spread: Spread): MasterGuide[] {
	if (isMasterSpread(spread)) return [];
	const out: MasterGuide[] = [];
	spread.pages.forEach((page, index) => {
		const master = page.master ? book.masterPages?.find((m) => m.id === page.master) : undefined;
		for (const guide of master?.guides ?? []) {
			out.push({
				axis: guide.axis,
				pos: guide.axis === 'x' ? guide.pos + index * book.trim.w : guide.pos,
				page: index,
				master: master!.id,
			});
		}
	});
	return out;
}

/**
 * Whether a guide of `spread` can go on every page: not on the cover, and a vertical one over a page
 * and not in the gutter or off the spread. The session says the same.
 */
export function canPromote(book: Book, spread: Spread, guide: Guide): boolean {
	if (spread.kind === 'cover' || isMasterSpread(spread) || guide.locked) return false;
	if (guide.axis === 'y') return true;
	const w = book.trim.w;
	const pages = spread.pages.length;
	if (guide.pos < 0 || guide.pos > w * pages) return false;
	for (let k = 1; k < pages; k += 1) if (guide.pos === k * w) return false;
	return true;
}
