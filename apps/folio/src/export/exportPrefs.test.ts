// Tests for putting export choices together
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { describe, expect, it } from 'vitest';
import { effectivePdf, emptyPrefs, isBlank, kindOf, overlay, withPdf } from './exportPrefs';

describe('export choices', () => {
	it('knows which kind of PDF a target makes', () => {
		expect(kindOf('sharingPdf')).toBe('sharing');
		expect(kindOf('kdpCover')).toBe('kdp');
		expect(kindOf('printShopPdf')).toBe('printShop');
		expect(kindOf('kindleEpub')).toBeNull();
	});

	it('lets the higher layer win and keeps the rest', () => {
		const user = withPdf(emptyPrefs(), 'sharing', { preset: 'small', jpegQuality: 50 });
		const book = withPdf(emptyPrefs(), 'sharing', { preset: 'high' });
		expect(effectivePdf('sharing', user, book)).toEqual({ preset: 'high', jpegQuality: 50 });
		expect(overlay(user, book).pdf.sharing).toEqual({ preset: 'high', jpegQuality: 50 });
	});

	it('takes a choice back with undefined', () => {
		const chosen = withPdf(emptyPrefs(), 'kdp', { preset: 'original' });
		expect(isBlank(withPdf(chosen, 'kdp', { preset: undefined }))).toBe(true);
		expect(isBlank(chosen)).toBe(false);
	});

	it('lays a folder over another', () => {
		const book = { ...emptyPrefs(), folder: 'for-print' };
		expect(overlay(book, emptyPrefs()).folder).toBe('for-print');
		expect(overlay(book, { ...emptyPrefs(), folder: 'out' }).folder).toBe('out');
	});
});
