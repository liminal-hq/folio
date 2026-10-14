// Tests for the welcome greeting
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { describe, expect, it } from 'vitest';
import { greetingKey } from './greeting';

const at = (hour: number) => greetingKey(new Date(2026, 9, 13, hour, 30));

describe('greetingKey', () => {
	it('greets by the time of day', () => {
		expect(at(0)).toBe('welcome.greeting.morning');
		expect(at(11)).toBe('welcome.greeting.morning');
		expect(at(12)).toBe('welcome.greeting.afternoon');
		expect(at(17)).toBe('welcome.greeting.afternoon');
		expect(at(18)).toBe('welcome.greeting.evening');
		expect(at(23)).toBe('welcome.greeting.evening');
	});
});
