// Tests for the page menu's rows
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { expectEveryItemHasIcon } from '@liminal-hq/folio-chrome/ContextMenu/expectEveryItemHasIcon';
import { describe, expect, it } from 'vitest';
import { DEFAULT_MARKS } from './marksModel';
import { pageMenuItems } from './pageMenu';

const items = [
	{ page: 'p1', frame: 'n1', label: 'the page number', hidden: false },
	{ page: 'p1', frame: 'n2', label: 'Footer', hidden: true },
];

describe('pageMenuItems', () => {
	it('gives the master item rows an icon', () => {
		expectEveryItemHasIcon(
			pageMenuItems(DEFAULT_MARKS, true, items).filter((r) => String(r.id).includes('Master:')),
		);
	});

	it('offers to hide a master item the page shows and to show one it hides', () => {
		const rows = pageMenuItems(DEFAULT_MARKS, true, items);
		expect(rows.find((r) => r.id === 'hideMaster:p1:n1')).toMatchObject({
			label: 'Hide the page number on this page',
		});
		expect(rows.find((r) => r.id === 'showMaster:p1:n2')).toMatchObject({
			label: 'Show Footer on this page',
		});
	});

	it('adds nothing for a page with no master items', () => {
		const rows = pageMenuItems(DEFAULT_MARKS, true);
		expect(rows.some((r) => String(r.id).includes('Master'))).toBe(false);
		expect(rows.at(-1)).not.toMatchObject({ type: 'separator' });
	});
});
