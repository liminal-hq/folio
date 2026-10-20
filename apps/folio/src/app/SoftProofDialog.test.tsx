// Tests for the Soft proof dialog
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it } from 'vitest';
import { createFakeBooksClient } from '../books/fakeBooksClient';
import { SoftProofDialog } from './SoftProofDialog';

describe('SoftProofDialog', () => {
	it('needs a profile, then shows the proof and how much the press cannot reproduce', async () => {
		const fake = createFakeBooksClient();
		const made = await fake.client.newBook({});
		render(
			<SoftProofDialog
				open
				onClose={() => undefined}
				client={fake.client}
				id={made.summary.id}
				spread="sp1"
			/>,
		);
		const show = screen.getByRole('button', { name: 'Show proof' });
		expect(show).toBeDisabled();
		await userEvent.type(screen.getByLabelText(/CMYK profile/), '/tmp/press.icc');
		await userEvent.click(show);
		expect(await screen.findByText(/About 12.5% of this page/)).toBeInTheDocument();
		expect(fake.calls).toContain(`proofSpread("${made.summary.id}", "sp1", "/tmp/press.icc")`);
		await userEvent.click(screen.getByRole('checkbox', { name: 'As drawn' }));
		expect(screen.getByRole('img', { name: /As drawn/ })).toBeInTheDocument();
	});

	it('says why a proof could not be made', async () => {
		const fake = createFakeBooksClient();
		const made = await fake.client.newBook({});
		fake.refuseNext('proofSpread', 'export', 'not a profile');
		render(
			<SoftProofDialog
				open
				onClose={() => undefined}
				client={fake.client}
				id={made.summary.id}
				spread="sp1"
			/>,
		);
		await userEvent.type(screen.getByLabelText(/CMYK profile/), 'x');
		await userEvent.click(screen.getByRole('button', { name: 'Show proof' }));
		expect(await screen.findByRole('alert')).toHaveTextContent('not a profile');
	});
});
