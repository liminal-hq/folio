// Tests pasting into the on-page editor: Folio's own words, other programs' formatted words and plain words
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { EditRequest } from '@liminal-hq/folio-protocol/generated/EditRequest';
import type { StoryPara } from '@liminal-hq/folio-protocol/generated/StoryPara';
import type { TextPos } from '@liminal-hq/folio-protocol/generated/TextPos';
import { act, fireEvent, render, screen } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { caretAfterText, replaceInTexts } from './edit';
import { layoutOf } from './fixtureLayout';
import { TextSurface } from './TextSurface';

const layout = layoutOf([['hello world']]);

const pasted: StoryPara[] = [
	{
		style: null,
		inline: [
			{ text: 'from ', style: null, bold: false, italic: false },
			{ text: 'Word', style: null, bold: true, italic: false, href: 'https://example.org' },
		],
	},
	{ style: null, inline: [{ text: 'next', style: null, bold: false, italic: true }] },
];

function setup(onPasteHtml?: (html: string) => Promise<StoryPara[]>) {
	const calls: EditRequest[] = [];
	let texts = ['hello world'];
	const onEdit = vi.fn(async (edit: EditRequest) => {
		calls.push(edit);
		const text =
			edit.insert.type === 'text'
				? edit.insert.text
				: edit.insert.paragraphs.map((p) => p.inline.map((r) => r.text).join('')).join('\n');
		texts = replaceInTexts(texts, edit.from, edit.to, text);
		return { caret: caretAfterText(edit.from, text) as TextPos };
	});
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
			{...(onPasteHtml ? { onPasteHtml } : {})}
		/>,
	);
	return { calls, sink: screen.getByRole('textbox') as HTMLTextAreaElement };
}

const clipboard = (data: Record<string, string>) => ({
	clipboardData: { getData: (k: string) => data[k] ?? '' },
});

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

describe('pasting words from another program', () => {
	it('reads the HTML in Rust and pastes the paragraphs with their look, without setting smart punctuation again', async () => {
		const read = vi.fn(async () => pasted);
		const { calls, sink } = setup(read);
		fireEvent.paste(
			sink,
			clipboard({ 'text/html': '<p>from <b>Word</b></p>', 'text/plain': 'from Word' }),
		);
		await act(async () => undefined);
		expect(read).toHaveBeenCalledWith('<p>from <b>Word</b></p>');
		expect(calls).toHaveLength(1);
		expect(calls[0]).toMatchObject({
			insert: { type: 'rich', paragraphs: pasted },
			step: 'own',
		});
		expect(calls[0]!.smart).toBeUndefined();
	});

	it('pastes the plain words when the HTML is refused or has nothing in it, and when there is no reader', async () => {
		const refuse = vi.fn(async () => {
			throw new Error('too large');
		});
		const warn = vi.spyOn(console, 'warn').mockImplementation(() => undefined);
		const first = setup(refuse);
		fireEvent.paste(
			first.sink,
			clipboard({ 'text/html': '<p>big</p>', 'text/plain': 'big plain words' }),
		);
		await act(async () => undefined);
		expect(first.calls[0]).toMatchObject({ insert: { type: 'text', text: 'big plain words' } });
		warn.mockRestore();
		document.body.innerHTML = '';

		const nothing = setup(vi.fn(async () => []));
		fireEvent.paste(
			nothing.sink,
			clipboard({ 'text/html': '<img src="x">', 'text/plain': 'alt words' }),
		);
		await act(async () => undefined);
		expect(nothing.calls[0]).toMatchObject({ insert: { type: 'text', text: 'alt words' } });
		document.body.innerHTML = '';

		const bare = setup();
		fireEvent.paste(bare.sink, clipboard({ 'text/html': '<b>x</b>', 'text/plain': 'x' }));
		await act(async () => undefined);
		expect(bare.calls[0]).toMatchObject({ insert: { type: 'text', text: 'x' } });
	});

	it('does not read the HTML of a clipboard that has none, or that came from Folio', async () => {
		const read = vi.fn(async () => pasted);
		const { calls, sink } = setup(read);
		fireEvent.paste(sink, clipboard({ 'text/plain': 'just words' }));
		await act(async () => undefined);
		expect(calls[0]).toMatchObject({ insert: { type: 'text', text: 'just words' } });
		const own = [
			{ style: null, inline: [{ text: 'mine', style: null, bold: true, italic: false }] },
		];
		fireEvent.paste(
			sink,
			clipboard({
				'application/x-folio-text+json': JSON.stringify(own),
				'text/html': '<p><strong>mine</strong></p>',
				'text/plain': 'mine',
			}),
		);
		await act(async () => undefined);
		expect(read).not.toHaveBeenCalled();
		expect(calls[1]).toMatchObject({ insert: { type: 'rich', paragraphs: own } });
	});

	it('pastes only the plain words after Ctrl+Shift+V, and the next paste is formatted again', async () => {
		const read = vi.fn(async () => pasted);
		const { calls, sink } = setup(read);
		const data = clipboard({ 'text/html': '<p>from <b>Word</b></p>', 'text/plain': 'from Word' });
		fireEvent.keyDown(sink, { key: 'V', ctrlKey: true, shiftKey: true });
		fireEvent.paste(sink, data);
		await act(async () => undefined);
		expect(read).not.toHaveBeenCalled();
		expect(calls[0]).toMatchObject({ insert: { type: 'text', text: 'from Word' } });
		fireEvent.paste(sink, data);
		await act(async () => undefined);
		expect(read).toHaveBeenCalledTimes(1);
		expect(calls[1]).toMatchObject({ insert: { type: 'rich' } });
		// The same holds for the plain words of Folio's own copy.
		const own = [
			{ style: null, inline: [{ text: 'mine', style: null, bold: true, italic: false }] },
		];
		fireEvent.keyDown(sink, { key: 'v', ctrlKey: true, shiftKey: true });
		fireEvent.paste(
			sink,
			clipboard({ 'application/x-folio-text+json': JSON.stringify(own), 'text/plain': 'mine' }),
		);
		await act(async () => undefined);
		expect(calls[2]).toMatchObject({ insert: { type: 'text', text: 'mine' } });
	});
});
