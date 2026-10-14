// Tests for the book commands' keys
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { describe, expect, it } from 'vitest';
import { commandForKey } from './useBookKeys';

const key = (k: string, extra: Partial<Parameters<typeof commandForKey>[0]> = {}) =>
	commandForKey({
		key: k,
		ctrlKey: true,
		metaKey: false,
		shiftKey: false,
		altKey: false,
		...extra,
	});

describe('commandForKey', () => {
	it('maps the book keys', () => {
		expect(key('s')).toBe('file.save');
		expect(key('S', { shiftKey: true })).toBe('file.saveAs');
		expect(key('o')).toBe('file.open');
		expect(key('n')).toBe('file.new');
		expect(key('w')).toBe('file.close');
		expect(key('z')).toBe('edit.undo');
		expect(key('Z', { shiftKey: true })).toBe('edit.redo');
		expect(key('y')).toBe('edit.redo');
	});

	it('takes Command on a Mac for Control', () => {
		expect(key('s', { ctrlKey: false, metaKey: true })).toBe('file.save');
	});

	it('ignores a plain letter, Alt combinations and other keys', () => {
		expect(key('s', { ctrlKey: false })).toBeNull();
		expect(key('s', { altKey: true })).toBeNull();
		expect(key('q')).toBeNull();
	});
});
