// Tests smart punctuation in the on-page editor: what is asked for, and Backspace putting back what was typed
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { EditRequest } from '@liminal-hq/folio-protocol/generated/EditRequest';
import type { SmartNote } from '@liminal-hq/folio-protocol/generated/SmartNote';
import type { TextPos } from '@liminal-hq/folio-protocol/generated/TextPos';
import { act, fireEvent, render, screen } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { caretAfterText } from './edit';
import { layoutOf } from './fixtureLayout';
import { TextSurface } from './TextSurface';

const layout = layoutOf([['hello world']]);

/** An `onEdit` that answers like Rust does, and sets a typed hyphen after a hyphen as an em dash. */
function editor() {
	const calls: EditRequest[] = [];
	const onEdit = vi.fn(async (edit: EditRequest) => {
		calls.push(edit);
		const text = edit.insert.type === 'text' ? edit.insert.text : '';
		if (edit.smart && text === '-') {
			// The hyphen before the place is rewritten too, so the edit begins one place earlier.
			const from = { para: 0, offset: edit.from.offset - 1 };
			const caret = caretAfterText(from, '—') as TextPos;
			return { caret, smart: { from, to: caret, text: '—', raw: '--' } as SmartNote };
		}
		return { caret: caretAfterText(edit.from, text) as TextPos };
	});
	return { calls, onEdit };
}

function mount(onEdit: (e: EditRequest) => Promise<{ caret: TextPos; smart?: SmartNote }>) {
	render(
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
		/>,
	);
	return screen.getByRole('textbox') as HTMLTextAreaElement;
}

function input(sink: HTMLElement, inputType: string, data?: string) {
	const event = new InputEvent('beforeinput', { inputType, data, cancelable: true, bubbles: true });
	act(() => {
		sink.dispatchEvent(event);
	});
}

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

describe('smart punctuation in the editor', () => {
	it('asks for it on typed and pasted words, not on Enter or a finished composition', async () => {
		const { calls, onEdit } = editor();
		const sink = mount(onEdit);
		input(sink, 'insertText', '"');
		await act(async () => undefined);
		input(sink, 'insertFromPaste', 'a--b');
		await act(async () => undefined);
		input(sink, 'insertParagraph');
		await act(async () => undefined);
		expect(calls.map((c) => c.smart)).toEqual([true, true, undefined]);
		const event = new Event('compositionend', { bubbles: true });
		Object.defineProperty(event, 'data', { value: 'ö' });
		act(() => {
			sink.dispatchEvent(event);
		});
		await act(async () => undefined);
		expect(calls[3]!.smart).toBeUndefined();
	});

	it('puts back what was typed on Backspace straight after a conversion, as one more edit', async () => {
		const { calls, onEdit } = editor();
		const sink = mount(onEdit);
		input(sink, 'insertText', '-');
		await act(async () => undefined);
		fireEvent.keyDown(sink, { key: 'Backspace' });
		await act(async () => undefined);
		expect(calls).toHaveLength(2);
		expect(calls[1]).toMatchObject({
			from: { para: 0, offset: 10 },
			to: { para: 0, offset: 11 },
			insert: { type: 'text', text: '--' },
			step: 'own',
		});
		expect(calls[1]!.smart).toBeUndefined();
		// What is typed next is left as typed, so the hyphen put back is not made a dash again.
		input(sink, 'insertText', '-');
		await act(async () => undefined);
		expect(calls[2]!.smart).toBeUndefined();
		// A second Backspace is an ordinary one.
		fireEvent.keyDown(sink, { key: 'Backspace' });
		await act(async () => undefined);
		expect(calls[3]).toMatchObject({ insert: { type: 'text', text: '' } });
	});

	it('does not revert when the caret has moved away from the conversion', async () => {
		const { calls, onEdit } = editor();
		const sink = mount(onEdit);
		input(sink, 'insertText', '-');
		await act(async () => undefined);
		fireEvent.keyDown(sink, { key: 'ArrowLeft' });
		fireEvent.keyDown(sink, { key: 'Backspace' });
		await act(async () => undefined);
		expect(calls[1]).toMatchObject({ insert: { type: 'text', text: '' } });
	});
});
