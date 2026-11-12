// Tests that the smart punctuation setting decides whether the editor's request for it is sent
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { EditRequest } from '@liminal-hq/folio-protocol/generated/EditRequest';
import { act, renderHook } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import type { BooksStore } from '../books/booksStore';
import { useTextEdits } from './useTextEdits';

const request: EditRequest = {
	target: { type: 'frame', frame: 'f' },
	from: { para: 0, offset: 0 },
	to: { para: 0, offset: 0 },
	insert: { type: 'text', text: '"' },
	step: 'typing',
	smart: true,
};

function sent(smartPunctuation: boolean) {
	const editText = vi.fn(async () => ({
		caret: { para: 0, offset: 1 },
		smart: { from: { para: 0, offset: 0 }, to: { para: 0, offset: 1 }, text: '“', raw: '"' },
	}));
	const store = { editText } as unknown as BooksStore;
	const { result } = renderHook(() => useTextEdits(store, 'sp', null, smartPunctuation));
	return { editText, result };
}

describe('useTextEdits', () => {
	it('passes the request for smart punctuation on while the setting is on, and hands back the note', async () => {
		const { editText, result } = sent(true);
		let answer: Awaited<ReturnType<typeof result.current.onEdit>> | undefined;
		await act(async () => {
			answer = await result.current.onEdit(request);
		});
		expect((editText.mock.calls[0] as unknown[])[0]).toMatchObject({ smart: true });
		expect(answer?.smart?.text).toBe('“');
	});

	it('leaves it out when the setting is off', async () => {
		const { editText, result } = sent(false);
		await act(async () => {
			await result.current.onEdit(request);
		});
		expect((editText.mock.calls[0] as unknown[])[0]).not.toHaveProperty('smart');
	});
});
