// Tests that Close and Recovered ask with the system's message dialog when dialogs are windows
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { render, renderHook } from '@testing-library/react';
import type { ReactNode } from 'react';
import { describe, expect, it, vi } from 'vitest';
import type { BookSummary, Recoverable } from '@liminal-hq/folio-plugin-books';
import { useCloseQuestion } from '../app/CloseBookDialog';
import { useRecoveryQuestion } from '../app/RecoveredBookDialog';
import { DEFAULT_SETTINGS, createFakeSettingsClient } from '../settings/fakeSettingsClient';
import { SettingsProvider } from '../settings/SettingsProvider';
import type { Questions } from './questions';

function wrapper(ownWindows: boolean) {
	const client = createFakeSettingsClient({
		...DEFAULT_SETTINGS,
		dialogs: { ownWindows, sizes: {} },
	});
	return ({ children }: { children: ReactNode }) => (
		<SettingsProvider client={client}>{children}</SettingsProvider>
	);
}

const summary = { title: 'Pip' } as BookSummary;
const found = { title: 'Pip', savedAt: null } as unknown as Recoverable;

describe('the system questions', () => {
	it('asks Close with Save, Don’t save and Cancel, and returns the choice', async () => {
		const ask = vi.fn(async (q: { answers: { value: string }[] }) => q.answers[1]!.value);
		const questions = { ask } as unknown as Questions;
		const { result } = renderHook(() => useCloseQuestion(questions), { wrapper: wrapper(true) });
		const first = result.current.ask;
		await vi.waitFor(() => expect(result.current.ask).not.toBe(first));
		expect(await result.current.ask(summary)).toBe('discard');
		const asked = ask.mock.calls[0]![0] as unknown as {
			title: string;
			answers: { label: string }[];
		};
		expect(asked.title).toBe('Save changes to “Pip”?');
		expect(asked.answers.map((a) => a.label)).toEqual(['Save', 'Don’t save', 'Cancel']);
	});

	it('asks Recovered with the restore, saved-file and cancel answers', async () => {
		const ask = vi.fn(async (q: { answers: { value: string }[] }) => q.answers[0]!.value);
		const questions = { ask } as unknown as Questions;
		const { result } = renderHook(() => useRecoveryQuestion(questions), {
			wrapper: wrapper(true),
		});
		const first = result.current.ask;
		await vi.waitFor(() => expect(result.current.ask).not.toBe(first));
		expect(await result.current.ask(found)).toBe('restore');
		const asked = ask.mock.calls[0]![0] as { answers: { value: string }[] };
		expect(asked.answers.map((a) => a.value)).toEqual(['restore', 'file', 'cancel']);
	});

	it('keeps the in-page dialog while dialogs are not windows', () => {
		const ask = vi.fn();
		const { result } = renderHook(() => useCloseQuestion({ ask } as unknown as Questions), {
			wrapper: wrapper(false),
		});
		void result.current.ask(summary);
		expect(ask).not.toHaveBeenCalled();
		render(<>{result.current.dialog}</>, { wrapper: wrapper(false) });
	});
});
