// Tests for the state of the find bar: searching the book, the current match and replacing
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { FindMatch } from '@liminal-hq/folio-protocol/generated/FindMatch';
import { act, renderHook, waitFor } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { createFakeBooksClient } from '../books/fakeBooksClient';
import { useFind } from './useFind';

const match = (from: number, spread = 's'): FindMatch => ({
	target: { type: 'frame', frame: 'a' },
	para: 0,
	from,
	to: from + 3,
	spread,
	context: 'cat',
	locked: false,
});

async function setup(matches: FindMatch[] = [match(0), match(5), match(9)]) {
	const fake = createFakeBooksClient();
	const made = await fake.client.newBook({});
	const id = made.summary.id;
	fake.found.push(...matches);
	const hook = renderHook(({ revision, master }) => useFind(fake.client, id, revision, master), {
		initialProps: { revision: made.revision, master: null as string | null },
	});
	return { fake, hook, id, revision: made.revision };
}

describe('useFind', () => {
	it('searches nothing while the bar is shut or the query is empty', async () => {
		const { fake, hook } = await setup();
		act(() => hook.result.current.setQuery('cat'));
		await new Promise((r) => setTimeout(r, 200));
		expect(fake.calls.some((c) => c.startsWith('find('))).toBe(false);
		act(() => hook.result.current.show(false));
		act(() => hook.result.current.setQuery(''));
		await new Promise((r) => setTimeout(r, 200));
		expect(fake.calls.some((c) => c.startsWith('find('))).toBe(false);
	});

	it('finds the matches once typing pauses, and goes to the first', async () => {
		const { fake, hook, id } = await setup();
		act(() => hook.result.current.show(false));
		act(() => hook.result.current.setQuery('cat'));
		await waitFor(() => expect(hook.result.current.matches).toHaveLength(3));
		expect(fake.calls).toContain(`find("${id}", "cat")`);
		expect(hook.result.current.index).toBe(0);
		expect(hook.result.current.focus?.match.from).toBe(0);
	});

	it('steps to the next and previous match, round the ends, asking the page to go there', async () => {
		const { hook } = await setup();
		act(() => hook.result.current.show(false));
		act(() => hook.result.current.setQuery('cat'));
		await waitFor(() => expect(hook.result.current.index).toBe(0));
		const first = hook.result.current.focus!.nonce;
		act(() => hook.result.current.step(false));
		expect(hook.result.current.index).toBe(1);
		expect(hook.result.current.focus!.nonce).toBeGreaterThan(first);
		act(() => hook.result.current.step(true));
		act(() => hook.result.current.step(true));
		expect(hook.result.current.index).toBe(2);
		expect(hook.result.current.focus?.match.from).toBe(9);
	});

	it('keeps the current match when the book changes under it, and does not move the page', async () => {
		const { fake, hook, revision } = await setup();
		act(() => hook.result.current.show(false));
		act(() => hook.result.current.setQuery('cat'));
		await waitFor(() => expect(hook.result.current.index).toBe(0));
		act(() => hook.result.current.step(false));
		const nonce = hook.result.current.focus!.nonce;
		fake.found.splice(0, 1);
		hook.rerender({ revision: revision + 1, master: null });
		await waitFor(() => expect(hook.result.current.matches).toHaveLength(2));
		expect(hook.result.current.index).toBe(0);
		expect(hook.result.current.matches[0]!.from).toBe(5);
		expect(hook.result.current.focus!.nonce).toBe(nonce);
	});

	it('says why a search failed, and finds nothing', async () => {
		const { fake, hook } = await setup();
		fake.refuseNext('find', 'badSearch', 'that is not a pattern');
		act(() => hook.result.current.show(false));
		act(() => hook.result.current.setQuery('('));
		await waitFor(() =>
			expect(hook.result.current.error).toBe('That is not a pattern Folio can search for.'),
		);
		expect(hook.result.current.matches).toHaveLength(0);
		expect(hook.result.current.index).toBeNull();
	});

	it('replaces all, says how many were left alone, and searches again', async () => {
		const { fake, hook, id } = await setup();
		fake.skipped[0] = 1;
		fake.skipped[1] = 2;
		act(() => hook.result.current.show(true));
		act(() => hook.result.current.setQuery('cat'));
		act(() => hook.result.current.setReplacement('dog'));
		await waitFor(() => expect(hook.result.current.matches).toHaveLength(3));
		await act(async () => {
			await hook.result.current.replaceAll();
		});
		expect(fake.calls).toContain(`replace("${id}", "cat", "dog", "all")`);
		expect(hook.result.current.notice).toBe(
			'Replaced 3. Left alone: 3 in locked frames or on other master pages.',
		);
	});

	it('replaces just the current match', async () => {
		const { fake, hook, id } = await setup();
		act(() => hook.result.current.show(true));
		act(() => hook.result.current.setQuery('cat'));
		act(() => hook.result.current.setReplacement('dog'));
		await waitFor(() => expect(hook.result.current.index).toBe(0));
		act(() => hook.result.current.step(false));
		await act(async () => {
			await hook.result.current.replaceOne();
		});
		expect(fake.calls).toContain(`replace("${id}", "cat", "dog", "one")`);
		expect(hook.result.current.notice).toBe('Replaced 1.');
	});

	it('closes: no matches are marked and the page is not asked to go anywhere', async () => {
		const { hook } = await setup();
		act(() => hook.result.current.show(false));
		act(() => hook.result.current.setQuery('cat'));
		await waitFor(() => expect(hook.result.current.matches).toHaveLength(3));
		act(() => hook.result.current.close());
		expect(hook.result.current.open).toBe(false);
		expect(hook.result.current.focus).toBeNull();
		await waitFor(() => expect(hook.result.current.matches).toHaveLength(0));
	});
});
