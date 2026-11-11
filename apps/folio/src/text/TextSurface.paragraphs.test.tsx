// Tests select-all and paragraph joins in the on-page editor, and the paragraph looks the ribbon is told
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { EditRequest } from '@liminal-hq/folio-protocol/generated/EditRequest';
import type { ParaLook } from '@liminal-hq/folio-protocol/generated/ParaLook';
import type { TextTarget } from '@liminal-hq/folio-protocol/generated/TextTarget';
import { act, fireEvent, render, screen } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { getTextParagraphs, getTextSelection } from '../canvas/textSelection';
import { caretAfterText, replaceInTexts } from './edit';
import { layoutOf } from './fixtureLayout';
import { TextSurface } from './TextSurface';

beforeEach(() => {
	vi.spyOn(HTMLElement.prototype, 'getBoundingClientRect').mockReturnValue({
		left: 0,
		top: 0,
		width: 400,
		height: 300,
		right: 400,
		bottom: 300,
		x: 0,
		y: 0,
		toJSON: () => ({}),
	});
});

const story: TextTarget = { type: 'story', flow: 's' };
const frame: TextTarget = { type: 'frame', frame: 'f' };

/** The editor over `texts`, laid out a line to a paragraph, with `looks` for the paragraphs. */
function mount(target: TextTarget, texts: string[], looks: ParaLook[] = []) {
	const calls: EditRequest[] = [];
	const onEdit = vi.fn(async (edit: EditRequest) => {
		calls.push(edit);
		const text = edit.insert.type === 'text' ? edit.insert.text : '';
		return { caret: caretAfterText(edit.from, text) };
	});
	const props = (words: string[], own: ParaLook[]) => ({
		target,
		frameId: 'f',
		texts: words,
		content: {
			inline: words.flatMap((w, i) => [
				{ text: i === 0 ? w : `\n${w}`, style: null, bold: false, italic: false },
			]),
			looks: own,
		},
		layout: layoutOf(words.map((w) => [w])),
		box: { w: 400, h: 300 },
		at: { x: 0, y: 0 },
		scale: 1,
		onEdit,
		onClose: () => undefined,
	});
	const view = render(<TextSurface {...props(texts, looks)} />);
	const sink = screen.getByRole('textbox') as HTMLTextAreaElement;
	return {
		calls,
		sink,
		update: (words: string[], own: ParaLook[] = []) =>
			view.rerender(<TextSurface {...props(words, own)} />),
	};
}

const all = (texts: string[]) => texts.join('\n').length;

describe('Ctrl+A', () => {
	const cases: [string, TextTarget, string[], ParaLook[]][] = [
		['a frame of plain words', frame, ['One', 'Two', 'Three'], []],
		['a frame with blank paragraphs', frame, ['One', '', 'Three', ''], []],
		[
			'a story whose paragraphs have looks',
			story,
			['One', 'Two', ''],
			[{ spaceBefore: 8 }, {}, { spaceAfter: 4 }],
		],
		['a story whose first paragraph is blank', story, ['', 'Two'], [{ spaceBefore: 8 }]],
	];
	it.each(cases)('selects every paragraph of %s', (_, target, texts, looks) => {
		const { sink } = mount(target, texts, looks);
		fireEvent.keyDown(sink, { key: 'a', ctrlKey: true });
		expect(getTextSelection()).toMatchObject({ start: 0, end: all(texts) });
		expect(getTextParagraphs()).toMatchObject({ from: 0, to: texts.length - 1 });
		// Every paragraph with words is drawn selected (a blank one inside the selection is too).
		expect(document.querySelectorAll('[data-selection]').length).toBeGreaterThanOrEqual(
			texts.filter((t) => t !== '').length,
		);
	});

	it('selects the paragraphs the book has now, not those the editor opened on', () => {
		const { sink, update } = mount(story, ['One']);
		update(['One', 'Two', 'Three']);
		fireEvent.keyDown(sink, { key: 'a', ctrlKey: true });
		expect(getTextSelection()).toMatchObject({ start: 0, end: all(['One', 'Two', 'Three']) });
	});
});

describe('Backspace in a blank paragraph', () => {
	it('joins it to the one before, and the paragraph looks follow the paragraphs', async () => {
		const looks: ParaLook[] = [{ spaceBefore: 8 }, { spaceBefore: 8 }, { spaceBefore: 12 }];
		const texts = ['Emma.', '', 'Grandma.'];
		const { calls, sink, update } = mount(story, texts, looks);
		// The caret into the blank paragraph: the end, then up to the blank line.
		fireEvent.keyDown(sink, { key: 'End', ctrlKey: true });
		fireEvent.keyDown(sink, { key: 'ArrowUp' });
		expect(getTextParagraphs()).toMatchObject({ from: 1, to: 1 });
		fireEvent.keyDown(sink, { key: 'Backspace' });
		await act(async () => undefined);
		expect(calls[0]).toMatchObject({
			from: { para: 0, offset: 5 },
			to: { para: 1, offset: 0 },
			insert: { type: 'text', text: '' },
		});
		// The book answers: the blank paragraph is gone, and each paragraph keeps its own look.
		const joined = replaceInTexts(texts, calls[0]!.from, calls[0]!.to, '');
		expect(joined).toEqual(['Emma.', 'Grandma.']);
		update(joined, [looks[0]!, looks[2]!]);
		expect(getTextParagraphs()).toMatchObject({ from: 0, to: 0, looks: [looks[0], looks[2]] });
		fireEvent.keyDown(sink, { key: 'ArrowDown' });
		expect(getTextParagraphs()).toMatchObject({ from: 1, to: 1 });
	});
});
