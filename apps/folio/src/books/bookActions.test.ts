// Tests for the save, open and close actions
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { describe, expect, it, vi } from 'vitest';
import { createBookActions, needsAsking, samePath, withExtension } from './bookActions';
import { activeBook, createBooksStore } from './booksStore';
import { createFakeBooksClient } from './fakeBooksClient';
import type { FileDialogs } from './dialogs';

const MOVE = { type: 'MoveFrame' } as never;

async function setup(picks: { open?: string | null; save?: string | null } = {}) {
	const fake = createFakeBooksClient();
	const store = createBooksStore(fake.client);
	await store.start();
	const saveStarts: string[] = [];
	const dialogs: FileDialogs = {
		pickBook: async () => picks.open ?? null,
		pickSavePath: async (start) => {
			saveStarts.push(start);
			return picks.save ?? null;
		},
		pickImage: async () => null,
		pickText: async () => null,
	};
	return { fake, store, saveStarts, actions: createBookActions(store, fake.client, dialogs) };
}

describe('withExtension', () => {
	it('adds .folio only when it is missing', () => {
		expect(withExtension('/b/otter')).toBe('/b/otter.folio');
		expect(withExtension('/b/otter.folio')).toBe('/b/otter.folio');
		expect(withExtension('/b/otter.FOLIO')).toBe('/b/otter.FOLIO');
	});
});

describe('save', () => {
	it('asks where to go the first time, starting at the default place, then remembers it', async () => {
		const { store, actions, saveStarts } = await setup({ save: '/books/Pip/Pip' });
		await store.newBook({ title: 'Pip' });
		expect(await actions.save()).toBe('done');
		expect(saveStarts).toEqual(['/home/mom/Books/Pip/Pip.folio']);
		const shown = activeBook(store.state.getState())!;
		expect(shown.summary.path).toBe('/books/Pip/Pip.folio');
		expect(shown.summary.dirty).toBe(false);

		await store.apply(MOVE);
		expect(await actions.save()).toBe('done');
		expect(saveStarts).toHaveLength(1);
	});

	it('does nothing when the person cancels the picker', async () => {
		const { store, actions } = await setup({ save: null });
		await store.newBook({});
		expect(await actions.save()).toBe('cancelled');
		expect(activeBook(store.state.getState())!.summary.path).toBeNull();
	});

	it('save as starts where the book already is', async () => {
		const { store, actions, saveStarts } = await setup({ save: '/books/Copy' });
		await store.openBook('/books/a.folio');
		await actions.saveAs();
		expect(saveStarts).toEqual(['/books/a.folio']);
		expect(activeBook(store.state.getState())!.summary.path).toBe('/books/Copy.folio');
	});

	it('reports a refusal', async () => {
		const { fake, store, actions } = await setup({ save: '/b/x' });
		await store.newBook({});
		fake.refuseNext('saveBook', 'package', 'disk full');
		expect(await actions.save()).toBe('failed');
		expect(store.state.getState().error?.message).toBe('disk full');
	});
});

describe('open', () => {
	it('opens what the picker gives, or a path from the recents', async () => {
		const { store, actions } = await setup({ open: '/books/a.folio' });
		expect(await actions.open()).toBe('done');
		expect(await actions.open('/books/b.folio')).toBe('done');
		expect(store.state.getState().summaries.map((b) => b.path)).toEqual([
			'/books/a.folio',
			'/books/b.folio',
		]);
	});

	it('does nothing when the picker is cancelled', async () => {
		const { store, actions } = await setup({ open: null });
		expect(await actions.open()).toBe('cancelled');
		expect(store.state.getState().summaries).toHaveLength(0);
	});
});

describe('close', () => {
	it('closes an untouched new book without asking', async () => {
		const { store, actions } = await setup();
		await store.newBook({});
		const decide = vi.fn();
		expect(await actions.close(decide)).toBe('done');
		expect(decide).not.toHaveBeenCalled();
		expect(store.state.getState().summaries).toHaveLength(0);
	});

	it('asks once a new book has been changed, and keeps it on cancel', async () => {
		const { store, actions } = await setup();
		await store.newBook({});
		await store.apply(MOVE);
		expect(await actions.close(async () => 'cancel')).toBe('cancelled');
		expect(store.state.getState().summaries).toHaveLength(1);
		expect(await actions.close(async () => 'discard')).toBe('done');
		expect(store.state.getState().summaries).toHaveLength(0);
	});

	it('saves first when asked, and stays open if the save is cancelled', async () => {
		const cancelled = await setup({ save: null });
		await cancelled.store.newBook({});
		await cancelled.store.apply(MOVE);
		expect(await cancelled.actions.close(async () => 'save')).toBe('cancelled');
		expect(cancelled.store.state.getState().summaries).toHaveLength(1);

		const saved = await setup({ save: '/b/kept' });
		await saved.store.newBook({});
		await saved.store.apply(MOVE);
		expect(await saved.actions.close(async () => 'save')).toBe('done');
		expect(saved.store.state.getState().summaries).toHaveLength(0);
		expect(saved.fake.calls.some((c) => c.startsWith('saveBook'))).toBe(true);
	});

	it('asks for a saved book only when it has changes', async () => {
		const { store, actions } = await setup();
		await store.openBook('/books/a.folio');
		const decide = vi.fn(async () => 'discard' as const);
		expect(await actions.close(decide)).toBe('done');
		expect(decide).not.toHaveBeenCalled();

		await store.openBook('/books/b.folio');
		await store.apply(MOVE);
		expect(await actions.close(decide)).toBe('done');
		expect(decide).toHaveBeenCalledOnce();
	});
});

describe('samePath', () => {
	it('treats / and \\ alike, and ignores case only on a Windows drive', () => {
		expect(samePath('/books/Pip.folio', '/books/Pip.folio')).toBe(true);
		expect(samePath('/books/Pip.folio', '/books/pip.folio')).toBe(false);
		expect(samePath('C:\\Books\\Pip.folio', 'c:/books/pip.folio')).toBe(true);
		expect(samePath('\\\\?\\C:\\Books\\Pip.folio', 'C:/Books/Pip.folio')).toBe(true);
	});
});

describe('opening a book a crash left work on', () => {
	const left = { id: 'r1', title: 'Pip', path: '/books/Pip.folio', savedAt: 1_700_000_000 };

	it('asks which to open, and restores the work when asked to', async () => {
		const { fake, store, actions } = await setup();
		fake.recoveries.push(left);
		const decide = vi.fn(async () => 'restore' as const);
		expect(await actions.open('/books/Pip.folio', decide)).toBe('done');
		expect(decide).toHaveBeenCalledWith(left);
		const shown = activeBook(store.state.getState())!;
		expect(shown.summary.path).toBe('/books/Pip.folio');
		expect(shown.summary.dirty).toBe(true);
		expect(fake.recoveries).toEqual([]);
	});

	it('opens the file as saved, and gives the work up, when asked to', async () => {
		const { fake, store, actions } = await setup();
		fake.recoveries.push(left);
		expect(await actions.open('/books/Pip.folio', async () => 'file')).toBe('done');
		expect(activeBook(store.state.getState())!.summary.dirty).toBe(false);
		expect(fake.recoveries).toEqual([]);
	});

	it('opens nothing and keeps the work when the person cancels', async () => {
		const { fake, store, actions } = await setup();
		fake.recoveries.push(left);
		expect(await actions.open('/books/Pip.folio', async () => 'cancel')).toBe('cancelled');
		expect(store.state.getState().summaries).toHaveLength(0);
		expect(fake.recoveries).toEqual([left]);
	});

	it('does not ask about work on another book, or when no one is there to ask', async () => {
		const { fake, actions } = await setup();
		fake.recoveries.push({ ...left, path: '/books/Other.folio' });
		const decide = vi.fn(async () => 'cancel' as const);
		expect(await actions.open('/books/Pip.folio', decide)).toBe('done');
		expect(decide).not.toHaveBeenCalled();
		fake.recoveries.push(left);
		expect(await actions.open('/books/Pip.folio')).toBe('done');
		expect(fake.recoveries).toHaveLength(2);
	});
});

describe('closeAll', () => {
	it('closes books nobody has touched without asking, and says the app may go', async () => {
		const { store, actions } = await setup();
		await store.newBook({});
		const decide = vi.fn(async () => 'cancel' as const);
		expect(await actions.closeAll(decide)).toBe('done');
		expect(decide).not.toHaveBeenCalled();
		expect(store.state.getState().summaries).toHaveLength(0);
	});

	it('asks about each book with changes, in turn, and saves the ones asked to be', async () => {
		const { store, actions } = await setup({ save: '/books/first' });
		await store.newBook({ title: 'First' });
		await store.apply(MOVE);
		await store.newBook({ title: 'Second' });
		await store.apply(MOVE);
		const asked: string[] = [];
		const decide = async (summary: { title: string }) => {
			asked.push(summary.title);
			return summary.title === 'First' ? ('save' as const) : ('discard' as const);
		};
		expect(await actions.closeAll(decide)).toBe('done');
		expect(asked).toEqual(['First', 'Second']);
		expect(store.state.getState().summaries).toHaveLength(0);
	});

	it('closes nothing when the person cancels, even the books already answered', async () => {
		const { store, actions } = await setup();
		await store.newBook({ title: 'First' });
		await store.apply(MOVE);
		await store.newBook({ title: 'Second' });
		await store.apply(MOVE);
		const decide = async (summary: { title: string }) =>
			summary.title === 'First' ? ('discard' as const) : ('cancel' as const);
		expect(await actions.closeAll(decide)).toBe('cancelled');
		expect(store.state.getState().summaries).toHaveLength(2);
	});

	it('stays open when a save the person asked for is cancelled', async () => {
		const { store, actions } = await setup({ save: null });
		await store.newBook({ title: 'First' });
		await store.apply(MOVE);
		expect(await actions.closeAll(async () => 'save')).toBe('cancelled');
		expect(store.state.getState().summaries).toHaveLength(1);
	});
});

describe('needsAsking', () => {
	it('is about changes, not about never having been saved', () => {
		const summary = { id: 'x', title: 't', kind: 'picture', path: null, dirty: true } as const;
		const copy = { revision: 0 } as never;
		expect(needsAsking(summary, copy)).toBe(false);
		expect(needsAsking(summary, { revision: 3 } as never)).toBe(true);
		expect(
			needsAsking({ ...summary, path: '/a.folio', dirty: false }, { revision: 3 } as never),
		).toBe(false);
	});
});
