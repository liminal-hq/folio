// Tests for the Update dialog
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { act, render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import type { UpdateState } from '@liminal-hq/folio-protocol/generated/UpdateState';
import { createFakeSettingsClient } from '../settings/fakeSettingsClient';
import { SettingsProvider } from '../settings/SettingsProvider';
import { createFakeUpdateClient } from '../services/fakeUpdateClient';
import { UpdateDialog } from './UpdateDialog';

async function setup(
	state: UpdateState = { kind: 'idle' },
	{ checkOnOpen = false, beforeInstall = async () => true } = {},
) {
	const settings = createFakeSettingsClient();
	const fake = createFakeUpdateClient(state);
	render(
		<SettingsProvider client={settings}>
			<UpdateDialog
				open
				onClose={() => undefined}
				client={fake.client}
				checkOnOpen={checkOnOpen}
				beforeInstall={beforeInstall}
			/>
		</SettingsProvider>,
	);
	return { fake, settings };
}

describe('the Update dialog', () => {
	it('asks nothing of github.com until told to', async () => {
		const { fake } = await setup();
		await screen.findByText(/asks github.com/);
		expect(fake.calls).not.toContain('check()');
	});

	it('looks at once when opened from the Help item', async () => {
		const { fake } = await setup({ kind: 'idle' }, { checkOnOpen: true });
		expect(await screen.findByText('This is the newest version of Folio.')).toBeInTheDocument();
		expect(fake.calls).toContain('check()');
	});

	it('says what is new and how it will be installed, then fetches it', async () => {
		const { fake } = await setup({
			kind: 'available',
			version: '0.2.0',
			notes: 'Fixes the cover.',
			date: null,
			install: 'deb',
		});
		expect(await screen.findByText('Folio 0.2.0 is ready to download.')).toBeInTheDocument();
		expect(screen.getByText('Fixes the cover.')).toBeInTheDocument();
		expect(screen.getByText(/ask for your password/)).toBeInTheDocument();
		await userEvent.click(screen.getByRole('button', { name: 'Download' }));
		expect(fake.calls).toContain('download()');
		expect(await screen.findByText(/Installing it restarts Folio/)).toBeInTheDocument();
	});

	it('asks about unsaved books before it installs, and stops if the person says no', async () => {
		const beforeInstall = vi.fn(async () => false);
		const { fake } = await setup({ kind: 'ready', version: '0.2.0' }, { beforeInstall });
		await userEvent.click(await screen.findByRole('button', { name: 'Restart and install' }));
		expect(beforeInstall).toHaveBeenCalled();
		expect(fake.calls).not.toContain('install()');
	});

	it('installs once the books are dealt with', async () => {
		const { fake } = await setup({ kind: 'ready', version: '0.2.0' });
		await userEvent.click(await screen.findByRole('button', { name: 'Restart and install' }));
		await waitFor(() => expect(fake.calls).toContain('install()'));
	});

	it('says why a copy cannot update itself, and offers no switch', async () => {
		await setup({ kind: 'unsupported', reason: 'This is a development build.' });
		expect(await screen.findByText('This is a development build.')).toBeInTheDocument();
		expect(screen.queryByRole('checkbox')).toBeNull();
	});

	it('turns on automatic checks, and counts opening it as having been asked', async () => {
		const { settings } = await setup();
		await userEvent.click(await screen.findByRole('checkbox', { name: /Look for updates/ }));
		await waitFor(async () => {
			const { settings: now } = await settings.get();
			expect(now.updates).toEqual({ automatic: true, asked: true });
		});
	});

	it('shows a failed check', async () => {
		const { fake } = await setup();
		await screen.findByText(/asks github.com/);
		act(() => fake.emit({ kind: 'error', message: 'offline' }));
		expect(await screen.findByRole('alert')).toHaveTextContent('Folio could not update: offline');
	});
});
