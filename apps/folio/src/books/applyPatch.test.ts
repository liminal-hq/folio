// Checks the UI's patch code against patches a real Rust session produced
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { Book } from '@liminal-hq/folio-protocol/generated/Book';
import type { Patch } from '@liminal-hq/folio-protocol/generated/Patch';
import { describe, expect, it } from 'vitest';
import { applyPatch } from './applyPatch';
import cases from './fixtures/patchCases.json';

interface PatchCase {
	name: string;
	before: Book;
	patch: Patch;
	after: Book;
}

describe('applyPatch', () => {
	it('has cases to check against', () => {
		expect((cases as unknown as PatchCase[]).length).toBeGreaterThanOrEqual(7);
	});

	for (const c of cases as unknown as PatchCase[]) {
		it(`turns the book before into the book after: ${c.name}`, () => {
			expect(applyPatch(c.before, c.patch)).toEqual(c.after);
		});
	}

	it('does not change the book it was given', () => {
		const first = (cases as unknown as PatchCase[])[0]!;
		const before = structuredClone(first.before);
		applyPatch(first.before, first.patch);
		expect(first.before).toEqual(before);
	});

	it('leaves a book alone for a patch with no ops', () => {
		const first = (cases as unknown as PatchCase[])[0]!;
		const same = applyPatch(first.before, { revision: 9, ops: [] });
		expect(same).toEqual(first.before);
	});
});
