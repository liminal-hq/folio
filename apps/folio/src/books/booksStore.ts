// The page's copy of the open books: read once, then kept current by Rust's events
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type {
	BookSnapshot,
	BookSummary,
	BooksChanged,
	BooksCommandError,
	Intent,
	KindInfo,
	NewBookSpec,
	PatchEvent,
} from '@liminal-hq/folio-plugin-books';
import type { Book } from '@liminal-hq/folio-protocol/generated/Book';
import { createStore, type StoreApi } from 'zustand/vanilla';
import { applyPatch } from './applyPatch';
import type { BooksClient } from './booksClient';

/** A book this window holds in full. */
export interface OpenBookCopy {
	revision: number;
	book: Book;
	styles: unknown;
	settings: unknown;
	canUndo: boolean;
	canRedo: boolean;
}

export interface BooksState {
	/** Every open book, oldest first, from every window. */
	summaries: BookSummary[];
	/** The books this window has the contents of, by id. */
	copies: Record<string, OpenBookCopy>;
	/** The book this window is showing. */
	activeId: string | null;
	/** The kinds of book and their sizes, from Rust; empty until they have been read. */
	kinds: KindInfo[];
	/** The last refusal, until the next call succeeds. */
	error: BooksCommandError | null;
}

export interface BooksStore {
	state: StoreApi<BooksState>;
	/** Reads the open books and follows changes. Returns a stop function. */
	start(): Promise<() => void>;
	setActive(id: string | null): Promise<void>;
	newBook(spec: Partial<NewBookSpec>): Promise<BookSnapshot | null>;
	openBook(path: string): Promise<BookSnapshot | null>;
	save(id: string, path?: string): Promise<BookSummary | null>;
	close(id: string): Promise<void>;
	apply(intent: Intent): Promise<void>;
	undo(): Promise<void>;
	redo(): Promise<void>;
}

function asCommandError(reason: unknown): BooksCommandError {
	if (typeof reason === 'object' && reason !== null && 'kind' in reason && 'message' in reason) {
		return reason as BooksCommandError;
	}
	return { kind: 'package', message: String(reason) };
}

function copyOf(snapshot: BookSnapshot): OpenBookCopy {
	return {
		revision: snapshot.revision,
		book: snapshot.book,
		styles: snapshot.styles,
		settings: snapshot.settings,
		canUndo: snapshot.canUndo,
		canRedo: snapshot.canRedo,
	};
}

/**
 * The open books for one window. A change event and a read can arrive in either order, so a patch
 * is applied only when its revision is the next one the copy expects or later, and a copy that
 * would have to skip a revision is read again whole. The store subscribes before it reads, so a
 * change made in between reaches it as an event instead of falling through the gap.
 */
export function createBooksStore(client: BooksClient): BooksStore {
	const state = createStore<BooksState>(() => ({
		summaries: [],
		copies: {},
		activeId: null,
		kinds: [],
		error: null,
	}));

	const fail = (reason: unknown) => {
		state.setState({ error: asCommandError(reason) });
		return null;
	};

	/** Adopts a book read in full and makes it the one being shown. */
	const adopt = (snapshot: BookSnapshot) => {
		const { summary } = snapshot;
		state.setState((s) => ({
			summaries: s.summaries.some((b) => b.id === summary.id)
				? s.summaries.map((b) => (b.id === summary.id ? summary : b))
				: [...s.summaries, summary],
			copies: { ...s.copies, [summary.id]: copyOf(snapshot) },
			activeId: summary.id,
			error: null,
		}));
	};

	const load = async (id: string) => {
		try {
			const snapshot = await client.snapshot(id);
			state.setState((s) => ({
				copies: { ...s.copies, [id]: copyOf(snapshot) },
			}));
		} catch {
			// The book was closed from another window between the event and the read.
		}
	};

	const onPatch = (event: PatchEvent) => {
		const held = state.getState().copies[event.id];
		if (!held) return;
		const revision = event.patch.revision;
		if (revision <= held.revision) return;
		if (revision > held.revision + 1) {
			// A revision was missed: the copy cannot be patched, so read the book again.
			void load(event.id);
			return;
		}
		state.setState((s) => ({
			copies: {
				...s.copies,
				[event.id]: {
					...held,
					revision,
					book: applyPatch(held.book, event.patch),
					canUndo: event.canUndo,
					canRedo: event.canRedo,
				},
			},
		}));
	};

	const onChanged = (event: BooksChanged) => {
		const listed = new Set(event.books.map((b) => b.id));
		state.setState((s) => {
			const copies = Object.fromEntries(Object.entries(s.copies).filter(([id]) => listed.has(id)));
			const stillThere = s.activeId !== null && listed.has(s.activeId);
			const activeId = stillThere ? s.activeId : (event.books[event.books.length - 1]?.id ?? null);
			return { summaries: event.books, copies, activeId };
		});
		const { activeId, copies } = state.getState();
		if (activeId !== null && !copies[activeId]) void load(activeId);
	};

	const active = () => {
		const { activeId } = state.getState();
		return activeId;
	};

	return {
		state,
		async start() {
			const stop = await client.subscribe({ onPatch, onChanged });
			try {
				state.setState({ kinds: await client.presets() });
			} catch {
				// Not in Tauri: there is nothing to start a book from.
			}
			try {
				const books = await client.list();
				state.setState((s) => ({
					summaries: books,
					activeId: s.activeId ?? books[0]?.id ?? null,
				}));
				const { activeId } = state.getState();
				if (activeId !== null && !state.getState().copies[activeId]) await load(activeId);
			} catch {
				// Not in Tauri: there are no books to show.
			}
			return stop;
		},
		async setActive(id) {
			state.setState({ activeId: id });
			if (id !== null && !state.getState().copies[id]) await load(id);
		},
		async newBook(spec) {
			try {
				const snapshot = await client.newBook(spec);
				adopt(snapshot);
				return snapshot;
			} catch (reason) {
				return fail(reason);
			}
		},
		async openBook(path) {
			try {
				const snapshot = await client.openBook(path);
				adopt(snapshot);
				return snapshot;
			} catch (reason) {
				return fail(reason);
			}
		},
		async save(id, path) {
			try {
				const summary = await client.saveBook(id, path);
				state.setState((s) => ({
					summaries: s.summaries.map((b) => (b.id === id ? summary : b)),
					error: null,
				}));
				return summary;
			} catch (reason) {
				return fail(reason);
			}
		},
		async close(id) {
			try {
				await client.closeBook(id);
				state.setState({ error: null });
			} catch (reason) {
				fail(reason);
			}
		},
		async apply(intent) {
			const id = active();
			if (id === null) return;
			try {
				const patch = await client.apply(id, intent);
				onPatch({
					id,
					patch,
					canUndo: true,
					canRedo: false,
				});
				state.setState({ error: null });
			} catch (reason) {
				fail(reason);
			}
		},
		async undo() {
			const id = active();
			if (id === null) return;
			try {
				await client.undo(id);
			} catch (reason) {
				fail(reason);
			}
		},
		async redo() {
			const id = active();
			if (id === null) return;
			try {
				await client.redo(id);
			} catch (reason) {
				fail(reason);
			}
		},
	};
}

/** The book being shown with its summary, or `null` when there is none (or it is still loading). */
export function activeBook(state: BooksState): { summary: BookSummary; copy: OpenBookCopy } | null {
	if (state.activeId === null) return null;
	const summary = state.summaries.find((b) => b.id === state.activeId);
	const copy = state.copies[state.activeId];
	return summary && copy ? { summary, copy } : null;
}
