// Tests for the link dialog
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import { LinkDialog, type LinkDialogProps } from './LinkDialog';

function setup(extra: Partial<LinkDialogProps> = {}) {
	const onSet = vi.fn();
	const onRemove = vi.fn();
	const onClose = vi.fn();
	render(
		<LinkDialog
			open
			onClose={onClose}
			words="our site"
			href=""
			editing={false}
			mixed={false}
			hasWords
			spreads={['Cover', 'Pages 2–3', 'Pages 4–5']}
			onSet={onSet}
			onRemove={onRemove}
			{...extra}
		/>,
	);
	return { onSet, onRemove, onClose };
}

describe('LinkDialog', () => {
	it('sets a link, making an address of what is typed', async () => {
		const { onSet, onClose } = setup();
		expect(screen.getByText(/“our site”/)).toBeInTheDocument();
		expect(screen.getByRole('button', { name: 'Set link' })).toBeDisabled();
		await userEvent.type(screen.getByLabelText('Where should it go?'), 'example.org');
		await userEvent.click(screen.getByRole('button', { name: 'Set link' }));
		expect(onSet).toHaveBeenCalledWith('https://example.org');
		expect(onClose).toHaveBeenCalled();
	});

	it('refuses an address that runs code and says why', async () => {
		const { onSet } = setup();
		await userEvent.type(screen.getByLabelText('Where should it go?'), 'javascript:alert(1)');
		expect(screen.getByRole('alert')).toHaveTextContent(/only links to web, email and phone/);
		expect(screen.getByRole('button', { name: 'Set link' })).toBeDisabled();
		expect(onSet).not.toHaveBeenCalled();
	});

	it('links to a page of the book from the list', async () => {
		const { onSet } = setup();
		await userEvent.selectOptions(screen.getByLabelText('Or a page in this book'), '3');
		expect(screen.getByLabelText('Where should it go?')).toHaveValue('#spread-3');
		await userEvent.click(screen.getByRole('button', { name: 'Set link' }));
		expect(onSet).toHaveBeenCalledWith('#spread-3');
	});

	it('shows the link it edits, and takes it away', async () => {
		const { onSet, onRemove } = setup({ editing: true, href: 'https://example.org/a' });
		expect(screen.getByRole('heading', { name: 'Edit link' })).toBeInTheDocument();
		expect(screen.getByLabelText('Where should it go?')).toHaveValue('https://example.org/a');
		await userEvent.click(screen.getByRole('button', { name: 'Remove link' }));
		expect(onRemove).toHaveBeenCalled();
		expect(onSet).not.toHaveBeenCalled();
	});

	it('asks for words to be selected when there are none', () => {
		setup({ hasWords: false });
		expect(screen.getByRole('status')).toHaveTextContent(
			'Select the words you want to link first.',
		);
		expect(screen.queryByLabelText('Where should it go?')).toBeNull();
	});
});
