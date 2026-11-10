// Tests for what copy, cut and paste do with page objects
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { Frame } from '@liminal-hq/folio-protocol/generated/Frame';
import { afterEach, describe, expect, it } from 'vitest';
import {
	FRAMES_MIME,
	PASTE_OFFSET,
	copyable,
	cuttable,
	forget,
	framesFromPaste,
	hold,
	holding,
	nextStep,
	parseFrames,
	pasteIntents,
	plainOf,
	renewFrame,
	serialiseFrames,
} from './frameClipboard';

const base = { rotation: 0, locked: false };
const text = (extra = {}) =>
	({
		type: 'text',
		id: 'a',
		box: { x: 10, y: 20, w: 100, h: 40 },
		...base,
		style: 'body',
		content: { inline: [{ text: 'Hello' }] },
		...extra,
	}) as unknown as Frame;
const picture = (extra = {}) =>
	({
		type: 'image',
		id: 'p',
		box: { x: 200, y: 40, w: 60, h: 60 },
		...base,
		image: { asset: 'x.png', fit: 'fill' },
		description: 'A fox',
		...extra,
	}) as unknown as Frame;

let n = 0;
const ids = () => `new_${++n}`;
afterEach(() => {
	forget();
	n = 0;
});

describe('renewFrame', () => {
	it('gives a copy a new id, moves it, and makes it the writer’s own', () => {
		const copy = renewFrame(
			text({ locked: true, fromMaster: 'mst_a', flow: { id: 'fl', index: 0 } }),
			12,
			12,
			ids,
		);
		expect(copy).toMatchObject({
			id: 'new_1',
			locked: false,
			box: { x: 22, y: 32, w: 100, h: 40 },
		});
		expect(copy).not.toHaveProperty('fromMaster');
		expect(copy).not.toHaveProperty('flow');
		// The original is left as it was.
		expect(text({ locked: true }).locked).toBe(true);
	});

	it('renews the children of a group and moves only the group', () => {
		const group = {
			type: 'group',
			id: 'g',
			box: { x: 0, y: 0, w: 50, h: 50 },
			...base,
			children: [text({ id: 'c1' }), picture({ id: 'c2' })],
		} as unknown as Frame;
		const copy = renewFrame(group, 5, 5, ids) as Frame & { children: Frame[] };
		expect(copy.id).toBe('new_1');
		expect(copy.children.map((c) => c.id)).toEqual(['new_2', 'new_3']);
		expect(copy.children[0]!.box).toEqual({ x: 10, y: 20, w: 100, h: 40 });
	});
});

describe('pasteIntents', () => {
	const on = { spread: 'sp' } as never;

	it('adds every frame with new ids, nudged by the step', () => {
		const made = pasteIntents([text(), picture()], on, { kind: 'offset', step: 2 }, ids);
		expect(made.ids).toEqual(['new_1', 'new_2']);
		expect(made.intents).toMatchObject([
			{ type: 'AddFrame', on, frame: { id: 'new_1', box: { x: 10 + 2 * PASTE_OFFSET } } },
			{ type: 'AddFrame', on, frame: { id: 'new_2', box: { x: 200 + 2 * PASTE_OFFSET } } },
		]);
	});

	it('pastes in place, or with the middle of all of them at a point', () => {
		const inPlace = pasteIntents([text()], on, { kind: 'inPlace' }, ids);
		expect(inPlace.intents[0]).toMatchObject({ frame: { box: { x: 10, y: 20 } } });
		// The two together span x 10..260, y 20..100: the middle is (135, 60).
		const atPoint = pasteIntents([text(), picture()], on, { kind: 'point', x: 335, y: 160 }, ids);
		expect(atPoint.intents[0]).toMatchObject({ frame: { box: { x: 210, y: 120 } } });
		expect(atPoint.intents[1]).toMatchObject({ frame: { box: { x: 400, y: 140 } } });
	});
});

describe('what a cut takes', () => {
	it('copies a locked frame and a master’s but takes only the free ones off the page', () => {
		const frames = [text(), text({ id: 'b', locked: true }), text({ id: 'c', fromMaster: 'm' })];
		expect(copyable(frames)).toHaveLength(3);
		expect(cuttable(frames).map((f) => f.id)).toEqual(['a']);
	});
});

describe('the payload', () => {
	it('round-trips and refuses text that is not Folio’s', () => {
		const frames = [text(), picture()];
		expect(parseFrames(serialiseFrames(frames))).toEqual(frames);
		expect(parseFrames('hello')).toBeNull();
		expect(parseFrames('{"folio":2,"frames":[]}')).toBeNull();
		expect(parseFrames(JSON.stringify({ folio: 1, frames: [{ nope: 1 }] }))).toBeNull();
		expect(parseFrames(JSON.stringify({ folio: 1, frames: [] }))).toBeNull();
		expect(FRAMES_MIME).toBe('application/x-folio-frames+json');
	});

	it('says in words what other programs get', () => {
		expect(plainOf([text()])).toBe('Hello');
		expect(plainOf([picture()])).toBe('A fox');
		expect(plainOf([picture({ description: undefined })])).toBe('Folio: 1 frame');
		expect(
			plainOf([picture({ description: undefined }), picture({ id: 'q', description: '' })]),
		).toBe('Folio: 2 frames');
	});
});

describe('framesFromPaste', () => {
	it('takes the frames in Folio’s own format as they are', () => {
		const json = serialiseFrames([text()]);
		expect(framesFromPaste({ json, plain: 'anything' }, null)).toEqual([text()]);
	});

	it('takes the last copy when the clipboard still holds its words, and not after another program copied', () => {
		const last = hold([text()], true);
		expect(framesFromPaste({ json: '', plain: 'Hello\r\n' }, last)).toEqual([text()]);
		expect(framesFromPaste({ json: '', plain: 'Something from a browser' }, last)).toBeNull();
		expect(framesFromPaste({ json: '', plain: '' }, last)).toBeNull();
		expect(framesFromPaste({ json: '', plain: 'Hello' }, null)).toBeNull();
	});

	it('takes the last copy from an empty clipboard when the system would not take it', () => {
		const last = hold([text()], false);
		expect(framesFromPaste({ json: '', plain: '' }, last)).toEqual([text()]);
		expect(framesFromPaste({ json: '', plain: 'Elsewhere' }, last)).toBeNull();
	});
});

describe('the paste step', () => {
	it('counts up with each paste of the same copy and starts again with a new one', () => {
		hold([text()], true);
		expect([nextStep(), nextStep(), nextStep()]).toEqual([1, 2, 3]);
		hold([text()], true);
		expect(nextStep()).toBe(1);
		expect(holding()?.pastes).toBe(1);
	});
});
