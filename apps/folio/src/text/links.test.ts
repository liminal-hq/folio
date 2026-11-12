// Tests for reading what was typed as the address of a link
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { describe, expect, it } from 'vitest';
import { checkHref, isExternalHref, isInternalHref, spreadOfHref } from './links';

const ok = (typed: string, spreads = 5) => {
	const found = checkHref(typed, spreads);
	return found.ok ? found.href : found.reason;
};

describe('checkHref', () => {
	it('keeps web, email and phone addresses', () => {
		expect(ok('https://example.org/a?b=c#d')).toBe('https://example.org/a?b=c#d');
		expect(ok('HTTP://Example.org')).toBe('http://Example.org');
		expect(ok('mailto:me@example.org')).toBe('mailto:me@example.org');
		expect(ok('tel:+15555550100')).toBe('tel:+15555550100');
	});

	it('makes an address from what a person would type', () => {
		expect(ok('example.org')).toBe('https://example.org');
		expect(ok('  www.example.org/path  ')).toBe('https://www.example.org/path');
		expect(ok('example.org:8080/a')).toBe('https://example.org:8080/a');
		expect(ok('localhost:3000')).toBe('https://localhost:3000');
		expect(ok('me@example.org')).toBe('mailto:me@example.org');
	});

	it('links to a spread of the book, within the book', () => {
		expect(ok('#spread-3')).toBe('#spread-3');
		expect(ok('#SPREAD-03')).toBe('#spread-3');
		expect(ok('#spread-6')).toBe('spread');
		expect(ok('#spread-0')).toBe('spread');
		expect(ok('#chapter-1')).toBe('invalid');
	});

	it('refuses addresses that run code or open files, and what is not an address', () => {
		expect(ok('javascript:alert(1)')).toBe('unsafe');
		expect(ok('JaVaScRiPt:alert(1)')).toBe('unsafe');
		expect(ok('data:text/html,hi')).toBe('unsafe');
		expect(ok('file:///etc/passwd')).toBe('unsafe');
		expect(ok('ftp://example.org')).toBe('unsafe');
		expect(ok('')).toBe('empty');
		expect(ok('   ')).toBe('empty');
		expect(ok('https://')).toBe('invalid');
		expect(ok('mailto:')).toBe('invalid');
		expect(ok('hello there')).toBe('invalid');
		expect(ok('hello')).toBe('invalid');
		expect(ok('https://exa mple.org')).toBe('invalid');
	});
});

describe('links to the book and out of it', () => {
	it('tells a spread from an address', () => {
		expect(spreadOfHref('#spread-12')).toBe(12);
		expect(spreadOfHref('#spread-0')).toBeNull();
		expect(spreadOfHref('https://example.org')).toBeNull();
		expect(isInternalHref('#spread-2')).toBe(true);
		expect(isExternalHref('#spread-2')).toBe(false);
		expect(isExternalHref('mailto:a@b.org')).toBe(true);
		expect(isExternalHref('javascript:alert(1)')).toBe(false);
	});
});
