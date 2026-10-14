// Tests for the theme root
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { act, render } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { createFakeSettingsClient } from '../settings/fakeSettingsClient';
import { SettingsProvider } from '../settings/SettingsProvider';
import { ThemeRoot } from './ThemeRoot';

function mount(client = createFakeSettingsClient()) {
	const root = document.createElement('div');
	render(
		<SettingsProvider client={client}>
			<ThemeRoot root={root}>
				<span>page</span>
			</ThemeRoot>
		</SettingsProvider>,
	);
	return { root, client };
}

describe('ThemeRoot', () => {
	it('follows the system until a scheme is chosen', async () => {
		const { root, client } = mount();
		expect(root.dataset.theme).toBeUndefined();
		await act(async () => {
			await client.update({ colourMode: 'dark' });
		});
		expect(root.dataset.theme).toBe('dark');
		await act(async () => {
			await client.update({ colourMode: 'system' });
		});
		expect(root.dataset.theme).toBeUndefined();
	});

	it('marks larger controls on the root', async () => {
		const { root, client } = mount();
		expect(root.dataset.largerControls).toBeUndefined();
		await act(async () => {
			await client.update({ interface: { largerControls: true } });
		});
		expect(root.dataset.largerControls).toBe('true');
	});
});
