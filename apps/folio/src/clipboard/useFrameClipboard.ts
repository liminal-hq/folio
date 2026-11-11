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
	contentFromPaste,
	copyable,
	cuttable,
	hold,
	holding,
	nextStep,
	parseFrames,
	pasteIntents,
	readSystem,
	wordsOf,
	writeSystem,
	type ClipboardContent,
	type PastedPicture,
	type PasteAt,
} from './frameClipboard';

/** How a paste uses what the clipboard holds. */
export type PasteMode =
	/** Frames as frames, or a picture as a picture frame. */
	| 'paste'
	/** Frames exactly where they were copied from. */
	| 'inPlace'
	/** Only the words, in a new text box. */
	| 'plain'
	/** The words or the picture, into the one frame selected. */
	| 'into';

export interface FrameClipboardInput {
	/** Whether the page objects can be worked on now: a book is on show as it is printed. */
	active: boolean;
	selected: Frame[];
	/** Where the objects of the page on show are kept: the spread or the master being edited. */
	surface: Surface;
	apply(intent: Intent): void;
	/** Selects frames, which is what a paste does with the ones it made. */
	select(ids: string[]): void;
	/** Puts a picture the clipboard brought on the page: in a new picture frame, or in `into`. */
	pastePicture(picture: PastedPicture, into?: string): void;
	/** Puts words on the page: in a new text box, or as the words of `into`. */
	pasteText(text: string, into?: string): void;
	/** Says why a paste did nothing. */
	say(message: string): void;
	/** The message for a paste that found nothing it could place. */
	nothingMessage: string;
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

/** How long a Ctrl+Shift+V waits for the paste event the webview may send before it reads the clipboard itself. */
const PLAIN_WAIT_MS = 150;

export interface FrameClipboard {
	copy(): Promise<void>;
	cut(): Promise<void>;
	/** Pastes what was copied; `at` says where, and the next paste is nudged off the last by default. */
	paste(at?: PasteAt): boolean;
	/** Puts frames on the page as a paste does, and selects them. */
	pasteFrames(frames: Frame[], at: PasteAt): void;
	/** Reads the system clipboard and pastes it in `mode`. */
	pasteWith(mode: PasteMode): Promise<void>;
	/** Uses content already in hand in `mode`; answers whether there was something to place. */
	place(mode: PasteMode, content: ClipboardContent): boolean;
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
			made.intents.forEach((intent) => apply(intent));
			select(made.ids);
		};
		/** The one frame a paste goes into: a free text box or picture frame, alone in the selection. */
		const target = () => {
			const { selected } = latest.current;
			const only = selected.length === 1 ? selected[0]! : null;
			if (!only || only.locked || only.fromMaster) return null;
			return only.type === 'text' || only.type === 'shape' || only.type === 'image' ? only : null;
		};
		/** Uses the content in `mode`; answers whether it found something to place. */
		const place = (mode: PasteMode, content: ClipboardContent): boolean => {
			const { apply, pastePicture, pasteText } = latest.current;
			const words = wordsOf(content);
			if (mode === 'inPlace') {
				if (!content.frames) return false;
				pasteFrames(content.frames, { kind: 'inPlace' });
				return true;
			}
			if (mode === 'plain') {
				if (words === '') return false;
				pasteText(words);
				return true;
			}
			if (mode === 'into') {
				const into = target();
				if (!into) return false;
				if (into.type === 'image') {
					if (content.picture) {
						pastePicture(content.picture, into.id);
						return true;
					}
					const copied = content.frames?.length === 1 ? content.frames[0]! : null;
					if (copied?.type === 'image' && copied.image.asset) {
						apply({ type: 'SetImage', frame: into.id, asset: copied.image.asset, fit: null });
						return true;
					}
					return false;
				}
				if (words === '') return false;
				pasteText(words, into.id);
				return true;
			}
			if (content.frames) {
				pasteFrames(content.frames, { kind: 'offset', step: nextStep() });
				return true;
			}
			if (content.picture) {
				pastePicture(content.picture);
				return true;
			}
			return false;
		};
		api.current = {
			copy,
			pasteFrames,
			place,
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
			async pasteWith(mode) {
				const content = await readSystem();
				if (!place(mode, content)) latest.current.say(latest.current.nothingMessage);
			},
		};
	}
	const board = api.current;

	useEffect(() => {
		/** Ctrl+Shift+V waiting for its paste event, and the timer that reads the clipboard if none comes. */
		let plain: ReturnType<typeof setTimeout> | null = null;
		const onKeyDown = (event: KeyboardEvent) => {
			const { active } = latest.current;
			if (!active || event.defaultPrevented || event.altKey) return;
			if (!(event.ctrlKey || event.metaKey) || elsewhere(event)) return;
			const key = event.key.toLowerCase();
			if (key === 'v' && event.shiftKey) {
				// Paste only the words. The webview may send a paste event, which carries them; if it does
				// not (it only does for what can be edited), the clipboard is read here instead.
				plain = setTimeout(() => {
					plain = null;
					void board.pasteWith('plain');
				}, PLAIN_WAIT_MS);
				return;
			}
			if (event.shiftKey || (key !== 'c' && key !== 'x')) return;
			// The page objects are the clipboard's: left alone, the webview copies nothing and complains.
			event.preventDefault();
			void (key === 'c' ? board.copy() : board.cut());
		};
		const onPaste = (event: ClipboardEvent) => {
			if (!latest.current.active || event.defaultPrevented || elsewhere(event)) return;
			const data = event.clipboardData;
			const content = contentFromPaste(
				{
					json: data?.getData(FRAMES_MIME) ?? '',
					plain: data?.getData('text/plain') ?? '',
					files: data?.files,
				},
				holding(),
			);
			const asked = plain !== null;
			if (plain !== null) clearTimeout(plain);
			plain = null;
			// Words and frames the clipboard holds that are not Folio's are not Folio's to place.
			if (!asked && !content.frames && !content.picture) return;
			if (asked && wordsOf(content) === '' && !content.frames) return;
			event.preventDefault();
			board.place(asked ? 'plain' : 'paste', content);
		};
		const onRequest = (event: Event) => {
			if (!latest.current.active) return;
			const kind = (event as CustomEvent<ClipboardKind>).detail;
			if (kind === 'copy') void board.copy();
			else if (kind === 'cut') void board.cut();
			else void board.pasteWith('paste');
		};
		window.addEventListener('keydown', onKeyDown);
		window.addEventListener('paste', onPaste);
		window.addEventListener(CLIPBOARD_EVENT, onRequest);
		return () => {
			if (plain !== null) clearTimeout(plain);
			window.removeEventListener('keydown', onKeyDown);
			window.removeEventListener('paste', onPaste);
			window.removeEventListener(CLIPBOARD_EVENT, onRequest);
		};
	}, [board]);

	return board;
}
