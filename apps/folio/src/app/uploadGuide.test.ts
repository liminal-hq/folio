// Tests for telling where each exported file goes
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { describe, expect, it } from 'vitest';
import { whereItGoes } from './uploadGuide';

describe('whereItGoes', () => {
	it('tells the two fixed-layout files apart', () => {
		expect(whereItGoes('pip-fixed.epub')).toBe('export.where.fixedEpub');
		expect(whereItGoes('pip-fixed.fxl.kepub.epub')).toBe('export.where.fixedKobo');
	});

	it('tells the Kobo upload from the Kobo copy for a reader', () => {
		expect(whereItGoes('pip.epub')).toBe('export.where.koboEpub');
		expect(whereItGoes('pip.kepub.epub')).toBe('export.where.kepub');
	});

	it('knows the Kindle book, the cover picture and the paperback files', () => {
		expect(whereItGoes('pip-kindle.epub')).toBe('export.where.kindleEpub');
		expect(whereItGoes('pip-ebook-cover.jpg')).toBe('export.where.ebookCover');
		expect(whereItGoes('pip-paperback-interior.pdf')).toBe('export.where.kdpInterior');
		expect(whereItGoes('pip-paperback-cover.pdf')).toBe('export.where.kdpCover');
		expect(whereItGoes('pip-print-pdfx4.pdf')).toBe('export.where.printShop');
		expect(whereItGoes('pip-share.pdf')).toBe('export.where.sharing');
	});

	it('says nothing of a file it did not name', () => {
		expect(whereItGoes('notes.txt')).toBeNull();
	});
});
