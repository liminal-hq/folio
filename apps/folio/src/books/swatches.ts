// Swatch references turned into the colours they show as, for drawing a book on screen
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { Book } from '@liminal-hq/folio-protocol/generated/Book';

/** What a reference to a swatch the book does not have is drawn as. */
const MISSING = '#808080';

/** The colour each swatch of `book` shows as, by id. */
export function swatchDisplays(book: Book): Map<string, string> {
	return new Map((book.styles?.swatches ?? []).map((s) => [s.id, s.display]));
}

/**
 * `value` with every `swatch:<id>` string in it replaced by the colour that swatch shows as. A
 * value with no reference in it is returned as it is, so what has not changed stays the same
 * object and drawing it again is cheap.
 */
export function resolveSwatches<T>(value: T, displays: Map<string, string>): T {
	if (typeof value === 'string') {
		if (!value.startsWith('swatch:')) return value;
		return (displays.get(value.slice('swatch:'.length)) ?? MISSING) as T;
	}
	if (Array.isArray(value)) {
		let changed = false;
		const mapped = value.map((item) => {
			const next = resolveSwatches(item, displays);
			if (next !== item) changed = true;
			return next;
		});
		return (changed ? mapped : value) as T;
	}
	if (value !== null && typeof value === 'object') {
		let changed = false;
		const entries = Object.entries(value as Record<string, unknown>).map(([key, item]) => {
			const next = resolveSwatches(item, displays);
			if (next !== item) changed = true;
			return [key, next] as const;
		});
		return (changed ? Object.fromEntries(entries) : value) as T;
	}
	return value;
}
