// Tests that the bubble outlines match the Rust twin, through one fixture
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { describe, expect, it } from 'vitest';
import cases from './fixtures/shapePaths.json';
import { SHAPE_PATHS } from './shapePaths';

describe('SHAPE_PATHS', () => {
	it('is what the Rust scene draws the bubbles with', () => {
		expect(SHAPE_PATHS).toEqual(cases);
	});
});
