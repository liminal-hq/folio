// Tests for finding the help article that fits a question
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { describe, expect, it } from 'vitest';
import { askHelp, HELP_ARTICLES } from './helpArticles';

describe('askHelp', () => {
	it.each([
		['What is bleed?', 'bleed'],
		['how wide is the spine for my paperback', 'spine'],
		['Will it look right on a Kindle?', 'kindle-kobo'],
		['can i import my word document', 'import'],
		['how do I send this to amazon', 'amazon'],
		['I deleted something, undo please', 'undo'],
	])('answers %j with the %s article', (question, id) => {
		expect(askHelp(question)?.id).toBe(id);
	});

	it('has nothing to say to words it does not know', () => {
		expect(askHelp('zzz qqq')).toBeNull();
		expect(askHelp('')).toBeNull();
	});

	it('keeps article ids distinct', () => {
		expect(new Set(HELP_ARTICLES.map((a) => a.id)).size).toBe(HELP_ARTICLES.length);
	});
});
