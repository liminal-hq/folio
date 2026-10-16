// Tests for the undo steps words are kept in while they are typed: a pause, a paste, a correction
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { act, fireEvent, render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { keepWords } from './editCheckpoint';
import { TextEditor, type EditedWords, type Speller } from './TextEditor';

const frame = (text: string): EditedWords => ({
	type: 'text',
	box: { w: 200, h: 80 },
	content: { inline: [{ text, style: null, bold: false, italic: false }] },
});

const setup = (text: string, speller?: Speller) => {
	const onCommit = vi.fn();
	const onCheckpoint = vi.fn();
	render(
		<TextEditor
			frame={frame(text)}
			layout={undefined}
			scale={1}
			at={{ x: 0, y: 0 }}
			onCommit={onCommit}
			onCheckpoint={onCheckpoint}
			speller={speller}
		/>,
	);
	return { onCommit, onCheckpoint, box: screen.getByRole('textbox') as HTMLTextAreaElement };
};

afterEach(() => vi.useRealTimers());

describe('keeping typed words as undo steps', () => {
	it('keeps the words after a pause in typing, and again after the next pause', () => {
		vi.useFakeTimers();
		const { onCheckpoint, onCommit, box } = setup('Once');
		fireEvent.change(box, { target: { value: 'Once upon' } });
		act(() => vi.advanceTimersByTime(500));
		fireEvent.change(box, { target: { value: 'Once upon a time' } });
		// Typing again before the pause ends starts the wait over.
		act(() => vi.advanceTimersByTime(900));
		expect(onCheckpoint).not.toHaveBeenCalled();
		act(() => vi.advanceTimersByTime(200));
		expect(onCheckpoint).toHaveBeenCalledTimes(1);
		expect(onCheckpoint).toHaveBeenLastCalledWith('Once upon a time');

		fireEvent.change(box, { target: { value: 'Once upon a time, there' } });
		act(() => vi.advanceTimersByTime(1000));
		expect(onCheckpoint).toHaveBeenCalledTimes(2);
		expect(onCheckpoint).toHaveBeenLastCalledWith('Once upon a time, there');
		// The editor is still open: nothing was committed.
		expect(onCommit).not.toHaveBeenCalled();
	});

	it('commits nothing when the last step already held every word', () => {
		vi.useFakeTimers();
		const { onCommit, box } = setup('Once');
		fireEvent.change(box, { target: { value: 'Once upon' } });
		act(() => vi.advanceTimersByTime(1000));
		fireEvent.keyDown(box, { key: 'Escape' });
		expect(onCommit).toHaveBeenCalledWith(null);
	});

	it('commits the words typed since the last step', () => {
		vi.useFakeTimers();
		const { onCommit, onCheckpoint, box } = setup('Once');
		fireEvent.change(box, { target: { value: 'Once upon' } });
		act(() => vi.advanceTimersByTime(1000));
		fireEvent.change(box, { target: { value: 'Once upon a' } });
		fireEvent.keyDown(box, { key: 'Escape' });
		expect(onCheckpoint).toHaveBeenCalledTimes(1);
		expect(onCommit).toHaveBeenCalledWith('Once upon a');
	});

	it('keeps nothing when nothing was typed', () => {
		vi.useFakeTimers();
		const { onCheckpoint, onCommit } = setup('Once');
		act(() => vi.advanceTimersByTime(5000));
		expect(onCheckpoint).not.toHaveBeenCalled();
		keepWords();
		expect(onCheckpoint).not.toHaveBeenCalled();
		expect(onCommit).not.toHaveBeenCalled();
	});

	it('keeps the words before a paste and the pasted words after it, as steps of their own', () => {
		const { onCheckpoint, box } = setup('Once');
		fireEvent.change(box, { target: { value: 'Once upon' } });
		fireEvent.paste(box);
		expect(onCheckpoint).toHaveBeenCalledTimes(1);
		expect(onCheckpoint).toHaveBeenLastCalledWith('Once upon');

		fireEvent.input(box, {
			target: { value: 'Once upon a pasted line' },
			inputType: 'insertFromPaste',
		});
		expect(onCheckpoint).toHaveBeenCalledTimes(2);
		expect(onCheckpoint).toHaveBeenLastCalledWith('Once upon a pasted line');
	});

	it('keeps the words when something else is about to change the book, and closes on an undo', () => {
		const { onCheckpoint, onCommit, box } = setup('Once');
		fireEvent.change(box, { target: { value: 'Once upon' } });
		keepWords();
		expect(onCheckpoint).toHaveBeenCalledWith('Once upon');
		expect(onCommit).not.toHaveBeenCalled();

		fireEvent.change(box, { target: { value: 'Once upon a' } });
		keepWords(true);
		expect(onCommit).toHaveBeenCalledWith('Once upon a');
	});

	it('keeps the words before a correction and the corrected words after it', async () => {
		const speller: Speller = {
			check: async (text) => {
				const at = text.indexOf('ottr');
				return at < 0 ? [] : [{ start: at, end: at + 4, word: 'ottr' }];
			},
			suggest: async () => ['otter'],
			ignore: () => undefined,
			addToDictionary: () => undefined,
		};
		const { onCheckpoint, onCommit, box } = setup('A small ottr sat', speller);
		await waitFor(() => expect(document.querySelector('[data-miss]')).not.toBeNull());
		fireEvent.change(box, { target: { value: 'A small ottr sat down' } });
		await waitFor(() => expect(document.querySelector('[data-miss]')).not.toBeNull());
		box.setSelectionRange(9, 9);
		fireEvent.keyDown(box, { key: 'F10', shiftKey: true });
		await screen.findByRole('menu', { name: 'Spelling' });
		await userEvent.click(screen.getByRole('menuitem', { name: 'otter' }));

		expect(onCheckpoint.mock.calls.map(([words]) => words)).toEqual([
			'A small ottr sat down',
			'A small otter sat down',
		]);
		expect(onCommit).not.toHaveBeenCalled();
	});

	it('puts a correction in as typing, so the box keeps its own undo history', async () => {
		const speller: Speller = {
			check: async (text) => {
				const at = text.indexOf('ottr');
				return at < 0 ? [] : [{ start: at, end: at + 4, word: 'ottr' }];
			},
			suggest: async () => ['otter'],
			ignore: () => undefined,
			addToDictionary: () => undefined,
		};
		// jsdom has no editing commands: this one types the text over the selection, as a browser does.
		const typed = vi.fn((_command: string, _ui: boolean, text: string) => {
			const el = document.activeElement as HTMLTextAreaElement;
			el.setRangeText(text, el.selectionStart, el.selectionEnd, 'end');
			return true;
		});
		Object.defineProperty(document, 'execCommand', { value: typed, configurable: true });
		try {
			const { box } = setup('A small ottr sat', speller);
			await waitFor(() => expect(document.querySelector('[data-miss]')).not.toBeNull());
			box.setSelectionRange(9, 9);
			fireEvent.keyDown(box, { key: 'F10', shiftKey: true });
			await screen.findByRole('menu', { name: 'Spelling' });
			await userEvent.click(screen.getByRole('menuitem', { name: 'otter' }));
			expect(typed).toHaveBeenCalledWith('insertText', false, 'otter');
			expect(box.value).toBe('A small otter sat');
		} finally {
			Reflect.deleteProperty(document, 'execCommand');
		}
	});
});
