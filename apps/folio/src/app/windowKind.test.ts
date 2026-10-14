// Tests for windowKindFromLabel
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { describe, expect, it } from 'vitest';
import { windowKindFromLabel } from './windowKind';

describe('windowKindFromLabel', () => {
	it('classifies known labels', () => {
		expect(windowKindFromLabel('main')).toBe('Main');
		expect(windowKindFromLabel('about')).toBe('About');
		expect(windowKindFromLabel('preferences')).toBe('Preferences');
		expect(windowKindFromLabel('book-tools')).toBe('BookTools');
	});

	it('rejects unknown labels', () => {
		expect(windowKindFromLabel('mystery')).toBeNull();
		expect(windowKindFromLabel('main-1')).toBeNull();
		expect(windowKindFromLabel('')).toBeNull();
	});
});
