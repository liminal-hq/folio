// Tests for the cover sheet preview
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { createFakeBooksClient } from '../books/fakeBooksClient';
import { CoverSheetDialog } from './CoverSheetDialog';

describe('CoverSheetDialog', () => {
	it('draws the sheet the plugin makes', async () => {
		const fake = createFakeBooksClient();
		const made = await fake.client.newBook({});
		render(
			<CoverSheetDialog
				open
				onClose={() => undefined}
				client={fake.client}
				id={made.summary.id}
				revision={0}
			/>,
		);
		expect(
			await screen.findByRole('img', { name: 'The paperback cover sheet' }),
		).toBeInTheDocument();
		expect(fake.calls).toContain(`previewCoverSheet("${made.summary.id}")`);
	});

	it('is not drawn while closed', async () => {
		const fake = createFakeBooksClient();
		const made = await fake.client.newBook({});
		render(
			<CoverSheetDialog
				open={false}
				onClose={() => undefined}
				client={fake.client}
				id={made.summary.id}
				revision={0}
			/>,
		);
		expect(fake.calls.some((c) => c.startsWith('previewCoverSheet'))).toBe(false);
	});
});
