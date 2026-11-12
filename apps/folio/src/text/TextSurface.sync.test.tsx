// Tests that the on-page editor's words stay the book's whichever comes first, the book's words or the edit's answer
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { EditRequest } from '@liminal-hq/folio-protocol/generated/EditRequest';
import type { TextPos } from '@liminal-hq/folio-protocol/generated/TextPos';
import type { TextTarget } from '@liminal-hq/folio-protocol/generated/TextTarget';
import { act, cleanup, fireEvent, render, screen } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { createStore, useStore } from 'zustand';
import { getTextSelection } from '../canvas/textSelection';
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
afterEach(cleanup);

const story: TextTarget = { type: 'story', flow: 's' };

/**
 * When the book's words reach the window: `first`, as the store does it (the patch is applied
 * before the answer is returned, so the window draws the words before the editor hears back), or
 * `later`, a while after the answer.
 */
type Timing = 'first' | 'later';

/** A position that is really in the words: an edit sent from anywhere else was made on other words. */
function inside(texts: readonly string[], pos: TextPos): boolean {
	return (
		pos.para >= 0 &&
		pos.para < texts.length &&
		pos.offset >= 0 &&
		pos.offset <= [...texts[pos.para]!].length
	);
}

/** The editor over a book held in a store, as the window holds it, with the words laid out a line to a paragraph. */
function mount(start: string[], timing: Timing) {
	// What the window draws, which may lag behind the book itself.
	const drawn = createStore<{ texts: string[] }>(() => ({ texts: start }));
	const book = { texts: start };
	const refused: string[] = [];
	const onEdit = async (edit: EditRequest) => {
		const text = edit.insert.type === 'text' ? edit.insert.text : '';
		const words = book.texts;
		if (!inside(words, edit.from) || !inside(words, edit.to)) {
			refused.push(
				`${JSON.stringify(edit.from)}..${JSON.stringify(edit.to)} in ${JSON.stringify(words)}`,
			);
		}
		const after = replaceInTexts(words, edit.from, edit.to, text);
		book.texts = after;
		await Promise.resolve();
		if (timing === 'first') drawn.setState({ texts: after });
		else setTimeout(() => drawn.setState({ texts: after }), 0);
		await Promise.resolve();
		return { caret: caretAfterText(edit.from, text) };
	};
	function Host() {
		const texts = useStore(drawn, (s) => s.texts);
		return (
			<TextSurface
				target={story}
				frameId="f"
				texts={texts}
				layout={layoutOf(texts.map((w) => [w]))}
				box={{ w: 400, h: 300 }}
				at={{ x: 0, y: 0 }}
				scale={1}
				onEdit={onEdit}
				onClose={() => undefined}
			/>
		);
	}
	render(<Host />);
	return { book, refused, sink: screen.getByRole('textbox') as HTMLTextAreaElement };
}

const settle = () =>
	act(async () => {
		await new Promise((r) => setTimeout(r, 0));
		await new Promise((r) => setTimeout(r, 0));
	});

/** The words and caret a person expects after each key: a plain model of the editor. */
class Model {
	constructor(
		public texts: string[],
		public caret: TextPos = { para: texts.length - 1, offset: texts[texts.length - 1]!.length },
	) {}
	key(key: string) {
		const { para, offset } = this.caret;
		const t = this.texts;
		if (key === 'Backspace') {
			if (offset > 0) {
				t[para] = t[para]!.slice(0, offset - 1) + t[para]!.slice(offset);
				this.caret = { para, offset: offset - 1 };
			} else if (para > 0) {
				const at = t[para - 1]!.length;
				t.splice(para - 1, 2, t[para - 1]! + t[para]!);
				this.caret = { para: para - 1, offset: at };
			}
		} else if (key === 'Delete') {
			if (offset < t[para]!.length)
				t[para] = t[para]!.slice(0, offset) + t[para]!.slice(offset + 1);
			else if (para + 1 < t.length) t.splice(para, 2, t[para]! + t[para + 1]!);
		} else if (key === 'Enter') {
			t.splice(para, 1, t[para]!.slice(0, offset), t[para]!.slice(offset));
			this.caret = { para: para + 1, offset: 0 };
		} else if (key === 'ArrowLeft') {
			if (offset > 0) this.caret = { para, offset: offset - 1 };
			else if (para > 0) this.caret = { para: para - 1, offset: t[para - 1]!.length };
		} else if (key === 'ArrowRight') {
			if (offset < t[para]!.length) this.caret = { para, offset: offset + 1 };
			else if (para + 1 < t.length) this.caret = { para: para + 1, offset: 0 };
		} else {
			t[para] = t[para]!.slice(0, offset) + key + t[para]!.slice(offset);
			this.caret = { para, offset: offset + 1 };
		}
	}
}

/** A small seeded random number generator, so a failing sequence can be run again. */
function random(seed: number) {
	let s = seed >>> 0;
	return () => {
		s = (s + 0x6d2b79f5) >>> 0;
		let r = Math.imul(s ^ (s >>> 15), 1 | s);
		r = (r + Math.imul(r ^ (r >>> 7), 61 | r)) ^ r;
		return ((r ^ (r >>> 14)) >>> 0) / 4294967296;
	};
}

function press(sink: HTMLTextAreaElement, key: string) {
	if (key.length === 1) {
		sink.dispatchEvent(
			new InputEvent('beforeinput', { inputType: 'insertText', data: key, cancelable: true }),
		);
	} else {
		fireEvent.keyDown(sink, { key });
	}
}

const KEYS = [
	'Backspace',
	'Backspace',
	'Backspace',
	'Delete',
	'Delete',
	'Enter',
	'Enter',
	'ArrowLeft',
	'ArrowRight',
	'x',
];

describe('the words the editor works on', () => {
	it.each<Timing>(['first', 'later'])(
		"are the book's after a blank paragraph is joined, the book's words coming %s",
		async (timing) => {
			const { book, refused, sink } = mount(['Well.', 'Emma.', '', 'Grandma.'], timing);
			fireEvent.keyDown(sink, { key: 'End', ctrlKey: true });
			fireEvent.keyDown(sink, { key: 'ArrowUp' });
			fireEvent.keyDown(sink, { key: 'Backspace' });
			await settle();
			expect(book.texts).toEqual(['Well.', 'Emma.', 'Grandma.']);
			// Select-all reaches the end of the last paragraph, not the end of one the editor made up.
			fireEvent.keyDown(sink, { key: 'a', ctrlKey: true });
			expect(getTextSelection()).toMatchObject({ start: 0, end: 'Well.\nEmma.\nGrandma.'.length });
			expect(refused).toEqual([]);
		},
	);

	it.each<Timing>(['first', 'later'])(
		'deleting the paragraph under a blank line takes only what is deleted, the book coming %s',
		async (timing) => {
			const start = ['Well, girls.', '', 'I can’t wait.', '', 'Me either.', '', 'Grandma.'];
			const { book, refused, sink } = mount(start, timing);
			// The caret at the start of `Me either.`, under a blank line.
			const model = new Model([...start], { para: 4, offset: 0 });
			fireEvent.keyDown(sink, { key: 'End', ctrlKey: true });
			fireEvent.keyDown(sink, { key: 'ArrowUp' });
			fireEvent.keyDown(sink, { key: 'ArrowUp' });
			fireEvent.keyDown(sink, { key: 'Home' });
			const keys = ['Backspace', 'Delete', 'Delete', 'Backspace', 'Backspace', 'Delete', 'Enter'];
			for (const key of [...keys, ...keys]) {
				press(sink, key);
				model.key(key);
				await settle();
				expect(book.texts, `after ${key}`).toEqual(model.texts);
			}
			expect(refused).toEqual([]);
		},
	);

	it.each<Timing>(['first', 'later'])(
		'follow random keys the way a person expects, the book coming %s',
		async (timing) => {
			for (let seed = 1; seed <= 40; seed += 1) {
				const next = random(seed);
				const start = ['ab', '', 'cd', '', '', 'ef'].slice(0, 2 + Math.floor(next() * 5));
				const { book, refused, sink } = mount(start, timing);
				const model = new Model([...start]);
				fireEvent.keyDown(sink, { key: 'End', ctrlKey: true });
				const keys: string[] = [];
				for (let i = 0; i < 24; i += 1) {
					// Some keys come in a burst, before the edit before them has been answered.
					const burst = 1 + Math.floor(next() * 3);
					for (let k = 0; k < burst; k += 1) {
						const key = KEYS[Math.floor(next() * KEYS.length)]!;
						// The arrows move the caret at once; edits wait their turn. A burst holds edits only,
						// so the model and the editor agree on where each one is made.
						if (k > 0 && key.startsWith('Arrow')) continue;
						keys.push(key);
						press(sink, key);
						model.key(key);
					}
					await settle();
					expect(book.texts, `seed ${seed}: ${keys.join(' ')}`).toEqual(model.texts);
				}
				const end = model.texts.join('\n').length;
				fireEvent.keyDown(sink, { key: 'a', ctrlKey: true });
				// Nothing to select once every word is gone.
				if (end > 0) expect(getTextSelection(), `seed ${seed}`).toMatchObject({ start: 0, end });
				expect(refused, `seed ${seed}`).toEqual([]);
				cleanup();
			}
		},
		30_000,
	);
});
