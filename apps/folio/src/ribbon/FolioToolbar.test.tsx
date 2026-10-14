// Tests for the toolbar under the title bar: contextual tabs reach the ribbon
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import type { CommandId } from '../commands/appMenuModel';
import { createFakeSettingsClient } from '../settings/fakeSettingsClient';
import { SettingsProvider } from '../settings/SettingsProvider';
import { FolioToolbar } from './FolioToolbar';

const available = new Set<CommandId>();

function setup(contextual?: readonly string[]) {
	render(
		<SettingsProvider client={createFakeSettingsClient()}>
			<FolioToolbar available={available} run={async () => {}} contextual={contextual} />
		</SettingsProvider>,
	);
}

describe('FolioToolbar', () => {
	it('shows no contextual tab by default', async () => {
		setup();
		await screen.findByRole('tab', { name: 'Home' });
		expect(screen.queryByRole('tab', { name: 'Picture' })).not.toBeInTheDocument();
	});

	it('shows the Picture tab when it is listed as contextual', async () => {
		setup(['picture']);
		expect(await screen.findByRole('tab', { name: 'Picture' })).toBeInTheDocument();
	});
});
