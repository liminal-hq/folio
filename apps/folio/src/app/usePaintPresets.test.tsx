// Tests for reading the palette once, when it is first wanted
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { renderHook, waitFor } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import type { BooksClient } from '../books/booksClient';
import { FAKE_PAINTS } from '../books/fakeBooksClient';
import { usePaintPresets } from './usePaintPresets';

const clientWith = (read: () => Promise<typeof FAKE_PAINTS>) =>
	({ paintPresets: vi.fn(read) }) as unknown as BooksClient & {
		paintPresets: ReturnType<typeof vi.fn>;
	};

describe('usePaintPresets', () => {
	it('reads nothing until it is wanted, and the palette once after', async () => {
		const client = clientWith(async () => FAKE_PAINTS);
		const { result, rerender } = renderHook(({ wanted }) => usePaintPresets(client, wanted), {
			initialProps: { wanted: false },
		});
		expect(result.current).toBeNull();
		expect(client.paintPresets).not.toHaveBeenCalled();
		rerender({ wanted: true });
		await waitFor(() => expect(result.current).toEqual(FAKE_PAINTS));
		rerender({ wanted: false });
		rerender({ wanted: true });
		expect(client.paintPresets).toHaveBeenCalledTimes(1);
	});

	it('stays empty when the plugin cannot say', async () => {
		const client = clientWith(async () => {
			throw new Error('no');
		});
		const { result } = renderHook(() => usePaintPresets(client, true));
		await waitFor(() => expect(client.paintPresets).toHaveBeenCalled());
		expect(result.current).toBeNull();
	});
});
