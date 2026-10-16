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
	'dictionaries' | 'downloadDictionary' | 'removeDictionary' | 'onDictionaryProgress'
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
	/** How far the download under way has got, from 0 to 1, with the id of what is being fetched. */
	progress: { id: string; fraction: number } | null;
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
	const [progress, setProgress] = useState<{ id: string; fraction: number } | null>(null);

	// Any window downloading is heard, so the bar shows wherever the list is.
	useEffect(() => {
		let stop: (() => void) | undefined;
		let live = true;
		void api
			.onDictionaryProgress((event) =>
				setProgress({
					id: event.id,
					fraction: event.total > 0 ? Math.min(1, event.received / event.total) : 0,
				}),
			)
			.then((off) => (live ? (stop = off) : off()))
			.catch(() => undefined);
		return () => {
			live = false;
			stop?.();
		};
	}, [api]);

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
			.finally(() => {
				setBusy(null);
				setProgress(null);
			});
	}, []);

	return {
		list,
		busy,
		progress,
		error,
		download: (id) => run(id, api.downloadDictionary),
		remove: (id) => run(id, api.removeDictionary),
	};
}
