// Tests for the languages a book can be written in
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { DictionaryStatus } from '@liminal-hq/folio-protocol/generated/DictionaryStatus';
import { describe, expect, it } from 'vitest';
import { languageChoices } from './languageChoices';

const dictionary = (
	id: string,
	name: string,
	languages: string[],
	installed: boolean,
): DictionaryStatus => ({
	id,
	name,
	languages,
	licence: 'MIT',
	bytes: 1,
	installed,
	bundled: id === 'en-CA',
});

const list = [
	dictionary('en-CA', 'English (Canada)', ['en-CA'], true),
	dictionary('en-US', 'English (United States)', ['en-US', 'en'], false),
	dictionary('fr', 'French', ['fr', 'fr-CA'], true),
];

describe('languageChoices', () => {
	it('offers each dictionary under its own name, saying which are not downloaded', () => {
		const { choices } = languageChoices(list, 'en-CA');
		expect(choices).toEqual([
			{ value: 'en-CA', label: 'English (Canada)' },
			{ value: 'en-US', label: 'English (United States) — not downloaded' },
			{ value: 'fr', label: 'French' },
		]);
	});

	it('selects the dictionary that claims the book language exactly', () => {
		expect(languageChoices(list, 'en-CA').selected).toBe('en-CA');
		expect(languageChoices(list, 'fr-CA').selected).toBe('fr');
	});

	it('falls back to a dictionary for the plain language', () => {
		expect(languageChoices(list, 'en').selected).toBe('en-US');
		expect(languageChoices(list, 'fr-BE').selected).toBe('fr');
	});

	it('keeps a language no dictionary claims, so the book is never shown in another one', () => {
		const { choices, selected } = languageChoices(list, 'de');
		expect(selected).toBe('de');
		expect(choices.at(-1)).toEqual({ value: 'de', label: 'de' });
	});
});
