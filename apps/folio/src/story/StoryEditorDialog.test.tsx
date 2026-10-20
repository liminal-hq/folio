// Tests for the story editor
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { Book } from '@liminal-hq/folio-protocol/generated/Book';
import { fireEvent, render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import { createFakeBooksClient } from '../books/fakeBooksClient';
import type { Speller } from '../canvas/TextEditor';
import { StoryEditorDialog } from './StoryEditorDialog';

async function setup(focusParagraph?: number, chapters = false, speller?: Speller) {
	const fake = createFakeBooksClient();
	const made = await fake.client.newBook({});
	const book: Book = {
		...made.book,
		flows: [
			{
				id: 'fl',
				frames: ['a'],
				autoAddPages: true,
				paragraphs: [
					{ style: 'h1', inline: [{ text: 'A chapter', style: null, bold: false, italic: false }] },
					{ inline: [{ text: 'The otter slept.', style: null, bold: false, italic: false }] },
					...(chapters
						? [
								{
									style: 'h1',
									inline: [{ text: 'Second chapter', style: null, bold: false, italic: false }],
								},
							]
						: []),
				],
			},
		],
	};
	const apply = vi.fn();
	render(
		<StoryEditorDialog
			open
			onClose={() => undefined}
			book={book}
			flow="fl"
			focusParagraph={focusParagraph}
			speller={speller}
			choices={[
				{ id: 'h1', name: 'Heading', size: 20 },
				{ id: 'body', name: 'Body', size: 12 },
			]}
			apply={apply}
		/>,
	);
	return { apply };
}

describe('the story editor', () => {
	it('opens with the caret in the paragraph it was asked for', async () => {
		await setup(1);
		expect(screen.getByRole('textbox', { name: 'Paragraph 2' })).toHaveFocus();
	});

	it('lists the chapters and jumps to the one picked', async () => {
		await setup(undefined, true);
		const list = screen.getByRole('navigation', { name: 'Chapters:' });
		expect(list).toBeInTheDocument();
		await userEvent.click(screen.getByRole('button', { name: 'Second chapter' }));
		expect(screen.getByRole('textbox', { name: 'Paragraph 3' })).toHaveFocus();
	});

	it('underlines the misspelt words of a paragraph when spelling is on', async () => {
		const speller: Speller = {
			check: async (text) =>
				text === 'The otter slept.' ? [{ start: 4, end: 9, word: 'otter' }] : [],
			suggest: async () => [],
			ignore: () => undefined,
			addToDictionary: () => undefined,
		};
		await setup(undefined, false, speller);
		const miss = await screen.findByText('otter', { selector: 'span' });
		expect(miss.className).toMatch(/miss/);
	});

	it('lists the paragraphs with their styles', async () => {
		await setup();
		expect(screen.getByRole('textbox', { name: 'Paragraph 1' })).toHaveValue('A chapter');
		expect(screen.getByRole('textbox', { name: 'Paragraph 2' })).toHaveValue('The otter slept.');
		expect(screen.getByRole('combobox', { name: 'Style of paragraph 1' })).toHaveValue('h1');
	});

	it('sends the words of a paragraph as they are typed', async () => {
		const { apply } = await setup();
		await userEvent.type(screen.getByRole('textbox', { name: 'Paragraph 2' }), '!');
		expect(apply).toHaveBeenLastCalledWith({
			type: 'SetParagraphText',
			flow: 'fl',
			para: 1,
			text: 'The otter slept.!',
		});
	});

	it('splits at the caret on Enter and joins on Backspace at the start', async () => {
		const { apply } = await setup();
		const box = screen.getByRole('textbox', { name: 'Paragraph 2' }) as HTMLTextAreaElement;
		box.setSelectionRange(3, 3);
		fireEvent.keyDown(box, { key: 'Enter' });
		expect(apply).toHaveBeenLastCalledWith({
			type: 'SplitParagraph',
			flow: 'fl',
			para: 1,
			at: 3,
		});
		box.setSelectionRange(0, 0);
		fireEvent.keyDown(box, { key: 'Backspace' });
		expect(apply).toHaveBeenLastCalledWith({ type: 'JoinParagraphs', flow: 'fl', para: 0 });
	});

	it('sets a paragraph style', async () => {
		const { apply } = await setup();
		await userEvent.selectOptions(
			screen.getByRole('combobox', { name: 'Style of paragraph 2' }),
			'h1',
		);
		expect(apply).toHaveBeenLastCalledWith({
			type: 'StyleParagraphs',
			flow: 'fl',
			from: 1,
			to: 1,
			style: 'h1',
		});
	});
});
