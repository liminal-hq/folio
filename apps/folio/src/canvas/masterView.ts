// Editing a master page: it is shown as a one-page spread of its own, and its edits are addressed to it
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { Book } from '@liminal-hq/folio-protocol/generated/Book';
import type { Spread } from '@liminal-hq/folio-protocol/generated/Spread';
import type { Surface } from '@liminal-hq/folio-protocol/generated/Surface';

/** What a shown master's spread id starts with, so it is never taken for one of the book's spreads. */
const PREFIX = 'master:';

/** The master B of page numbers outside is the left-hand page; every other master is a right-hand one. */
const VERSO = 'mst_b';

/**
 * The master `id` as a spread the stage can show: one page, its frames and its guides. A verso is
 * shown as the last page of a book, so its margins are the left-hand ones.
 */
export function masterSpread(book: Book, id: string): Spread | null {
	const master = book.masterPages?.find((m) => m.id === id);
	if (!master) return null;
	return {
		id: `${PREFIX}${id}`,
		kind: id === VERSO ? 'closer' : 'single',
		pages: [{ id: `pg_${id}`, master: null }],
		frames: master.frames,
		guides: master.guides ?? [],
		masterOverrides: [],
	} as Spread;
}

/** Whether `spread` is a master shown by `masterSpread`. */
export function isMasterSpread(spread: Spread): boolean {
	return spread.id.startsWith(PREFIX);
}

/** Where an intent for a frame or guide on `spread` is addressed: the spread, or the master it shows. */
export function surfaceOf(spread: Spread): Surface {
	return isMasterSpread(spread)
		? { master: spread.id.slice(PREFIX.length) }
		: { spread: spread.id };
}

/** The master a page of the spread at `index` uses, when there is one to edit. */
export function masterOfPage(spread: Spread, index = 0): string | null {
	return spread.pages[index]?.master ?? null;
}
