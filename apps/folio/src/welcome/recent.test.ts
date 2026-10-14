// Tests for the recents list's entries
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { describe, expect, it } from 'vitest';
import { recentEntry } from './recent';

describe('recentEntry', () => {
	it('splits a path on either kind of separator and drops the extension', () => {
		expect(recentEntry('/home/mom/Books/Pip/Pip.folio')).toEqual({
			path: '/home/mom/Books/Pip/Pip.folio',
			name: 'Pip',
			folder: '/home/mom/Books/Pip',
		});
		expect(recentEntry('C:\\Users\\Mom\\Documents\\Books\\Pip\\Pip.FOLIO')).toMatchObject({
			name: 'Pip',
			folder: 'C:\\Users\\Mom\\Documents\\Books\\Pip',
		});
	});

	it('copes with a bare file name', () => {
		expect(recentEntry('Pip.folio')).toEqual({ path: 'Pip.folio', name: 'Pip', folder: '' });
	});
});
