// The chapters of a book's stories, read from the plugin and read again when the book changes
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { StoryChapter } from '@liminal-hq/folio-protocol/generated/StoryChapter';
import { useEffect, useState } from 'react';
import type { BooksClient } from '../books/booksClient';

/** How long after the book last changed the chapters are read again, in milliseconds. */
const WAIT_MS = 500;

/** The chapters for the book `id`; `revision` is when to ask again, and `wanted` whether to at all. */
export function useStoryChapters(
	client: BooksClient,
	id: string,
	revision: number,
	wanted: boolean,
): StoryChapter[] {
	const [chapters, setChapters] = useState<StoryChapter[]>([]);
	useEffect(() => {
		if (!wanted) return;
		let live = true;
		// Every spread is laid out to find them, so a run of edits is read once.
		const wait = setTimeout(() => {
			client
				.storyChapters(id)
				.then((found) => live && setChapters(found))
				.catch(() => undefined);
		}, WAIT_MS);
		return () => {
			live = false;
			clearTimeout(wait);
		};
	}, [client, id, revision, wanted]);
	return chapters;
}
