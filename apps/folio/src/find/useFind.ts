// Searching the open book for the find bar: the query, the matches and which one is current
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { FindMatch } from '@liminal-hq/folio-protocol/generated/FindMatch';
import type { FindRequest } from '@liminal-hq/folio-protocol/generated/FindRequest';
import { useCallback, useEffect, useRef, useState } from 'react';
import type { BooksClient } from '../books/booksClient';
import { t } from '../i18n/messages';
import { matchKey, relocate, stepIndex } from './matches';

/** How the words are compared. */
export interface FindOptions {
	matchCase: boolean;
	wholeWord: boolean;
	regex: boolean;
}

export const PLAIN_SEARCH: FindOptions = { matchCase: false, wholeWord: false, regex: false };

/** The match the page is to show and select now; `nonce` changes each time it is asked for. */
export interface FindFocus {
	match: FindMatch;
	nonce: number;
}

export interface FindApi {
	open: boolean;
	/** Counts each time the bar was asked for, so it can take the keyboard again when it is already open. */
	asked: number;
	/** The replace row is showing. */
	replacing: boolean;
	query: string;
	replacement: string;
	options: FindOptions;
	matches: readonly FindMatch[];
	/** There were more matches than are listed. */
	truncated: boolean;
	/** Where the current match is in `matches`. */
	index: number | null;
	/** Why the search could not be made, in words for the person. */
	error: string | null;
	/** What the last replace did. */
	notice: string | null;
	focus: FindFocus | null;
	show(replacing: boolean): void;
	close(): void;
	setQuery(query: string): void;
	setReplacement(replacement: string): void;
	setOptions(options: FindOptions): void;
	step(back: boolean): void;
	replaceOne(): Promise<void>;
	replaceAll(): Promise<void>;
}

/** How long typing pauses before the book is searched again. */
const PAUSE_MS = 120;

/**
 * The state of the find bar. The book is searched in Rust whenever the bar is open and the query,
 * the options or the book change; the page shows the matches and moves to the one asked for.
 * `master` is the master page being edited, whose words may be replaced.
 */
export function useFind(
	client: BooksClient,
	bookId: string,
	revision: number,
	master: string | null,
): FindApi {
	const [open, setOpen] = useState(false);
	const [asked, setAsked] = useState(0);
	const [replacing, setReplacing] = useState(false);
	const [query, setQueryState] = useState('');
	const [replacement, setReplacement] = useState('');
	const [options, setOptionsState] = useState<FindOptions>(PLAIN_SEARCH);
	const [found, setFound] = useState<{
		matches: readonly FindMatch[];
		truncated: boolean;
	}>({ matches: [], truncated: false });
	const [index, setIndex] = useState<number | null>(null);
	const [error, setError] = useState<string | null>(null);
	const [notice, setNotice] = useState<string | null>(null);
	const [focus, setFocus] = useState<FindFocus | null>(null);
	/** Set when the person asked to be taken to a match, so the next answer moves the page. */
	const wantFocus = useRef(false);
	const current = useRef<{ key: string; index: number } | null>(null);
	const nonce = useRef(0);

	const request = useCallback((): FindRequest => ({ query, ...options }), [query, options]);

	const land = useCallback((matches: readonly FindMatch[], at: number | null) => {
		setIndex(at);
		const match = at === null ? undefined : matches[at];
		current.current = match && at !== null ? { key: matchKey(match), index: at } : null;
		if (match && wantFocus.current) {
			wantFocus.current = false;
			nonce.current += 1;
			setFocus({ match, nonce: nonce.current });
		}
	}, []);

	useEffect(() => {
		if (!open || query === '') {
			setFound({ matches: [], truncated: false });
			setError(null);
			land([], null);
			return;
		}
		let latest = true;
		const timer = setTimeout(() => {
			client
				.find(bookId, request())
				.then((report) => {
					if (!latest) return;
					setError(null);
					setFound({ matches: report.matches, truncated: report.truncated });
					land(report.matches, relocate(report.matches, current.current));
				})
				.catch((reason: unknown) => {
					if (!latest) return;
					const kind = (reason as { kind?: string } | null)?.kind;
					setError(kind === 'badSearch' ? t('find.badPattern') : String(reason));
					setFound({ matches: [], truncated: false });
					land([], null);
				});
		}, PAUSE_MS);
		return () => {
			latest = false;
			clearTimeout(timer);
		};
		// The book's revision is a reason to search again: the words moved.
	}, [open, query, options, revision, bookId, client, request, land]);

	const show = useCallback((replace: boolean) => {
		setOpen(true);
		setAsked((n) => n + 1);
		setReplacing((was) => replace || was);
		setNotice(null);
		wantFocus.current = true;
	}, []);

	const close = useCallback(() => {
		setOpen(false);
		setFocus(null);
		setNotice(null);
		wantFocus.current = false;
	}, []);

	const setQuery = useCallback((next: string) => {
		wantFocus.current = true;
		current.current = null;
		setNotice(null);
		setQueryState(next);
	}, []);

	const setOptions = useCallback((next: FindOptions) => {
		wantFocus.current = true;
		current.current = null;
		setOptionsState(next);
	}, []);

	const step = useCallback(
		(back: boolean) => {
			const at = stepIndex(index, found.matches.length, back);
			if (at === null) return;
			wantFocus.current = true;
			land(found.matches, at);
		},
		[index, found.matches, land],
	);

	const replaceWith = useCallback(
		async (only: FindMatch | null) => {
			if (query === '') return;
			try {
				const report = await client.replace(bookId, {
					find: request(),
					replacement,
					...(master ? { master } : {}),
					...(only
						? { only: { target: only.target, para: only.para, from: only.from, to: only.to } }
						: {}),
				});
				const skipped = report.skippedLocked + report.skippedMasters;
				setNotice(
					report.replaced === 0 && skipped === 0
						? t('find.replaced.none')
						: skipped > 0
							? t('find.replaced.skipped', { n: report.replaced, skipped })
							: t('find.replaced', { n: report.replaced }),
				);
				// The next match takes the place of the one replaced, and the page goes to it.
				wantFocus.current = true;
			} catch (reason) {
				setNotice(String((reason as { message?: string } | null)?.message ?? reason));
			}
		},
		[client, bookId, request, replacement, master, query],
	);

	const replaceOne = useCallback(
		() => replaceWith(index === null ? null : (found.matches[index] ?? null)),
		[replaceWith, index, found.matches],
	);

	return {
		open,
		asked,
		replacing,
		query,
		replacement,
		options,
		matches: found.matches,
		truncated: found.truncated,
		index,
		error,
		notice,
		focus,
		show,
		close,
		setQuery,
		setReplacement,
		setOptions,
		step,
		replaceOne: async () => {
			if (index === null) return;
			await replaceOne();
		},
		replaceAll: () => replaceWith(null),
	};
}
