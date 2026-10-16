// Tests for page labels, against the cases the Rust side also runs
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { Spread } from '@liminal-hq/folio-protocol/generated/Spread';
import { describe, expect, it } from 'vitest';
import cases from './fixtures/labelCases.json';
import { pageLabels, roman } from './pageLabels';

interface Case {
	name: string;
	spreads: { kind: string; pages: number; section?: unknown }[];
	labels: string[][];
	sections: string[][];
}

const spreadsOf = (c: Case) =>
	c.spreads.map((s, i) => ({
		id: `sp_${i}`,
		kind: s.kind,
		pages: Array.from({ length: s.pages }, (_, p) => ({ id: `pg_${i}_${p}`, master: null })),
		frames: [],
		guides: [],
		masterOverrides: [],
		...(s.section ? { section: s.section } : {}),
	})) as unknown as Spread[];

describe('pageLabels', () => {
	for (const c of cases.cases as Case[]) {
		it(c.name, () => {
			const got = pageLabels(spreadsOf(c));
			expect(got.map((row) => row.map((l) => l.label))).toEqual(c.labels);
			expect(got.map((row) => row.map((l) => l.section))).toEqual(c.sections);
		});
	}
});

describe('roman', () => {
	it('writes lower-case numerals up to 3999 and leaves the rest in Arabic', () => {
		expect([1, 4, 9, 14, 1994, 3999].map(roman)).toEqual([
			'i',
			'iv',
			'ix',
			'xiv',
			'mcmxciv',
			'mmmcmxcix',
		]);
		expect([0, 4000].map(roman)).toEqual(['0', '4000']);
	});
});
