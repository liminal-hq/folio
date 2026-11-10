// Tests the on-page editor: keys, input methods and the mouse become edits, and the caret follows the answers
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { EditRequest } from '@liminal-hq/folio-protocol/generated/EditRequest';
import type { TextPos } from '@liminal-hq/folio-protocol/generated/TextPos';
import { act, fireEvent, render, screen } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { clearPendingLook, getTextCaret, setPendingLook } from '../canvas/textSelection';
import { replaceInTexts, caretAfterText } from './edit';
import { CHAR_W, layoutOf, LINE_H } from './fixtureLayout';
import { TextSurface } from './TextSurface';

const layout = layoutOf([['hello world']]);

/** An `onEdit` that answers like Rust does, after `delay` has been resolved by the test. */
function editor(initial = ['hello world']) {
	const calls: EditRequest[] = [];
	let texts = initial;
	const onEdit = vi.fn(async (edit: EditRequest) => {
		calls.push(edit);
		const text = edit.insert.type === 'text' ? edit.insert.text : '';
		texts = replaceInTexts(texts, edit.from, edit.to, text);
		return { caret: caretAfterText(edit.from, text) as TextPos };
	});
	return { calls, onEdit, words: () => texts };
}

function mount(onEdit: (e: EditRequest) => Promise<{ caret: TextPos }>, extra = {}) {
	const result = render(
		<TextSurface
			target={{ type: 'frame', frame: 'f' }}
			frameId="f"
			texts={['hello world']}
			layout={layout}
			box={{ w: 200, h: 100 }}
			at={{ x: 0, y: 0 }}
			scale={1}
			onEdit={onEdit}
			onClose={() => undefined}
			{...extra}
		/>,
	);
	const sink = screen.getByRole('textbox') as HTMLTextAreaElement;
	return { ...result, sink };
}

/** A composition event carrying `data`, since jsdom's has none to set. */
function compose(sink: HTMLElement, type: string, data: string) {
	const event = new Event(type, { bubbles: true });
	Object.defineProperty(event, 'data', { value: data });
	act(() => {
		sink.dispatchEvent(event);
	});
}

function input(sink: HTMLElement, inputType: string, data?: string) {
	const event = new InputEvent('beforeinput', { inputType, data, cancelable: true, bubbles: true });
	act(() => {
		sink.dispatchEvent(event);
	});
	return event;
}

const caretLeft = () =>
	Number.parseFloat((document.querySelector('[data-caret]') as HTMLElement).style.left);

beforeEach(() => {
	vi.spyOn(HTMLElement.prototype, 'getBoundingClientRect').mockReturnValue({
		left: 0,
		top: 0,
		width: 200,
		height: 100,
		right: 200,
		bottom: 100,
		x: 0,
		y: 0,
		toJSON: () => ({}),
	});
});

describe('typing', () => {
	it('sends what is typed at the caret, which starts at the end, and the caret follows the answer', async () => {
		const { calls, onEdit } = editor();
		const { sink } = mount(onEdit);
		expect(caretLeft()).toBe(11 * CHAR_W);
		const event = input(sink, 'insertText', '!');
		expect(event.defaultPrevented).toBe(true);
		await act(async () => undefined);
		expect(calls[0]).toMatchObject({
			target: { type: 'frame', frame: 'f' },
			from: { para: 0, offset: 11 },
			to: { para: 0, offset: 11 },
			insert: { type: 'text', text: '!' },
			step: 'typing',
		});
		// The layout the page holds has not changed here, so the caret is clamped to its words.
		expect(caretLeft()).toBeGreaterThan(0);
	});

	it('queues keys pressed while an edit is in flight and takes each place from the answer before', async () => {
		const gate: { release: () => void } = { release: () => undefined };
		const calls: EditRequest[] = [];
		const onEdit = vi.fn(async (edit: EditRequest) => {
			calls.push(edit);
			if (calls.length === 1) await new Promise<void>((resolve) => (gate.release = resolve));
			const text = edit.insert.type === 'text' ? edit.insert.text : '';
			return { caret: caretAfterText(edit.from, text) };
		});
		const { sink } = mount(onEdit);
		input(sink, 'insertText', 'a');
		input(sink, 'insertText', 'b');
		await act(async () => undefined);
		expect(calls).toHaveLength(1);
		await act(async () => {
			gate.release();
		});
		expect(calls).toHaveLength(2);
		expect(calls[1]!.from).toEqual({ para: 0, offset: 12 });
	});

	it('turns Enter into a line break, as a step of its own', async () => {
		const { calls, onEdit } = editor();
		const { sink } = mount(onEdit);
		fireEvent.keyDown(sink, { key: 'Enter' });
		await act(async () => undefined);
		expect(calls[0]).toMatchObject({ insert: { type: 'text', text: '\n' }, step: 'own' });
	});

	it('deletes the grapheme before the caret on Backspace and the selection if there is one', async () => {
		const { calls, onEdit } = editor();
		const { sink } = mount(onEdit);
		fireEvent.keyDown(sink, { key: 'Backspace' });
		await act(async () => undefined);
		expect(calls[0]).toMatchObject({
			from: { para: 0, offset: 10 },
			to: { para: 0, offset: 11 },
			insert: { type: 'text', text: '' },
			step: 'typing',
		});
		fireEvent.keyDown(sink, { key: 'a', ctrlKey: true });
		fireEvent.keyDown(sink, { key: 'Delete' });
		await act(async () => undefined);
		expect(calls[1]).toMatchObject({
			from: { para: 0, offset: 0 },
			to: { para: 0, offset: 10 },
			step: 'own',
		});
	});

	it('does nothing for Backspace at the very start', async () => {
		const { calls, onEdit } = editor();
		const { sink } = mount(onEdit);
		fireEvent.keyDown(sink, { key: 'Home' });
		fireEvent.keyDown(sink, { key: 'Backspace' });
		await act(async () => undefined);
		expect(calls).toHaveLength(0);
	});
});

describe('moving and selecting', () => {
	it('moves the caret with the arrows and Home and End without an edit', () => {
		const { onEdit } = editor();
		const { sink } = mount(onEdit);
		fireEvent.keyDown(sink, { key: 'ArrowLeft' });
		expect(caretLeft()).toBe(10 * CHAR_W);
		fireEvent.keyDown(sink, { key: 'Home' });
		expect(caretLeft()).toBe(0);
		fireEvent.keyDown(sink, { key: 'ArrowRight', ctrlKey: true });
		expect(caretLeft()).toBe(5 * CHAR_W);
		fireEvent.keyDown(sink, { key: 'End' });
		expect(caretLeft()).toBe(11 * CHAR_W);
		expect(onEdit).not.toHaveBeenCalled();
	});

	it('selects with Shift and draws it, and collapsing to an edge with a plain arrow', () => {
		const { onEdit } = editor();
		const { sink } = mount(onEdit);
		fireEvent.keyDown(sink, { key: 'ArrowLeft', shiftKey: true, ctrlKey: true });
		const rects = document.querySelectorAll('[data-selection]');
		expect(rects).toHaveLength(1);
		expect((rects[0] as HTMLElement).style.left).toBe(`${6 * CHAR_W}px`);
		expect((rects[0] as HTMLElement).style.width).toBe(`${5 * CHAR_W}px`);
		fireEvent.keyDown(sink, { key: 'ArrowLeft' });
		expect(document.querySelectorAll('[data-selection]')).toHaveLength(0);
		expect(caretLeft()).toBe(6 * CHAR_W);
	});

	it('puts the caret under a click and selects a word on a double click', () => {
		const { onEdit } = editor();
		const { container } = mount(onEdit);
		const surface = container.querySelector('[data-text-surface]') as HTMLElement;
		fireEvent.mouseDown(surface, { clientX: 3 * CHAR_W + 1, clientY: LINE_H / 2, detail: 1 });
		expect(caretLeft()).toBe(3 * CHAR_W);
		fireEvent.mouseDown(surface, { clientX: 8 * CHAR_W, clientY: LINE_H / 2, detail: 2 });
		const rect = document.querySelector('[data-selection]') as HTMLElement;
		expect(rect.style.left).toBe(`${6 * CHAR_W}px`);
	});

	it('closes on Escape', () => {
		const { onEdit } = editor();
		const onClose = vi.fn();
		const { sink } = mount(onEdit, { onClose });
		fireEvent.keyDown(sink, { key: 'Escape' });
		expect(onClose).toHaveBeenCalled();
	});
});

describe('input methods', () => {
	it('lets the browser compose and sends the finished words as one edit', async () => {
		const { calls, onEdit } = editor();
		const { sink } = mount(onEdit);
		fireEvent.compositionStart(sink);
		const during = input(sink, 'insertCompositionText', 'に');
		expect(during.defaultPrevented).toBe(false);
		compose(sink, 'compositionupdate', 'にほ');
		expect(sink.className).toContain('composing');
		compose(sink, 'compositionend', '日本');
		await act(async () => undefined);
		expect(calls).toHaveLength(1);
		expect(calls[0]).toMatchObject({ insert: { type: 'text', text: '日本' } });
		expect(sink.className).not.toContain('composing');
		expect(sink.value).toBe('');
	});
});

describe('clipboard', () => {
	it('copies the selection as plain text and pastes plain text as a step of its own', async () => {
		const { calls, onEdit } = editor();
		const { sink } = mount(onEdit);
		fireEvent.keyDown(sink, { key: 'ArrowLeft', shiftKey: true, ctrlKey: true });
		const store: Record<string, string> = {};
		fireEvent.copy(sink, {
			clipboardData: { setData: (k: string, v: string) => (store[k] = v) },
		});
		expect(store['text/plain']).toBe('world');
		fireEvent.paste(sink, { clipboardData: { getData: () => 'a\r\nb' } });
		await act(async () => undefined);
		expect(calls[0]).toMatchObject({ insert: { type: 'text', text: 'a\nb' }, step: 'own' });
	});
});

describe('undo and redo', () => {
	it('undoes through the book, keeps the editor open and puts the caret where the step had it', async () => {
		const { onEdit } = editor();
		const target = { type: 'frame', frame: 'f' } as const;
		const onUndo = vi.fn(async () => ({
			target,
			anchor: { para: 0, offset: 2 },
			focus: { para: 0, offset: 5 },
		}));
		const onClose = vi.fn();
		const { sink } = mount(onEdit, { onUndo, onClose });
		fireEvent.keyDown(sink, { key: 'z', ctrlKey: true });
		await act(async () => undefined);
		expect(onUndo).toHaveBeenCalledTimes(1);
		expect(onClose).not.toHaveBeenCalled();
		const rect = document.querySelector('[data-selection]') as HTMLElement;
		expect(rect.style.left).toBe(`${2 * CHAR_W}px`);
		expect(rect.style.width).toBe(`${3 * CHAR_W}px`);
	});

	it('redoes with Ctrl+Shift+Z and Ctrl+Y, and ignores a caret meant for other words', async () => {
		const { onEdit } = editor();
		const onRedo = vi.fn(async () => ({
			target: { type: 'frame', frame: 'other' } as const,
			anchor: { para: 0, offset: 1 },
			focus: { para: 0, offset: 1 },
		}));
		const { sink } = mount(onEdit, { onRedo });
		fireEvent.keyDown(sink, { key: 'Z', ctrlKey: true, shiftKey: true });
		fireEvent.keyDown(sink, { key: 'y', ctrlKey: true });
		await act(async () => undefined);
		expect(onRedo).toHaveBeenCalledTimes(2);
		expect(caretLeft()).toBe(11 * CHAR_W);
	});
});

describe('a look set at the caret', () => {
	it('publishes the caret, and goes with the next words typed as a step of their own', async () => {
		const { calls, onEdit } = editor();
		const { sink } = mount(onEdit);
		expect(getTextCaret()).toEqual({ frame: 'f', offset: 11 });
		setPendingLook('f', { bold: true });
		input(sink, 'insertText', 'x');
		await act(async () => undefined);
		expect(calls[0]).toMatchObject({ look: { bold: true }, step: 'own' });
		// It was used up, so the next key carries none.
		input(sink, 'insertText', 'y');
		await act(async () => undefined);
		expect(calls[1]!.look).toBeUndefined();
		clearPendingLook();
	});

	it('is forgotten when the caret moves', async () => {
		const { calls, onEdit } = editor();
		const { sink } = mount(onEdit);
		setPendingLook('f', { italic: true });
		fireEvent.keyDown(sink, { key: 'ArrowLeft' });
		input(sink, 'insertText', 'x');
		await act(async () => undefined);
		expect(calls[0]!.look).toBeUndefined();
	});
});

describe('copying with the look', () => {
	const content = {
		inline: [
			{ text: 'hello ', style: null, bold: false, italic: false },
			{ text: 'world', style: null, bold: true, italic: false },
		],
	};

	it('puts Folio, HTML and plain text on the clipboard', () => {
		const { onEdit } = editor();
		const { sink } = mount(onEdit, { content });
		fireEvent.keyDown(sink, { key: 'ArrowLeft', shiftKey: true, ctrlKey: true });
		const store: Record<string, string> = {};
		fireEvent.copy(sink, {
			clipboardData: { setData: (k: string, v: string) => (store[k] = v) },
		});
		expect(store['text/plain']).toBe('world');
		expect(store['text/html']).toBe('<p><strong>world</strong></p>');
		expect(JSON.parse(store['application/x-folio-text+json']!)[0].inline[0]).toMatchObject({
			text: 'world',
			bold: true,
		});
	});

	it('pastes what Folio copied with its look, and anything else as plain words', async () => {
		const { calls, onEdit } = editor();
		const { sink } = mount(onEdit, { content });
		const rich = [
			{ style: null, inline: [{ text: 'bold', style: null, bold: true, italic: false }] },
		];
		fireEvent.paste(sink, {
			clipboardData: {
				getData: (k: string) =>
					k === 'application/x-folio-text+json' ? JSON.stringify(rich) : 'ignored',
			},
		});
		await act(async () => undefined);
		expect(calls[0]).toMatchObject({ insert: { type: 'rich', paragraphs: rich }, step: 'own' });
		fireEvent.paste(sink, {
			clipboardData: { getData: (k: string) => (k === 'text/plain' ? 'plain' : '') },
		});
		await act(async () => undefined);
		expect(calls[1]).toMatchObject({ insert: { type: 'text', text: 'plain' } });
	});
});

describe('spelling', () => {
	// "hello wrold": the second word is marked.
	const marked = (() => {
		const base = layoutOf([['hello wrold']]);
		const run = base.lines[0]!.runs[1]!;
		run.miss = [[0, 5]];
		return base;
	})();
	const speller = () => ({
		check: async () => [],
		suggest: vi.fn(async () => ['world', 'wild']),
		ignore: vi.fn(),
		addToDictionary: vi.fn(),
	});

	it('selects the next misspelt word on Alt+F7', () => {
		const { onEdit } = editor(['hello wrold']);
		const { sink } = mount(onEdit, { layout: marked, texts: ['hello wrold'], speller: speller() });
		fireEvent.keyDown(sink, { key: 'Home' });
		fireEvent.keyDown(sink, { key: 'F7', altKey: true });
		const rect = document.querySelector('[data-selection]') as HTMLElement;
		expect(rect.style.left).toBe(`${6 * CHAR_W}px`);
		expect(rect.style.width).toBe(`${5 * CHAR_W}px`);
	});

	it('offers corrections for the word under a right click, and a pick replaces it as a step of its own', async () => {
		const { calls, onEdit } = editor(['hello wrold']);
		const s = speller();
		const { container } = mount(onEdit, { layout: marked, texts: ['hello wrold'], speller: s });
		const surface = container.querySelector('[data-text-surface]') as HTMLElement;
		await act(async () => {
			fireEvent.contextMenu(surface, { clientX: 8 * CHAR_W, clientY: LINE_H / 2 });
		});
		expect(s.suggest).toHaveBeenCalledWith('wrold');
		await act(async () => {
			fireEvent.click(await screen.findByText('world'));
		});
		await act(async () => undefined);
		expect(calls[0]).toMatchObject({
			from: { para: 0, offset: 6 },
			to: { para: 0, offset: 11 },
			insert: { type: 'text', text: 'world' },
			step: 'own',
		});
	});

	it('leaves a word that is spelt right to the usual menu', async () => {
		const { onEdit } = editor(['hello wrold']);
		const s = speller();
		const { container } = mount(onEdit, { layout: marked, texts: ['hello wrold'], speller: s });
		const surface = container.querySelector('[data-text-surface]') as HTMLElement;
		const event = fireEvent.contextMenu(surface, { clientX: 2 * CHAR_W, clientY: LINE_H / 2 });
		expect(event).toBe(true);
		expect(s.suggest).not.toHaveBeenCalled();
	});
});
