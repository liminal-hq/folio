// Tests for the clock that asks for recovery copies
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { act, render } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { BooksProvider } from '../books/BooksProvider';
import type { FileDialogs } from '../books/dialogs';
import { createFakeBooksClient } from '../books/fakeBooksClient';
import { createFakeSettingsClient } from '../settings/fakeSettingsClient';
import { SettingsProvider } from '../settings/SettingsProvider';
import { useAutosave } from './useAutosave';

const dialogs: FileDialogs = {
	pickBook: async () => null,
	pickSavePath: async () => null,
	pickImage: async () => null,
	pickText: async () => null,
};

function Clock() {
	useAutosave();
	return null;
}

const asked = (calls: string[]) => calls.filter((c) => c === 'autosave()').length;

async function start() {
	vi.useFakeTimers();
	const settings = createFakeSettingsClient();
	const books = createFakeBooksClient();
	await act(async () => {
		render(
			<SettingsProvider client={settings}>
				<BooksProvider client={books.client} dialogs={dialogs}>
					<Clock />
				</BooksProvider>
			</SettingsProvider>,
		);
	});
	return { settings, books };
}

afterEach(() => vi.useRealTimers());

describe('useAutosave', () => {
	it('asks for a copy every thirty seconds by default', async () => {
		const { books } = await start();
		await act(async () => vi.advanceTimersByTimeAsync(29_000));
		expect(asked(books.calls)).toBe(0);
		await act(async () => vi.advanceTimersByTimeAsync(1_000));
		expect(asked(books.calls)).toBe(1);
		await act(async () => vi.advanceTimersByTimeAsync(60_000));
		expect(asked(books.calls)).toBe(3);
	});

	it('follows the interval the settings give, and stops when autosave is turned off', async () => {
		const { settings, books } = await start();
		await act(async () => {
			await settings.update({ autosave: { intervalSecs: 300 } } as never);
		});
		await act(async () => vi.advanceTimersByTimeAsync(299_000));
		expect(asked(books.calls)).toBe(0);
		await act(async () => vi.advanceTimersByTimeAsync(1_000));
		expect(asked(books.calls)).toBe(1);

		await act(async () => {
			await settings.update({ autosave: { enabled: false } } as never);
		});
		await act(async () => vi.advanceTimersByTimeAsync(900_000));
		expect(asked(books.calls)).toBe(1);
	});

	it('keeps asking when a copy could not be written', async () => {
		const { books } = await start();
		books.refuseNext('autosave', 'recovery', 'the disk is full');
		await act(async () => vi.advanceTimersByTimeAsync(30_000));
		await act(async () => vi.advanceTimersByTimeAsync(30_000));
		expect(asked(books.calls)).toBe(2);
	});
});
