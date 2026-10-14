// Tests for the page's copy of the open books
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { describe, expect, it } from 'vitest';
import { activeBook, createBooksStore } from './booksStore';
import { createFakeBooksClient } from './fakeBooksClient';

async function started() {
	const fake = createFakeBooksClient();
	const store = createBooksStore(fake.client);
	await store.start();
	return { fake, store };
}

const MOVE = {
	type: 'MoveFrame',
	spread: 'x',
	frame: 'y',
	box: { x: 0, y: 0, w: 1, h: 1 },
} as never;

describe('createBooksStore', () => {
	it('starts with no books', async () => {
		const { store } = await started();
		expect(store.state.getState().summaries).toEqual([]);
		expect(activeBook(store.state.getState())).toBeNull();
	});

	it('shows a new book at once and makes it the one being shown', async () => {
		const { store } = await started();
		const made = await store.newBook({ title: 'Pip' });
		expect(made?.summary.title).toBe('Pip');
		const shown = activeBook(store.state.getState());
		expect(shown?.summary.title).toBe('Pip');
		expect(shown?.summary.dirty).toBe(true);
		expect(shown?.copy.revision).toBe(0);
	});

	it('applies a change to its copy and says whether undo is possible', async () => {
		const { store } = await started();
		await store.newBook({});
		const before = activeBook(store.state.getState())!.copy.book.bleed;
		await store.apply(MOVE);
		const after = activeBook(store.state.getState())!.copy;
		expect(after.revision).toBe(1);
		expect(after.book.bleed).toBe(before + 1);
		expect(after.canUndo).toBe(true);
		expect(after.canRedo).toBe(false);

		await store.undo();
		const undone = activeBook(store.state.getState())!.copy;
		expect(undone.book.bleed).toBe(before);
		expect(undone.canUndo).toBe(false);
		expect(undone.canRedo).toBe(true);

		await store.redo();
		expect(activeBook(store.state.getState())!.copy.book.bleed).toBe(before + 1);
	});

	it('ignores a patch it has already applied, and one older than its copy', async () => {
		const { fake, store } = await started();
		const made = (await store.newBook({}))!;
		await store.apply(MOVE);
		const copy = activeBook(store.state.getState())!.copy;
		fake.emit.patch({
			id: made.summary.id,
			patch: { revision: 1, ops: [] },
			canUndo: false,
			canRedo: false,
		});
		expect(activeBook(store.state.getState())!.copy).toEqual(copy);
	});

	it('reads a book again when it has missed a revision', async () => {
		const { fake, store } = await started();
		const made = (await store.newBook({}))!;
		// Rust is at revision 1; the page hears revision 3 and cannot patch across the gap.
		await fake.client.apply(made.summary.id, MOVE);
		await fake.client.apply(made.summary.id, MOVE);
		await new Promise((resolve) => setTimeout(resolve, 0));
		expect(activeBook(store.state.getState())!.copy.revision).toBeGreaterThanOrEqual(1);
		fake.emit.patch({
			id: made.summary.id,
			patch: { revision: 9, ops: [] },
			canUndo: true,
			canRedo: false,
		});
		await new Promise((resolve) => setTimeout(resolve, 0));
		expect(activeBook(store.state.getState())!.copy.revision).toBe(2);
		expect(fake.calls.filter((c) => c.startsWith('snapshot')).length).toBeGreaterThan(0);
	});

	it('follows a book opened in another window', async () => {
		const { fake, store } = await started();
		await fake.client.openBook('/books/other.folio');
		await new Promise((resolve) => setTimeout(resolve, 0));
		const state = store.state.getState();
		expect(state.summaries).toHaveLength(1);
		expect(activeBook(state)?.summary.title).toBe('other.folio');
	});

	it('moves on when the book being shown is closed', async () => {
		const { store } = await started();
		const first = (await store.newBook({ title: 'One' }))!;
		await store.newBook({ title: 'Two' });
		await store.close(store.state.getState().activeId!);
		expect(activeBook(store.state.getState())?.summary.id).toBe(first.summary.id);
		await store.close(first.summary.id);
		expect(store.state.getState().activeId).toBeNull();
	});

	it('opens the same file once', async () => {
		const { store } = await started();
		await store.openBook('/books/a.folio');
		await store.openBook('/books/a.folio');
		expect(store.state.getState().summaries).toHaveLength(1);
	});

	it('keeps a refusal until the next call succeeds', async () => {
		const { fake, store } = await started();
		await store.newBook({});
		fake.refuseNext('saveBook', 'needsPath', 'needs a place');
		const refused = await store.save(store.state.getState().activeId!);
		expect(refused).toBeNull();
		expect(store.state.getState().error?.kind).toBe('needsPath');
		const saved = await store.save(store.state.getState().activeId!, '/books/a.folio');
		expect(saved?.path).toBe('/books/a.folio');
		expect(store.state.getState().error).toBeNull();
		expect(activeBook(store.state.getState())?.summary.dirty).toBe(false);
	});
});
