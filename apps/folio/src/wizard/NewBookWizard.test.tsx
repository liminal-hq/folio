// Tests for the New book wizard window shell
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { WindowControls } from '@liminal-hq/folio-chrome/TitleBar/windowControls';
import { WindowChromeProvider } from '@liminal-hq/folio-chrome/WindowChromeProvider/WindowChromeProvider';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import { createFakeSettingsClient } from '../settings/fakeSettingsClient';
import { SettingsProvider } from '../settings/SettingsProvider';
import { NewBookWizard } from './NewBookWizard';

function mount(close = () => {}) {
	const controls: WindowControls = {
		minimize: () => {},
		toggleMaximize: () => {},
		close,
		setAlwaysOnTop: () => {},
		isMaximized: () => false,
		onMaximizedChange: () => () => {},
	};
	render(
		<SettingsProvider client={createFakeSettingsClient()}>
			<WindowChromeProvider controls={controls}>
				<NewBookWizard />
			</WindowChromeProvider>
		</SettingsProvider>,
	);
}

describe('NewBookWizard', () => {
	it('starts on the first step with Back unavailable', () => {
		mount();
		expect(screen.getByRole('heading', { name: 'What are you making?' })).toBeInTheDocument();
		expect(screen.getByRole('button', { name: 'Back' })).toBeDisabled();
		expect(
			screen.getByText('What are you making?', { selector: 'span' }).closest('li'),
		).toHaveAttribute('aria-current', 'step');
	});

	it('moves forward and back through every step, stopping at the review', async () => {
		mount();
		for (const name of [
			'Size and paper',
			'Title and you',
			'Bring your words',
			'How you like to work',
			'The look',
			'Review',
		]) {
			await userEvent.click(screen.getByRole('button', { name: 'Next' }));
			expect(screen.getByRole('heading', { name })).toBeInTheDocument();
		}
		expect(screen.getByRole('button', { name: 'Next' })).toBeDisabled();
		await userEvent.click(screen.getByRole('button', { name: 'Back' }));
		expect(screen.getByRole('heading', { name: 'The look' })).toBeInTheDocument();
	});

	it('closes the window from Skip the wizard', async () => {
		const close = vi.fn();
		mount(close);
		await userEvent.click(screen.getByRole('button', { name: 'Skip the wizard' }));
		expect(close).toHaveBeenCalled();
	});
});
