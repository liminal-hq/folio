// Tests for reading a typed colour
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { describe, expect, it } from 'vitest';
import { parseColour } from './colour';

describe('parseColour', () => {
	it('keeps the long form in lower case and expands the short one', () => {
		expect(parseColour('#1F3A5F')).toBe('#1f3a5f');
		expect(parseColour('Fa0')).toBe('#ffaa00');
	});

	it('refuses what is not a colour', () => {
		for (const bad of ['', '#', '#12345', '#1234567', '#gggggg', 'red', '#12 456']) {
			expect(parseColour(bad), bad).toBeNull();
		}
	});
});
