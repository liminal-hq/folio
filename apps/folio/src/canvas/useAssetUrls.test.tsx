// Tests for reading a spread's pictures from the book once each
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { renderHook, waitFor } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { createFakeBooksClient } from '../books/fakeBooksClient';
import { mediaTypeOf, useAssetUrls } from './useAssetUrls';

afterEach(() => vi.restoreAllMocks());

describe('mediaTypeOf', () => {
	it('names the type from the extension', () => {
		expect(mediaTypeOf('assets/images/a.JPG')).toBe('image/jpeg');
		expect(mediaTypeOf('assets/images/a.svg')).toBe('image/svg+xml');
		expect(mediaTypeOf('assets/images/a.xyz')).toBe('application/octet-stream');
	});
});

describe('useAssetUrls', () => {
	it('reads each picture once and gives an address for it', async () => {
		const create = vi.spyOn(URL, 'createObjectURL').mockReturnValue('blob:one');
		vi.spyOn(URL, 'revokeObjectURL').mockImplementation(() => {});
		const fake = createFakeBooksClient();
		const book = await fake.client.newBook({});
		const id = book.summary.id;
		const { result, rerender } = renderHook(({ assets }) => useAssetUrls(fake.client, id, assets), {
			initialProps: { assets: ['assets/images/a.png', 'assets/images/a.png'] },
		});
		await waitFor(() => expect(result.current['assets/images/a.png']).toBe('blob:one'));
		rerender({ assets: ['assets/images/a.png'] });
		expect(create).toHaveBeenCalledTimes(1);
		expect(fake.calls.filter((c) => c.startsWith('assetData')).length).toBe(1);
	});

	it('lets the addresses go when it closes, and has none for another book', async () => {
		vi.spyOn(URL, 'createObjectURL').mockReturnValue('blob:two');
		const revoke = vi.spyOn(URL, 'revokeObjectURL').mockImplementation(() => {});
		const fake = createFakeBooksClient();
		const id = (await fake.client.newBook({})).summary.id;
		const { result, unmount } = renderHook(() =>
			useAssetUrls(fake.client, id, ['assets/images/b.png']),
		);
		await waitFor(() => expect(result.current['assets/images/b.png']).toBe('blob:two'));
		unmount();
		expect(revoke).toHaveBeenCalledWith('blob:two');
	});
});
