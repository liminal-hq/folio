// Tests for the New book wizard window shell
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { WindowControls } from '@liminal-hq/folio-chrome/TitleBar/windowControls';
import { WindowChromeProvider } from '@liminal-hq/folio-chrome/WindowChromeProvider/WindowChromeProvider';
import { render, screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import { createFakeBooksClient } from '../books/fakeBooksClient';
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
				<NewBookWizard client={createFakeBooksClient().client} />
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

	it('returns to a step already met when it is clicked, and not to one ahead', async () => {
		mount();
		expect(screen.getByRole('button', { name: /Size and paper/ })).toBeDisabled();
		await userEvent.click(screen.getByRole('button', { name: 'Next' }));
		await userEvent.click(screen.getByRole('button', { name: 'Next' }));
		await userEvent.click(screen.getByRole('button', { name: /What are you making\?/ }));
		expect(screen.getByRole('heading', { name: 'What are you making?' })).toBeInTheDocument();
		await userEvent.click(screen.getByRole('button', { name: /Title and you/ }));
		expect(screen.getByRole('heading', { name: 'Title and you' })).toBeInTheDocument();
		expect(screen.getByRole('button', { name: /Bring your words/ })).toBeDisabled();
	});

	it('keeps a step marked as visited after going back, and not before it is reached', async () => {
		mount();
		const marked = (name: RegExp) =>
			within(screen.getByRole('button', { name })).queryByText('(visited)') !== null;
		expect(marked(/Size and paper/)).toBe(false);
		await userEvent.click(screen.getByRole('button', { name: 'Next' }));
		await userEvent.click(screen.getByRole('button', { name: 'Next' }));
		expect(marked(/What are you making\?/)).toBe(true);
		expect(marked(/Size and paper/)).toBe(true);
		expect(marked(/Title and you/)).toBe(false);
		await userEvent.click(screen.getByRole('button', { name: /What are you making\?/ }));
		expect(marked(/What are you making\?/)).toBe(false);
		expect(marked(/Size and paper/)).toBe(true);
		expect(marked(/Title and you/)).toBe(true);
		expect(marked(/Bring your words/)).toBe(false);
	});

	it('closes the window from Skip the wizard', async () => {
		const close = vi.fn();
		mount(close);
		await userEvent.click(screen.getByRole('button', { name: 'Skip the wizard' }));
		expect(close).toHaveBeenCalled();
	});
});

describe('the kind and size steps', () => {
	it('offers the six kinds and starts on the picture book', async () => {
		mount();
		expect(await screen.findAllByRole('radio')).toHaveLength(6);
		expect(screen.getByRole('radio', { name: /Picture book/ })).toBeChecked();
	});

	it('offers the sizes of the chosen kind, starting again from the first when the kind changes', async () => {
		mount();
		await userEvent.click(await screen.findByRole('radio', { name: /Novel/ }));
		await userEvent.click(screen.getByRole('button', { name: 'Next' }));
		expect(screen.getByRole('radio', { name: '5 × 8 in' })).toBeChecked();
		await userEvent.click(screen.getByRole('radio', { name: '6 × 9 in' }));
		expect(screen.getByRole('radio', { name: '6 × 9 in' })).toBeChecked();
		await userEvent.click(screen.getByRole('radio', { name: 'Cream' }));
		expect(screen.getByRole('radio', { name: 'Cream' })).toBeChecked();
		await userEvent.click(screen.getByRole('button', { name: /What are you making\?/ }));
		await userEvent.click(screen.getByRole('radio', { name: /Poetry/ }));
		await userEvent.click(screen.getByRole('button', { name: 'Next' }));
		expect(screen.getByRole('radio', { name: '5.5 × 8.5 in' })).toBeChecked();
		expect(screen.getByRole('radio', { name: 'Cream' })).toBeChecked();
	});
});

async function toStep(name: RegExp) {
	for (let n = 0; n < 6; n += 1) {
		if (screen.queryByRole('heading', { name }) !== null) return;
		await userEvent.click(screen.getByRole('button', { name: 'Next' }));
	}
}

describe('the title, words and way steps', () => {
	it('takes a title, an author and an ISBN, and flags a bad ISBN as it is typed', async () => {
		mount();
		await toStep(/Title and you/);
		await userEvent.type(screen.getByLabelText('Title'), 'Pip and the Moon');
		await userEvent.type(screen.getByLabelText('Author'), 'Mom');
		expect(screen.getByLabelText('Title')).toHaveValue('Pip and the Moon');
		const isbn = screen.getByLabelText('ISBN (optional)');
		await userEvent.type(isbn, '9780306406158');
		expect(screen.getByRole('alert')).toHaveTextContent('not a valid ISBN-13');
		await userEvent.clear(isbn);
		await userEvent.type(isbn, '978-0-306-40615-7');
		expect(screen.queryByRole('alert')).not.toBeInTheDocument();
	});

	it('chooses sample pages or an empty start, with importing not available yet', async () => {
		mount();
		await toStep(/Bring your words/);
		expect(screen.getByRole('radio', { name: /Start with sample pages/ })).toBeChecked();
		await userEvent.click(screen.getByRole('radio', { name: /Start empty/ }));
		expect(screen.getByRole('radio', { name: /Start empty/ })).toBeChecked();
		expect(screen.getByRole('radio', { name: /Bring in a document/ })).toBeDisabled();
	});

	it('starts from the saved way of working and lets the person change it', async () => {
		mount();
		await toStep(/How you like to work/);
		expect(screen.getByRole('radio', { name: 'Easy' })).toBeChecked();
		await userEvent.click(screen.getByRole('radio', { name: 'Expert' }));
		expect(screen.getByRole('radio', { name: 'Expert' })).toBeChecked();
		await userEvent.click(screen.getByRole('radio', { name: 'No thanks' }));
		expect(screen.getByRole('radio', { name: 'No thanks' })).toBeChecked();
	});
});
