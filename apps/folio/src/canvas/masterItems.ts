// The master page items a page shows, and whether each is hidden on it
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { Book } from '@liminal-hq/folio-protocol/generated/Book';
import type { Frame } from '@liminal-hq/folio-protocol/generated/Frame';
import type { Spread } from '@liminal-hq/folio-protocol/generated/Spread';
import { t } from '../i18n/messages';
import { spreadGeometry } from './geometry';

export interface MasterItem {
	page: string;
	frame: string;
	/** What to call it in a menu: "the page number", or the start of its words. */
	label: string;
	hidden: boolean;
}

const WORDS_SHOWN = 24;

/** What a master frame is called to a person: its words, with the page number token spelled out. */
function nameOf(frame: Frame, master: string): string {
	if (frame.type !== 'text') return master;
	const words = (frame.content?.inline ?? []).map((r) => r.text).join('');
	if (words.trim() === '{page}') return t('master.item.pageNumber');
	const trimmed = words.trim();
	if (trimmed === '') return master;
	return trimmed.length > WORDS_SHOWN ? `${trimmed.slice(0, WORDS_SHOWN)}…` : trimmed;
}

/** The index of the page of a spread that a point (in points from its first page's top-left) is over. */
export function pageAt(book: Book, spread: Spread, point: { x: number }): number {
	const { pages } = spreadGeometry(book, spread);
	const found = pages.findIndex((p) => point.x >= p.trim.x && point.x < p.trim.x + p.trim.w);
	if (found !== -1) return found;
	return point.x < 0 ? 0 : pages.length - 1;
}

/** The master items of the page under `point`, each with whether this page hides it. */
export function masterItemsAt(book: Book, spread: Spread, point: { x: number }): MasterItem[] {
	const page = spread.pages[pageAt(book, spread, point)];
	const master = page?.master ? book.masterPages?.find((m) => m.id === page.master) : undefined;
	if (!page || !master) return [];
	const hidden = new Set(spread.masterOverrides.find((o) => o.pageId === page.id)?.hide ?? []);
	return master.frames.map((frame) => ({
		page: page.id,
		frame: frame.id,
		label: nameOf(frame, master.name),
		hidden: hidden.has(frame.id),
	}));
}
