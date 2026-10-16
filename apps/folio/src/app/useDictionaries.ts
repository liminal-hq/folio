// The dictionaries Preferences lists, and downloading and removing them
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { DictionaryStatus } from '@liminal-hq/folio-protocol/generated/DictionaryStatus';
import { createContext, useCallback, useContext, useEffect, useState } from 'react';
import { tauriBooksClient, type BooksClient } from '../books/booksClient';

/** What Preferences needs of the books plugin. */
export type DictionaryApi = Pick<
	BooksClient,
	'dictionaries' | 'downloadDictionary' | 'removeDictionary'
>;

export const DictionaryApiContext = createContext<DictionaryApi>(tauriBooksClient);

const messageOf = (error: unknown) =>
	typeof error === 'object' && error !== null && 'message' in error
		? String((error as { message: unknown }).message)
		: String(error);

export interface Dictionaries {
	list: DictionaryStatus[];
	/** The id being downloaded or removed, if one is. */
	busy: string | null;
	/** Why the last download or removal failed, if it did. */
	error: string | null;
	download(id: string): void;
	remove(id: string): void;
}

export function useDictionaries(): Dictionaries {
	const api = useContext(DictionaryApiContext);
	const [list, setList] = useState<DictionaryStatus[]>([]);
	const [busy, setBusy] = useState<string | null>(null);
	const [error, setError] = useState<string | null>(null);

	useEffect(() => {
		let live = true;
		api
			.dictionaries()
			.then((found) => live && setList(found))
			.catch((e: unknown) => live && setError(messageOf(e)));
		return () => {
			live = false;
		};
	}, [api]);

	const run = useCallback((id: string, step: (id: string) => Promise<DictionaryStatus[]>) => {
		setBusy(id);
		setError(null);
		step(id)
			.then(setList)
			.catch((e: unknown) => setError(messageOf(e)))
			.finally(() => setBusy(null));
	}, []);

	return {
		list,
		busy,
		error,
		download: (id) => run(id, api.downloadDictionary),
		remove: (id) => run(id, api.removeDictionary),
	};
}
