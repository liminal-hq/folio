// Tests for the selection the ribbon styles
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { describe, expect, it } from 'vitest';
import { charsBefore, getTextSelection, setTextSelection } from './textSelection';

describe('the selected words', () => {
	it('are only there while a stretch is selected', () => {
		expect(getTextSelection()).toBeNull();
		setTextSelection({ frame: 'f', start: 3, end: 3 });
		expect(getTextSelection()).toBeNull();
		setTextSelection({ frame: 'f', start: 3, end: 6 });
		expect(getTextSelection()).toEqual({ frame: 'f', start: 3, end: 6 });
		setTextSelection(null);
		expect(getTextSelection()).toBeNull();
	});

	it('count characters, not UTF-16 units', () => {
		expect(charsBefore('a😀b', 3)).toBe(2);
		expect(charsBefore('héllo', 2)).toBe(2);
	});
});
