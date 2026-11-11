// Tests the dialogs that hand a result back, and the cover sheet's back-cover change, as dialog windows
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { WindowChromeProvider } from '@liminal-hq/folio-chrome/WindowChromeProvider/WindowChromeProvider';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import { putDescriptionIntent } from '../app/coverSheetIntents';
import { ImportReportDialogWindow } from '../app/ImportReportDialogWindow';
import { SoftProofDialogWindow } from '../app/SoftProofDialogWindow';
import { SpreadTemplatesDialogWindow } from '../app/SpreadTemplatesDialogWindow';
import { createFakeBooksClient } from '../books/fakeBooksClient';
import { createFakeSettingsClient } from '../settings/fakeSettingsClient';
import { SettingsProvider } from '../settings/SettingsProvider';
import { DialogWindowProvider } from './DialogSurface';
import type { DialogSession } from './DialogHost';

const controls = {
	minimize: vi.fn(),
	toggleMaximize: vi.fn(),
	close: vi.fn(),
	setAlwaysOnTop: vi.fn(),
	isMaximized: vi.fn().mockResolvedValue(false),
	onMaximizedChange: () => () => {},
};

function inWindow(ui: React.ReactNode) {
	render(
		<SettingsProvider client={createFakeSettingsClient()}>
			<WindowChromeProvider controls={controls}>
				<DialogWindowProvider>{ui}</DialogWindowProvider>
			</WindowChromeProvider>
		</SettingsProvider>,
	);
}

const session = () => ({ close: vi.fn(), finish: vi.fn() }) as unknown as DialogSession;

describe('dialogs that hand a result back', () => {
	it('Spread templates ends with the template chosen, for the main window to add', async () => {
		const fake = createFakeBooksClient();
		const made = await fake.client.newBook({});
		const s = session();
		inWindow(
			<SpreadTemplatesDialogWindow
				payload={{ bookId: made.summary.id }}
				session={s}
				client={fake.client}
			/>,
		);
		await userEvent.click(await screen.findByRole('button', { name: /Picture and words/ }));
		expect(s.finish).toHaveBeenCalledWith({ template: 'pb-split', name: 'Picture and words' });
	});

	it('Import report ends with the styles chosen when the file is brought in again', async () => {
		const s = session();
		inWindow(
			<ImportReportDialogWindow
				payload={{
					outcome: {
						report: {
							paragraphs: 3,
							headings: 1,
							sceneBreaks: 0,
							notes: [],
							styles: [{ name: 'Caption', count: 2, sample: 'A picture', folio: null }],
						},
					},
				}}
				session={s}
			/>,
		);
		await userEvent.selectOptions(
			screen.getByRole('combobox', { name: 'What Caption becomes' }),
			'epigraph',
		);
		await userEvent.click(screen.getByRole('button', { name: 'Bring it in again' }));
		expect(s.finish).toHaveBeenCalledWith({ mapping: { Caption: 'epigraph' } });
	});

	it('Soft proof shows for the spread it was opened on and closes with Esc', async () => {
		const fake = createFakeBooksClient();
		const made = await fake.client.newBook({});
		const s = session();
		inWindow(
			<SoftProofDialogWindow
				payload={{ bookId: made.summary.id, spread: 'x' }}
				session={s}
				client={fake.client}
			/>,
		);
		expect(screen.getByRole('dialog')).toBeInTheDocument();
		screen.getAllByRole('button', { name: 'Close' }).at(-1)!.focus();
		await userEvent.keyboard('{Escape}');
		expect(s.close).toHaveBeenCalled();
	});
});

describe('putDescriptionIntent', () => {
	const book = (frames: { id: string }[]) =>
		({ trim: { w: 612, h: 612 }, spreads: [{ id: 'cover', frames }] }) as never;

	it('changes the words of the back cover frame when there is one', () => {
		expect(putDescriptionIntent(book([{ id: 'blurb' }]), 'Hello')).toEqual({
			type: 'SetText',
			frame: 'blurb',
			text: 'Hello',
		});
	});

	it('adds the frame when the back cover has none', () => {
		const intent = putDescriptionIntent(book([]), 'Hello');
		expect(intent).toMatchObject({ type: 'AddFrame', on: { spread: 'cover' } });
	});

	it('does nothing for a book with no cover', () => {
		expect(putDescriptionIntent({ spreads: [] } as never, 'x')).toBeNull();
	});
});
