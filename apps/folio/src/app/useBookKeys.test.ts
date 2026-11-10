// Tests for the book commands' keys
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { renderHook } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { commandForKey, useBookKeys } from './useBookKeys';

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
		expect(key('e')).toBe('file.export');
		expect(key('z')).toBe('edit.undo');
		expect(key('Z', { shiftKey: true })).toBe('edit.redo');
		expect(key('y')).toBe('edit.redo');
		expect(key('f')).toBe('edit.find');
		expect(key('F')).toBe('edit.find');
		expect(key('h')).toBe('edit.replace');
	});

	it('leaves Ctrl+Shift+F and Ctrl+Shift+H alone', () => {
		expect(key('F', { shiftKey: true })).toBeNull();
		expect(key('H', { shiftKey: true })).toBeNull();
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

describe('useBookKeys and find', () => {
	function press(target: Element, k: string) {
		const event = new KeyboardEvent('keydown', {
			key: k,
			ctrlKey: true,
			bubbles: true,
			cancelable: true,
		});
		target.dispatchEvent(event);
		return event;
	}

	it('runs find from the page, and from the words being edited on it', () => {
		const run = vi.fn(async () => {});
		renderHook(() => useBookKeys(run));
		const page = document.createElement('div');
		const sink = document.createElement('textarea');
		sink.setAttribute('data-text-sink', '');
		document.body.append(page, sink);
		expect(press(page, 'f').defaultPrevented).toBe(true);
		expect(run).toHaveBeenLastCalledWith('edit.find');
		press(sink, 'h');
		expect(run).toHaveBeenLastCalledWith('edit.replace');
		page.remove();
		sink.remove();
	});

	it('leaves Ctrl+F alone in any other field, such as the find bar or a panel', () => {
		const run = vi.fn(async () => {});
		renderHook(() => useBookKeys(run));
		for (const tag of ['input', 'textarea', 'select']) {
			const field = document.createElement(tag);
			document.body.append(field);
			const event = press(field, 'f');
			expect(event.defaultPrevented).toBe(false);
			field.remove();
		}
		expect(run).not.toHaveBeenCalled();
	});
});
