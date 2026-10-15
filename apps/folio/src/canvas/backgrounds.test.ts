// Tests for resolving a spread's backgrounds
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { Master } from '@liminal-hq/folio-protocol/generated/Master';
import type { Spread } from '@liminal-hq/folio-protocol/generated/Spread';
import { describe, expect, it } from 'vitest';
import { resolveBackgrounds } from './backgrounds';
import cases from './fixtures/backgroundCases.json';

describe('resolveBackgrounds', () => {
	// The Rust `resolve` is tested against the same file, so the two cannot drift apart.
	it.each(cases)('$name', ({ masters, spread, expected }) => {
		expect(resolveBackgrounds(masters as unknown as Master[], spread as unknown as Spread)).toEqual(
			expected,
		);
	});
});
