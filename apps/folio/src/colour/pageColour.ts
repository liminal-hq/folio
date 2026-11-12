// The colour a spread's page is, for judging how well words or shapes read against it
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { Book } from '@liminal-hq/folio-protocol/generated/Book';
import type { Spread } from '@liminal-hq/folio-protocol/generated/Spread';
import { parseColour } from '@liminal-hq/folio-chrome/ColourPicker/colourMaths';
import { swatchDisplays } from '../books/swatches';
import { DEFAULT_PAGE } from './colourPicker';

/**
 * The solid colour behind the first page of `spread` (its own, the spread's, else the master's),
 * or white when the background is a gradient, a texture or a picture, which have no one colour.
 */
export function pageColourOf(spread: Spread, book: Book): string {
	const bg =
		spread.pages[0]?.bg ?? spread.bgSpan ?? (book.masterPages ?? []).find((m) => m.bg)?.bg ?? null;
	if (bg?.type !== 'colour') return DEFAULT_PAGE;
	const shown = bg.colour.startsWith('swatch:')
		? swatchDisplays(book).get(bg.colour.slice('swatch:'.length))
		: bg.colour;
	return (shown && parseColour(shown)) || DEFAULT_PAGE;
}
