// The spelling checks an editor asks for: Rust checks in the book's language and against its ignore list
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { Intent } from '@liminal-hq/folio-protocol/generated/Intent';
import type { SettingsPatch } from '@liminal-hq/folio-plugin-settings';
import { useMemo } from 'react';
import type { BooksClient } from '../books/booksClient';
import type { Speller } from '../canvas/TextEditor';
import { useSettings, useUpdateSettings } from '../settings/useSettings';

/** The speller for a book, or none when spelling is off. `apply` records "ignore this word" in the book. */
export function useSpeller(
	client: BooksClient,
	id: string,
	apply: (intent: Intent) => unknown,
): Speller | undefined {
	const settings = useSettings();
	const update = useUpdateSettings();
	const spelling = settings.spelling.enabled;
	const personal = settings.spelling.personal;
	return useMemo<Speller | undefined>(
		() =>
			spelling
				? {
						check: (text) =>
							client.spellCheck(id, text, personal).then((found) => found.misspellings),
						suggest: (word) => client.spellSuggest(id, word),
						ignore: (word) => void apply({ type: 'IgnoreWord', word }),
						addToDictionary: (word) =>
							void update({
								spelling: { personal: [...new Set([...personal, word])].sort() },
							} as SettingsPatch),
					}
				: undefined,
		// eslint-disable-next-line react-hooks/exhaustive-deps
		[spelling, client, id, personal],
	);
}
