// The words for each preflight issue: what is wrong, and what to do about it
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { PreflightItem } from '@liminal-hq/folio-protocol/generated/PreflightItem';
import type { PreflightKind } from '@liminal-hq/folio-protocol/generated/PreflightKind';
import { t, type MessageKey } from '../i18n/messages';

const KEYS: Record<PreflightKind, string> = {
	oversetText: 'overset',
	textOutsideMargin: 'textOutside',
	pictureShortOfBleed: 'bleedShort',
	poemLineTooLong: 'poemLong',
	pictureMissing: 'pictureMissing',
	lowResolution: 'lowRes',
	veryLowResolution: 'veryLowRes',
	bleedNeeded: 'bleedNeeded',
	fontMissing: 'fontMissing',
	missingGlyphs: 'missingGlyphs',
	inkOverLimit: 'inkOver',
	swatchMissing: 'swatchMissing',
	richBlackSmallText: 'richBlack',
	picturePlaceholder: 'placeholder',
	altMissing: 'altMissing',
};

/** The kinds said once for the whole book when there are several, as the spec does ("4 pictures are still placeholders"). */
const SAID_ONCE: ReadonlySet<PreflightKind> = new Set(['picturePlaceholder', 'altMissing']);

/** An issue, and how many like it there are when they are said together. */
export interface PreflightRow {
	item: PreflightItem;
	count: number;
	/** Every issue the row stands for, so leaving it alone leaves them all alone. */
	items: PreflightItem[];
}

/** The issues as rows: a kind that is said once becomes one row, at its first place, with a count. */
export function rowsOf(items: readonly PreflightItem[]): PreflightRow[] {
	const rows: PreflightRow[] = [];
	for (const item of items) {
		const same = SAID_ONCE.has(item.kind)
			? rows.find((row) => row.item.kind === item.kind)
			: undefined;
		if (same) {
			same.count += 1;
			same.items.push(item);
		} else rows.push({ item, count: 1, items: [item] });
	}
	return rows;
}

/** What is wrong and the way to fix it, for `item`, whose spread is called `where` on the shelf. */
export function describe(
	item: PreflightItem,
	where: string,
	count = 1,
): { problem: string; fix: string } {
	const key = KEYS[item.kind];
	const values = { where, family: item.family ?? '', chars: item.chars ?? '', count };
	const many = count > 1 && SAID_ONCE.has(item.kind);
	return {
		problem: t(`preflight.${key}${many ? '.many' : ''}` as MessageKey, values),
		fix: t(`preflight.${key}.fix` as MessageKey, values),
	};
}
