// Tests for how a paste uses what the clipboard holds
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { Frame } from '@liminal-hq/folio-protocol/generated/Frame';
import { renderHook } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import type { ClipboardContent } from './frameClipboard';
import { useFrameClipboard, type FrameClipboardInput } from './useFrameClipboard';

const base = { rotation: 0, locked: false };
const text = (extra = {}) =>
	({
		type: 'text',
		id: 't',
		box: { x: 10, y: 20, w: 100, h: 40 },
		...base,
		content: { inline: [{ text: 'Hello' }] },
		...extra,
	}) as unknown as Frame;
const picture = (extra = {}) =>
	({
		type: 'image',
		id: 'p',
		box: { x: 200, y: 40, w: 60, h: 60 },
		...base,
		image: { asset: 'assets/images/fox.png', fit: 'fill' },
		...extra,
	}) as unknown as Frame;

function setup(selected: Frame[]) {
	const input = {
		active: true,
		selected,
		surface: { spread: 'sp' },
		apply: vi.fn(),
		select: vi.fn(),
		pastePicture: vi.fn(),
		pasteText: vi.fn(),
		say: vi.fn(),
		nothingMessage: 'nothing',
	} as unknown as FrameClipboardInput & {
		apply: ReturnType<typeof vi.fn>;
		pastePicture: ReturnType<typeof vi.fn>;
		pasteText: ReturnType<typeof vi.fn>;
	};
	const { result } = renderHook(() => useFrameClipboard(input));
	return { board: result.current, input };
}

const content = (extra: Partial<ClipboardContent> = {}): ClipboardContent => ({
	frames: null,
	picture: null,
	text: '',
	...extra,
});
const shot = { file: new Blob(['x']), extension: 'png' };

describe('pasting in each mode', () => {
	it('pastes frames in place, exactly where they were', () => {
		const { board, input } = setup([]);
		expect(board.place('inPlace', content({ frames: [text()] }))).toBe(true);
		expect(input.apply).toHaveBeenCalledWith(
			expect.objectContaining({
				type: 'AddFrame',
				frame: expect.objectContaining({ box: text().box }),
			}),
		);
		expect(board.place('inPlace', content({ text: 'only words' }))).toBe(false);
	});

	it('pastes the words of frames or of the clipboard into a new text box', () => {
		const { board, input } = setup([]);
		expect(board.place('plain', content({ text: 'Some words' }))).toBe(true);
		expect(input.pasteText).toHaveBeenLastCalledWith('Some words');
		expect(board.place('plain', content({ frames: [text()] }))).toBe(true);
		expect(input.pasteText).toHaveBeenLastCalledWith('Hello');
		expect(board.place('plain', content())).toBe(false);
	});

	it('pastes words into the one free text frame selected, as its words', () => {
		const { board, input } = setup([text({ id: 'target' })]);
		expect(board.place('into', content({ text: 'New words' }))).toBe(true);
		expect(input.pasteText).toHaveBeenCalledWith('New words', 'target');
		expect(board.place('into', content())).toBe(false);
	});

	it('pastes a picture, or a copied picture frame’s picture, into the one picture frame selected', () => {
		const { board, input } = setup([picture({ id: 'slot' })]);
		expect(board.place('into', content({ picture: shot }))).toBe(true);
		expect(input.pastePicture).toHaveBeenCalledWith(shot, 'slot');
		expect(board.place('into', content({ frames: [picture({ id: 'other' })] }))).toBe(true);
		expect(input.apply).toHaveBeenCalledWith({
			type: 'SetImage',
			frame: 'slot',
			asset: 'assets/images/fox.png',
			fit: null,
		});
		// Words have no place in a picture frame.
		expect(board.place('into', content({ text: 'words' }))).toBe(false);
	});

	it('pastes into nothing that is locked, from a master, or one of several', () => {
		for (const selected of [
			[text({ locked: true })],
			[text({ fromMaster: 'm' })],
			[text(), text({ id: 'u' })],
			[],
		]) {
			const { board, input } = setup(selected);
			expect(board.place('into', content({ text: 'words' }))).toBe(false);
			expect(input.pasteText).not.toHaveBeenCalled();
		}
	});

	it('pastes a picture as a new picture frame, and frames as frames', () => {
		const { board, input } = setup([]);
		expect(board.place('paste', content({ picture: shot }))).toBe(true);
		expect(input.pastePicture).toHaveBeenCalledWith(shot);
		expect(board.place('paste', content({ frames: [text()] }))).toBe(true);
		expect(input.apply).toHaveBeenCalledTimes(1);
		expect(board.place('paste', content({ text: 'words from elsewhere' }))).toBe(false);
	});
});
