// The typefaces a book can draw in, read from the plugin and read again when the book's fonts change
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { FontChoice } from '@liminal-hq/folio-protocol/generated/FontChoice';
import { useEffect, useState } from 'react';
import type { BooksClient } from '../books/booksClient';

/**
 * The choices for the book `id`, or `undefined` until they have been read. `kept` is the book's
 * list of fonts it keeps (or just its length): keeping a font moves it from the computer's list to
 * the book's, so a change in it asks again.
 */
export function useFontChoices(
	client: BooksClient,
	id: string,
	kept: string,
): FontChoice[] | undefined {
	const [choices, setChoices] = useState<FontChoice[] | undefined>(undefined);
	useEffect(() => {
		let live = true;
		client
			.listFonts(id)
			.then((found) => live && setChoices(found))
			.catch(() => undefined);
		return () => {
			live = false;
		};
	}, [client, id, kept]);
	return choices;
}
