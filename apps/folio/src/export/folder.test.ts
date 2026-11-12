// Tests for keeping an export folder relative to its book
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { describe, expect, it } from 'vitest';
import { absoluteFor, relativeToBook } from './folder';

const book = '/home/mom/Books/Pip/Pip.folio';

describe('an export folder kept with a book', () => {
	it('is relative when it is inside the book’s own folder', () => {
		expect(relativeToBook('/home/mom/Books/Pip/for-print', book)).toBe('for-print');
		expect(relativeToBook('/home/mom/Books/Pip/a/b/', book)).toBe('a/b');
		expect(relativeToBook('/home/mom/Books/Pip', book)).toBe('.');
	});

	it('is absolute when it is anywhere else, or the book has not been saved', () => {
		expect(relativeToBook('/home/mom/Share', book)).toBe('/home/mom/Share');
		expect(relativeToBook('/home/mom/Books/Pipette', book)).toBe('/home/mom/Books/Pipette');
		expect(relativeToBook('/tmp/out', null)).toBe('/tmp/out');
	});

	it('reads Windows paths, whatever the case or slash', () => {
		const file = 'C:\\Users\\Mom\\Books\\Pip\\Pip.folio';
		expect(relativeToBook('c:\\users\\mom\\books\\pip\\print', file)).toBe('print');
		expect(relativeToBook('D:\\Share', file)).toBe('D:\\Share');
	});

	it('becomes a full path again from the book’s folder', () => {
		expect(absoluteFor('for-print', book)).toBe('/home/mom/Books/Pip/for-print');
		expect(absoluteFor('.', book)).toBe('/home/mom/Books/Pip');
		expect(absoluteFor('/home/mom/Share', book)).toBe('/home/mom/Share');
		expect(absoluteFor('for-print', null)).toBe('for-print');
	});
});
