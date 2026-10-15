// Tests for the spelling in the editor: underlines, corrections, ignoring and finding the next
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi, type Mock } from 'vitest';
import { TextEditor, type EditedWords, type Speller } from './TextEditor';

const frame = (text: string): EditedWords => ({
	type: 'text',
	box: { w: 200, h: 80 },
	content: { inline: [{ text, style: null, bold: false, italic: false }] },
});

/** A speller that flags the words it is told are wrong, wherever they are. */
function speller(wrong: string[]): Speller & {
	ignore: Mock<(word: string) => void>;
	addToDictionary: Mock<(word: string) => void>;
} {
	return {
		check: async (text) =>
			[...text.matchAll(/\p{L}+/gu)]
				.filter((m) => wrong.includes(m[0]))
				.map((m) => ({ start: m.index!, end: m.index! + m[0].length, word: m[0] })),
		suggest: async (word) => (word === 'ottr' ? ['otter', 'utter'] : []),
		ignore: vi.fn<(word: string) => void>(),
		addToDictionary: vi.fn<(word: string) => void>(),
	};
}

const setup = (text: string, s: Speller | undefined, onCommit = vi.fn()) => {
	render(
		<TextEditor
			frame={frame(text)}
			layout={undefined}
			scale={1}
			at={{ x: 0, y: 0 }}
			onCommit={onCommit}
			speller={s}
		/>,
	);
	return onCommit;
};

const marked = () => [...document.querySelectorAll('[data-miss]')].map((e) => e.textContent);

describe('spelling in the editor', () => {
	it('underlines the misspelt words, says how many, and leaves the box itself clear', async () => {
		setup('A small ottr and a zorb', speller(['ottr', 'zorb']));
		await waitFor(() => expect(marked()).toEqual(['ottr', 'zorb']));
		expect(screen.getByRole('status')).toHaveTextContent('2 misspelt');
		// The underline is in a mirror behind the box, hidden from assistive technology.
		expect(document.querySelector('[data-miss]')!.closest('[aria-hidden="true"]')).not.toBeNull();
	});

	it('checks nothing and draws nothing without a speller', () => {
		setup('A small ottr', undefined);
		expect(document.querySelector('[data-miss]')).toBeNull();
		expect(screen.queryByRole('status')).toBeNull();
	});

	it('checks again a moment after typing', async () => {
		setup('The ottr', speller(['ottr', 'zorb']));
		await waitFor(() => expect(marked()).toEqual(['ottr']));
		await userEvent.type(screen.getByRole('textbox'), ' zorb');
		await waitFor(() => expect(marked()).toEqual(['ottr', 'zorb']));
	});

	it('offers corrections from the keyboard and puts the chosen one in, keeping the editor open', async () => {
		const onCommit = setup('A small ottr sat', speller(['ottr']));
		await waitFor(() => expect(marked()).toEqual(['ottr']));
		const box = screen.getByRole('textbox') as HTMLTextAreaElement;
		box.setSelectionRange(9, 9);
		fireEvent.keyDown(box, { key: 'F10', shiftKey: true });
		const menu = await screen.findByRole('menu', { name: 'Spelling' });
		expect(menu).toBeInTheDocument();
		await userEvent.click(screen.getByRole('menuitem', { name: 'otter' }));
		expect(box.value).toBe('A small otter sat');
		expect(onCommit).not.toHaveBeenCalled();
		await waitFor(() => expect(document.querySelector('[data-miss]')).toBeNull());
	});

	it('says when there is nothing to suggest, and ignores a word in the book', async () => {
		const s = speller(['zorb']);
		setup('A zorb here', s);
		await waitFor(() => expect(marked()).toEqual(['zorb']));
		const box = screen.getByRole('textbox') as HTMLTextAreaElement;
		box.setSelectionRange(3, 3);
		fireEvent.keyDown(box, { key: 'ContextMenu' });
		expect(await screen.findByRole('menuitem', { name: 'No suggestions' })).toHaveAttribute(
			'aria-disabled',
			'true',
		);
		await userEvent.click(screen.getByRole('menuitem', { name: 'Ignore in this book' }));
		expect(s.ignore).toHaveBeenCalledWith('zorb');
		await waitFor(() => expect(document.querySelector('[data-miss]')).toBeNull());
	});

	it('adds a word to the dictionary for every book', async () => {
		const s = speller(['zorb']);
		setup('A zorb here', s);
		await waitFor(() => expect(marked()).toEqual(['zorb']));
		const box = screen.getByRole('textbox') as HTMLTextAreaElement;
		box.setSelectionRange(3, 3);
		fireEvent.keyDown(box, { key: 'ContextMenu' });
		await userEvent.click(await screen.findByRole('menuitem', { name: 'Add to dictionary' }));
		expect(s.addToDictionary).toHaveBeenCalledWith('zorb');
		expect(s.ignore).not.toHaveBeenCalled();
		await waitFor(() => expect(document.querySelector('[data-miss]')).toBeNull());
	});

	it('does nothing from the keyboard when the caret is not in a misspelt word', async () => {
		setup('A small ottr', speller(['ottr']));
		await waitFor(() => expect(marked()).toEqual(['ottr']));
		const box = screen.getByRole('textbox') as HTMLTextAreaElement;
		box.setSelectionRange(1, 1);
		fireEvent.keyDown(box, { key: 'F10', shiftKey: true });
		expect(screen.queryByRole('menu')).toBeNull();
	});

	it('jumps to the next misspelling with Alt+F7, and round to the first', async () => {
		setup('ottr is not zorb', speller(['ottr', 'zorb']));
		await waitFor(() => expect(marked()).toEqual(['ottr', 'zorb']));
		const box = screen.getByRole('textbox') as HTMLTextAreaElement;
		box.setSelectionRange(0, 0);
		fireEvent.keyDown(box, { key: 'F7', altKey: true });
		expect([box.selectionStart, box.selectionEnd]).toEqual([12, 16]);
		fireEvent.keyDown(box, { key: 'F7', altKey: true });
		expect([box.selectionStart, box.selectionEnd]).toEqual([0, 4]);
	});
});
