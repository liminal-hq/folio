// Tests for the recent colours list
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { describe, expect, it } from 'vitest';
import { pushRecentColour, RECENT_COLOURS_MAX } from './recentColours';

describe('pushRecentColour', () => {
	it('puts the newest first and drops an earlier copy', () => {
		expect(pushRecentColour(['#111111', '#222222'], '#222222')).toEqual(['#222222', '#111111']);
		expect(pushRecentColour([], '#abc')).toEqual(['#aabbcc']);
	});

	it('keeps to eight', () => {
		const eight = Array.from({ length: 8 }, (_, n) => `#00000${n}`);
		const next = pushRecentColour(eight, '#ffffff');
		expect(next).toHaveLength(RECENT_COLOURS_MAX);
		expect(next[0]).toBe('#ffffff');
		expect(next).not.toContain('#000007');
	});

	it('ignores what is not a colour, such as a swatch reference', () => {
		expect(pushRecentColour(['#111111'], 'swatch:ink')).toEqual(['#111111']);
	});
});
