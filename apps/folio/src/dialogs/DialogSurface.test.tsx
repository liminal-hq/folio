// Tests that a dialog's frame is the page's dialog in the page and the whole window in a dialog window
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { WindowChromeProvider } from '@liminal-hq/folio-chrome/WindowChromeProvider/WindowChromeProvider';
import { render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { createFakeBooksClient } from '../books/fakeBooksClient';
import { ExportDialog } from '../app/ExportDialog';
import { DialogWindowProvider } from './DialogSurface';

const controls = {
	minimize: vi.fn(),
	toggleMaximize: vi.fn(),
	close: vi.fn(),
	setAlwaysOnTop: vi.fn(),
	isMaximized: vi.fn().mockResolvedValue(false),
	onMaximizedChange: () => () => {},
};

describe('a dialog in a dialog window', () => {
	it('fills the window with the dialog held open, and asks the host to close on Esc', async () => {
		const fake = createFakeBooksClient();
		const made = await fake.client.newBook({});
		const onClose = vi.fn();
		render(
			<WindowChromeProvider controls={controls}>
				<DialogWindowProvider>
					<ExportDialog
						open
						onClose={onClose}
						client={fake.client}
						id={made.summary.id}
						onCheck={() => {}}
					/>
				</DialogWindowProvider>
			</WindowChromeProvider>,
		);
		const dialog = screen.getByRole('dialog', { name: 'Export book' });
		expect(dialog).toHaveAttribute('open');
		expect(dialog.className).not.toContain('backdrop');
		expect(screen.getByRole('checkbox', { name: /Sharing PDF/ })).toBeInTheDocument();
		document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape' }));
		expect(onClose).toHaveBeenCalledWith('escape');
	});
});
