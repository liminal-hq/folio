// Copy, cut and paste of page objects for the book on show: the keys, the paste event and the commands
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { Frame } from '@liminal-hq/folio-protocol/generated/Frame';
import type { Intent } from '@liminal-hq/folio-protocol/generated/Intent';
import type { Surface } from '@liminal-hq/folio-protocol/generated/Surface';
import { useEffect, useRef } from 'react';
import { CLIPBOARD_EVENT, type ClipboardKind } from './clipboardRequest';
import {
	FRAMES_MIME,
	copyable,
	cuttable,
	framesFromPaste,
	hold,
	holding,
	nextStep,
	parseFrames,
	pictureFromPaste,
	pasteIntents,
	writeSystem,
	type PasteAt,
} from './frameClipboard';

export interface FrameClipboardInput {
	/** Whether the page objects can be worked on now: a book is on show as it is printed. */
	active: boolean;
	selected: Frame[];
	/** Where the objects of the page on show are kept: the spread or the master being edited. */
	surface: Surface;
	apply(intent: Intent): void;
	/** Selects frames, which is what a paste does with the ones it made. */
	select(ids: string[]): void;
	/** Puts a picture the clipboard brought on the page as a picture frame. */
	pastePicture(picture: { file: File; extension: string }): void;
}

/** Whether typing here belongs to the field, so copy, cut and paste are its own. */
function isEditable(target: EventTarget | null): boolean {
	if (!(target instanceof HTMLElement)) return false;
	return target.isContentEditable || ['INPUT', 'TEXTAREA', 'SELECT'].includes(target.tagName);
}

/** Whether something else has the clipboard's keys: a dialog, or words selected on the page. */
function elsewhere(event: Event): boolean {
	if (isEditable(event.target)) return true;
	if (document.querySelector('dialog[open]')) return true;
	return (window.getSelection()?.toString() ?? '') !== '';
}

export interface FrameClipboard {
	copy(): Promise<void>;
	cut(): Promise<void>;
	/** Pastes what was copied; `at` says where, and the next paste is nudged off the last by default. */
	paste(at?: PasteAt): boolean;
	/** Puts frames on the page as a paste does, and selects them. */
	pasteFrames(frames: Frame[], at: PasteAt): void;
}

export function useFrameClipboard(input: FrameClipboardInput): FrameClipboard {
	// The listeners are made once and read the latest input from here.
	const latest = useRef(input);
	latest.current = input;

	const api = useRef<FrameClipboard | null>(null);
	if (api.current === null) {
		const copy = async () => {
			const { selected } = latest.current;
			if (selected.length === 0) return;
			const kept = hold(copyable(selected), false);
			kept.onSystem = await writeSystem(kept.json, kept.plain);
		};
		const pasteFrames = (frames: Frame[], at: PasteAt) => {
			const { surface, apply, select } = latest.current;
			const made = pasteIntents(frames, surface, at);
			made.intents.forEach(apply);
			select(made.ids);
		};
		api.current = {
			copy,
			pasteFrames,
			async cut() {
				const { selected, surface, apply, select } = latest.current;
				if (selected.length === 0) return;
				await copy();
				// What is locked, or a master's, stays where it is: it is copied but cannot be taken off.
				for (const frame of cuttable(selected)) {
					apply({ type: 'DeleteFrame', on: surface, frame: frame.id });
				}
				select([]);
			},
			paste(at) {
				const last = holding();
				const frames = last ? parseFrames(last.json) : null;
				if (!frames) return false;
				pasteFrames(frames, at ?? { kind: 'offset', step: nextStep() });
				return true;
			},
		};
	}
	const board = api.current;

	useEffect(() => {
		const onKeyDown = (event: KeyboardEvent) => {
			const { active } = latest.current;
			if (!active || event.defaultPrevented || event.altKey || event.shiftKey) return;
			if (!(event.ctrlKey || event.metaKey) || elsewhere(event)) return;
			const key = event.key.toLowerCase();
			if (key !== 'c' && key !== 'x') return;
			// The page objects are the clipboard's: left alone, the webview copies nothing and complains.
			event.preventDefault();
			void (key === 'c' ? board.copy() : board.cut());
		};
		const onPaste = (event: ClipboardEvent) => {
			if (!latest.current.active || event.defaultPrevented || elsewhere(event)) return;
			const data = event.clipboardData;
			const json = data?.getData(FRAMES_MIME) ?? '';
			// Folio's own frames come first; a picture is something another program copied.
			const picture = json ? null : pictureFromPaste(data?.files);
			if (picture) {
				event.preventDefault();
				latest.current.pastePicture(picture);
				return;
			}
			const frames = framesFromPaste({ json, plain: data?.getData('text/plain') ?? '' }, holding());
			if (!frames) return;
			event.preventDefault();
			board.pasteFrames(frames, { kind: 'offset', step: nextStep() });
		};
		const onRequest = (event: Event) => {
			if (!latest.current.active) return;
			const kind = (event as CustomEvent<ClipboardKind>).detail;
			if (kind === 'copy') void board.copy();
			else if (kind === 'cut') void board.cut();
			else board.paste();
		};
		window.addEventListener('keydown', onKeyDown);
		window.addEventListener('paste', onPaste);
		window.addEventListener(CLIPBOARD_EVENT, onRequest);
		return () => {
			window.removeEventListener('keydown', onKeyDown);
			window.removeEventListener('paste', onPaste);
			window.removeEventListener(CLIPBOARD_EVENT, onRequest);
		};
	}, [board]);

	return board;
}
