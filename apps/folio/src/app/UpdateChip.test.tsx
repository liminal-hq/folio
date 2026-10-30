// Tests for the toolbar chip that speaks up about updates
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { act, render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import type { UpdateState } from '@liminal-hq/folio-protocol/generated/UpdateState';
import { createFakeSettingsClient } from '../settings/fakeSettingsClient';
import { SettingsProvider } from '../settings/SettingsProvider';
import { createFakeUpdateClient } from '../services/fakeUpdateClient';
import { UpdateChip } from './UpdateChip';
import { UPDATE_EVENT } from './updateRequest';

async function setup(state: UpdateState, asked = true) {
	const settings = createFakeSettingsClient();
	await settings.update({ updates: { asked } } as never);
	const fake = createFakeUpdateClient(state);
	render(
		<SettingsProvider client={settings}>
			<UpdateChip client={fake.client} />
		</SettingsProvider>,
	);
	// Let the first answers from the app and the settings arrive.
	await act(async () => {});
	return fake;
}

const available: UpdateState = {
	kind: 'available',
	version: '0.2.0',
	notes: null,
	date: null,
	install: 'deb',
};

describe('the update chip', () => {
	it('says nothing when there is nothing to do', async () => {
		await setup({ kind: 'upToDate' });
		expect(screen.queryByRole('button')).toBeNull();
		expect(screen.queryByRole('progressbar')).toBeNull();
	});

	it('says nothing for a copy that cannot update itself', async () => {
		await setup({ kind: 'unsupported', reason: 'A development build.' }, false);
		expect(screen.queryByRole('button')).toBeNull();
	});

	it('names the new version and opens the dialog when pressed', async () => {
		const opened = vi.fn();
		window.addEventListener(UPDATE_EVENT, opened);
		await setup(available);
		await userEvent.click(await screen.findByRole('button', { name: 'Update 0.2.0' }));
		expect(opened).toHaveBeenCalledTimes(1);
		window.removeEventListener(UPDATE_EVENT, opened);
	});

	it('shows how far a download has got', async () => {
		const fake = await setup({ kind: 'idle' });
		act(() => fake.emit({ kind: 'downloading', received: 50, total: 200 }));
		const bar = await screen.findByRole('progressbar');
		expect(bar).toHaveAttribute('aria-valuenow', '25');
		expect(bar).toHaveTextContent('Downloading 25%');
	});

	it('offers a restart when the update is ready', async () => {
		await setup({ kind: 'ready', version: '0.2.0' });
		expect(await screen.findByRole('button', { name: 'Restart to update' })).toBeInTheDocument();
	});

	it('offers automatic checks once, to someone who has not been asked', async () => {
		await setup({ kind: 'idle' }, false);
		expect(await screen.findByRole('button', { name: 'Updates…' })).toBeInTheDocument();
	});

	it('does not offer them again once asked', async () => {
		await setup({ kind: 'idle' }, true);
		expect(screen.queryByRole('button')).toBeNull();
	});
});
