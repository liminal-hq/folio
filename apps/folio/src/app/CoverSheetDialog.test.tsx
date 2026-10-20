// Tests for the cover sheet preview
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { fireEvent, render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import { createFakeBooksClient } from '../books/fakeBooksClient';
import { CoverSheetDialog } from './CoverSheetDialog';

describe('CoverSheetDialog', () => {
	it('draws the sheet the plugin makes', async () => {
		const fake = createFakeBooksClient();
		const made = await fake.client.newBook({});
		render(
			<CoverSheetDialog
				open
				onClose={() => undefined}
				client={fake.client}
				id={made.summary.id}
				revision={0}
				book={made.book}
				onSave={() => undefined}
				onNotes={() => undefined}
				onPickPicture={async () => null}
				onFrameBoxes={() => undefined}
				onFrameText={() => undefined}
				onFrameDelete={() => undefined}
			/>,
		);
		expect(
			await screen.findByRole('img', { name: 'The paperback cover sheet' }),
		).toBeInTheDocument();
		expect(fake.calls).toContain(`previewCoverSheet("${made.summary.id}")`);
	});

	it('is not drawn while closed', async () => {
		const fake = createFakeBooksClient();
		const made = await fake.client.newBook({});
		render(
			<CoverSheetDialog
				open={false}
				onClose={() => undefined}
				client={fake.client}
				id={made.summary.id}
				revision={0}
				book={made.book}
				onSave={() => undefined}
				onNotes={() => undefined}
				onPickPicture={async () => null}
				onFrameBoxes={() => undefined}
				onFrameText={() => undefined}
				onFrameDelete={() => undefined}
			/>,
		);
		expect(fake.calls.some((c) => c.startsWith('previewCoverSheet'))).toBe(false);
	});

	it('shows the guides over the sheet, and hides them when asked', async () => {
		const fake = createFakeBooksClient();
		const made = await fake.client.newBook({});
		render(
			<CoverSheetDialog
				open
				onClose={() => undefined}
				client={fake.client}
				id={made.summary.id}
				revision={0}
				book={made.book}
				onSave={() => undefined}
				onNotes={() => undefined}
				onPickPicture={async () => null}
				onFrameBoxes={() => undefined}
				onFrameText={() => undefined}
				onFrameDelete={() => undefined}
			/>,
		);
		expect(await screen.findByTestId('cover-guides')).toBeInTheDocument();
		await userEvent.click(screen.getByRole('checkbox', { name: /Show the folds/ }));
		expect(screen.queryByTestId('cover-guides')).toBeNull();
	});

	it('keeps new words for the back cover, the spine and the front as the book details', async () => {
		const fake = createFakeBooksClient();
		const made = await fake.client.newBook({});
		const onSave = vi.fn();
		render(
			<CoverSheetDialog
				open
				onClose={() => undefined}
				client={fake.client}
				id={made.summary.id}
				revision={0}
				book={made.book}
				onSave={onSave}
				onNotes={() => undefined}
				onPickPicture={async () => null}
				onFrameBoxes={() => undefined}
				onFrameText={() => undefined}
				onFrameDelete={() => undefined}
			/>,
		);
		const update = await screen.findByRole('button', { name: 'Update the cover' });
		expect(update).toBeDisabled();
		const blurb = screen.getByRole('textbox', { name: 'Back cover blurb' });
		await userEvent.clear(blurb);
		await userEvent.type(blurb, 'A cosy tale.');
		await userEvent.click(update);
		expect(onSave).toHaveBeenCalledTimes(1);
		expect(onSave.mock.calls[0]![0]).toMatchObject({ description: 'A cosy tale.' });
	});

	it('adds words to the back cover, and moves them by dragging the box', async () => {
		const fake = createFakeBooksClient();
		const made = await fake.client.newBook({});
		const onNotes = vi.fn();
		const note = {
			id: 'n1',
			text: 'Praise',
			rect: { x: 40, y: 50, w: 100, h: 30 },
			size: 12,
		};
		render(
			<CoverSheetDialog
				open
				onClose={() => undefined}
				client={fake.client}
				id={made.summary.id}
				revision={0}
				book={{ ...made.book, backNotes: [note] }}
				onSave={() => undefined}
				onNotes={onNotes}
				onPickPicture={async () => null}
				onFrameBoxes={() => undefined}
				onFrameText={() => undefined}
				onFrameDelete={() => undefined}
			/>,
		);
		await userEvent.click(
			await screen.findByRole('button', { name: 'Add words to the back cover' }),
		);
		expect(onNotes).toHaveBeenLastCalledWith([
			note,
			expect.objectContaining({ id: 'n2', text: 'New words', size: 12 }),
		]);
		const box = screen.getByRole('button', { name: 'Words on the back cover: Praise' });
		fireEvent.pointerDown(box, { clientX: 100, clientY: 100, pointerId: 1 });
		fireEvent.pointerMove(box, { clientX: 120, clientY: 90, pointerId: 1 });
		fireEvent.pointerUp(box, { pointerId: 1 });
		expect(onNotes).toHaveBeenLastCalledWith([{ ...note, rect: { x: 60, y: 40, w: 100, h: 30 } }]);
		const corner = screen.getByRole('button', { name: 'Drag to stretch the box' });
		fireEvent.pointerDown(corner, { clientX: 10, clientY: 10, pointerId: 2 });
		fireEvent.pointerMove(corner, { clientX: 40, clientY: 25, pointerId: 2 });
		fireEvent.pointerUp(corner, { pointerId: 2 });
		expect(onNotes).toHaveBeenLastCalledWith([{ ...note, rect: { x: 40, y: 50, w: 130, h: 45 } }]);
		await userEvent.clear(screen.getByRole('spinbutton', { name: 'Size in points' }));
		await userEvent.type(screen.getByRole('spinbutton', { name: 'Size in points' }), '9');
		await userEvent.tab();
		expect(onNotes).toHaveBeenLastCalledWith([{ ...note, size: 9 }]);
		const width = screen.getByRole('spinbutton', { name: 'Box width in points' });
		await userEvent.clear(width);
		await userEvent.type(width, '150');
		await userEvent.tab();
		expect(onNotes).toHaveBeenLastCalledWith([{ ...note, rect: { ...note.rect, w: 150 } }]);
	});

	it('adds a picture to the back cover as a box that can be moved like words', async () => {
		const fake = createFakeBooksClient();
		const made = await fake.client.newBook({});
		const onNotes = vi.fn();
		render(
			<CoverSheetDialog
				open
				onClose={() => undefined}
				client={fake.client}
				id={made.summary.id}
				revision={0}
				book={made.book}
				onSave={() => undefined}
				onNotes={onNotes}
				onPickPicture={async () => 'assets/images/otter.png'}
				onFrameBoxes={() => undefined}
				onFrameText={() => undefined}
				onFrameDelete={() => undefined}
			/>,
		);
		await userEvent.click(
			await screen.findByRole('button', { name: 'Add a picture to the back cover' }),
		);
		expect(onNotes).toHaveBeenCalledWith([
			expect.objectContaining({ id: 'n1', text: '', picture: 'assets/images/otter.png' }),
		]);
	});

	it('draws on the sheet as a page, where words, pictures and shapes can be added', async () => {
		const fake = createFakeBooksClient();
		const made = await fake.client.newBook({});
		const onNotes = vi.fn();
		render(
			<CoverSheetDialog
				open
				onClose={() => undefined}
				client={fake.client}
				id={made.summary.id}
				revision={0}
				book={made.book}
				onSave={() => undefined}
				onNotes={onNotes}
				onPickPicture={async () => 'assets/images/otter.png'}
				onFrameBoxes={() => undefined}
				onFrameText={() => undefined}
				onFrameDelete={() => undefined}
			/>,
		);
		await userEvent.click(await screen.findByRole('button', { name: 'Draw on the sheet' }));
		expect(await screen.findByRole('group', { name: 'The cover sheet' })).toBeInTheDocument();
		await userEvent.selectOptions(screen.getByRole('combobox', { name: 'Shape' }), 'ellipse');
		await userEvent.click(screen.getByRole('button', { name: 'Add the shape' }));
		expect(onNotes).toHaveBeenLastCalledWith([
			expect.objectContaining({ id: 'n1', shape: 'ellipse', fill: '#e8833a' }),
		]);
		await userEvent.click(screen.getByRole('button', { name: 'Add a picture to the back cover' }));
		expect(onNotes).toHaveBeenLastCalledWith([
			expect.objectContaining({ picture: 'assets/images/otter.png' }),
		]);
	});
});
