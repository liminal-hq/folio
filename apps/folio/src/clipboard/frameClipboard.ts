// Copying, cutting and pasting page objects: what goes on the clipboard and how a paste is made
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { Frame } from '@liminal-hq/folio-protocol/generated/Frame';
import type { Intent } from '@liminal-hq/folio-protocol/generated/Intent';
import type { Surface } from '@liminal-hq/folio-protocol/generated/Surface';
import { newFrameId } from '../canvas/newFrames';

/** The format the frames travel in on the system clipboard, where it will hold one. */
export const FRAMES_MIME = 'application/x-folio-frames+json';

/** How far each paste is put from the last, in points; the same as a duplicate. */
export const PASTE_OFFSET = 12;

/** The version of the payload, so a later Folio can read an earlier one's. */
const VERSION = 1;

/** Frames that can be copied: all of them, whether locked or from a master. */
export const copyable = (frames: readonly Frame[]): Frame[] => [...frames];

/** The frames a cut takes off the page: locked ones and a master's stay, since they cannot be deleted. */
export const cuttable = (frames: readonly Frame[]): Frame[] =>
	frames.filter((f) => !f.locked && !f.fromMaster);

/** The frames as text for the clipboard. */
export function serialiseFrames(frames: readonly Frame[]): string {
	return JSON.stringify({ folio: VERSION, frames });
}

/** The frames in a payload, or `null` when the text is not Folio's or is not a list of frames. */
export function parseFrames(json: string): Frame[] | null {
	try {
		const parsed = JSON.parse(json) as { folio?: unknown; frames?: unknown };
		if (parsed.folio !== VERSION || !Array.isArray(parsed.frames)) return null;
		const frames = parsed.frames as Frame[];
		const valid = frames.every(
			(f) =>
				typeof f === 'object' &&
				f !== null &&
				typeof f.type === 'string' &&
				typeof f.id === 'string' &&
				typeof f.box === 'object' &&
				f.box !== null,
		);
		return valid && frames.length > 0 ? frames : null;
	} catch {
		return null;
	}
}

/**
 * The words other programs get when the frames are pasted there, and what Folio compares a paste
 * with to know the clipboard is still holding what it copied: the words of text frames, else the
 * description of what was copied, else a line saying how many frames.
 */
export function plainOf(frames: readonly Frame[]): string {
	const words = frames.flatMap((f) => {
		const text =
			f.type === 'text' || f.type === 'shape'
				? (f.content?.inline ?? []).map((r) => r.text).join('')
				: '';
		return text.trim() === '' ? [] : [text];
	});
	if (words.length > 0) return words.join('\n\n');
	const described = frames.flatMap((f) => {
		const description = 'description' in f ? f.description : undefined;
		return description ? [description] : [];
	});
	if (described.length > 0) return described.join('\n');
	return frames.length === 1 ? 'Folio: 1 frame' : `Folio: ${frames.length} frames`;
}

/** A line-ending-neutral form of text, to compare what was copied with what is pasted. */
export const sameText = (a: string, b: string): boolean =>
	a.replace(/\r\n/g, '\n').trim() === b.replace(/\r\n/g, '\n').trim();

/**
 * A copy of a frame for the page: every id new (a group's children too), moved by `dx` and `dy`,
 * and neither locked nor tied to a master, nor another place in a story's chain. These are the
 * rules of a duplicate.
 */
export function renewFrame(
	frame: Frame,
	dx: number,
	dy: number,
	ids: () => string = newFrameId,
): Frame {
	const copy = renew(frame, ids);
	copy.box = { ...copy.box, x: copy.box.x + dx, y: copy.box.y + dy };
	return copy;
}

function renew(frame: Frame, ids: () => string): Frame {
	const { fromMaster: _master, ...rest } = frame as Frame & { fromMaster?: unknown };
	const copy = { ...rest, id: ids(), locked: false } as Frame & { flow?: unknown };
	if (copy.type === 'text') delete copy.flow;
	if (copy.type === 'group') {
		// The children are renewed too but not moved: they are placed by the group they are in.
		copy.children = copy.children.map((child) => renew(child, ids));
	}
	return copy as Frame;
}

/** The box round frames, for putting a paste where the pointer is. */
export function boundsOf(frames: readonly Frame[]): { x: number; y: number; w: number; h: number } {
	const left = Math.min(...frames.map((f) => f.box.x));
	const top = Math.min(...frames.map((f) => f.box.y));
	const right = Math.max(...frames.map((f) => f.box.x + f.box.w));
	const bottom = Math.max(...frames.map((f) => f.box.y + f.box.h));
	return { x: left, y: top, w: right - left, h: bottom - top };
}

/** How a paste is placed: nudged off the original, left exactly where it was, or centred on a point. */
export type PasteAt =
	{ kind: 'offset'; step: number } | { kind: 'inPlace' } | { kind: 'point'; x: number; y: number };

/** The `AddFrame` intents of a paste, and the ids the new frames have. */
export function pasteIntents(
	frames: readonly Frame[],
	on: Surface,
	at: PasteAt,
	ids: () => string = newFrameId,
): { intents: Intent[]; ids: string[] } {
	let dx = 0;
	let dy = 0;
	if (at.kind === 'offset') {
		dx = dy = PASTE_OFFSET * at.step;
	} else if (at.kind === 'point') {
		const box = boundsOf(frames);
		dx = at.x - (box.x + box.w / 2);
		dy = at.y - (box.y + box.h / 2);
	}
	const made = frames.map((f) => renewFrame(f, dx, dy, ids));
	return {
		intents: made.map((frame): Intent => ({ type: 'AddFrame', on, frame })),
		ids: made.map((f) => f.id),
	};
}

/**
 * The frames a paste event brings, or `null` when it brings none of Folio's. Frames in Folio's own
 * format are taken as they are. Otherwise the words on the clipboard must be the ones Folio put there
 * with the last copy, so a copy made in another program since is not mistaken for it; a copy the
 * system clipboard would not take can only be told by the clipboard being empty.
 */
export function framesFromPaste(
	data: { json: string; plain: string },
	last: Held | null,
): Frame[] | null {
	if (data.json) return parseFrames(data.json);
	if (!last) return null;
	const mine = last.onSystem ? sameText(data.plain, last.plain) : data.plain === '';
	return mine ? parseFrames(last.json) : null;
}

/** What the program holds to paste: the frames and the words it gave other programs with them. */
export interface Held {
	json: string;
	plain: string;
	/** Whether the system clipboard took the copy, so its words can tell whether it still holds it. */
	onSystem: boolean;
	/** How many times it has been pasted, which is how far the next paste is nudged. */
	pastes: number;
}

let held: Held | null = null;

/** Remembers a copy. */
export function hold(frames: readonly Frame[], onSystem: boolean): Held {
	held = { json: serialiseFrames(frames), plain: plainOf(frames), onSystem, pastes: 0 };
	return held;
}

/** What was last copied, or `null`. */
export const holding = (): Held | null => held;

/** Counts a paste of what is held and answers how far to nudge it. */
export function nextStep(): number {
	if (!held) return 1;
	held.pastes += 1;
	return held.pastes;
}

/** Forgets what was copied; for tests. */
export function forget(): void {
	held = null;
}

/**
 * Puts a copy on the system clipboard as the frames' format and as words, as far as the webview
 * lets a page. Answers whether any of it was taken.
 */
export async function writeSystem(json: string, plain: string): Promise<boolean> {
	const clipboard = typeof navigator === 'undefined' ? undefined : navigator.clipboard;
	if (!clipboard) return false;
	const blob = (text: string, type: string) => new Blob([text], { type });
	const attempts: Array<() => Promise<void>> = [];
	if (typeof ClipboardItem !== 'undefined') {
		attempts.push(() =>
			clipboard.write([
				new ClipboardItem({
					'text/plain': blob(plain, 'text/plain'),
					[FRAMES_MIME]: blob(json, FRAMES_MIME),
				}),
			]),
		);
		attempts.push(() =>
			clipboard.write([new ClipboardItem({ 'text/plain': blob(plain, 'text/plain') })]),
		);
	}
	attempts.push(() => clipboard.writeText(plain));
	for (const attempt of attempts) {
		try {
			await attempt();
			return true;
		} catch {
			// The next, plainer way.
		}
	}
	return false;
}

/** The file extension Folio gives a picture of each kind the clipboard can bring. */
const PICTURE_EXTENSIONS: Readonly<Record<string, string>> = {
	'image/png': 'png',
	'image/jpeg': 'jpg',
	'image/gif': 'gif',
	'image/webp': 'webp',
	'image/svg+xml': 'svg',
};

/** The first picture among the files a paste brings, with the extension to store it under. */
export function pictureFromPaste(
	files: ArrayLike<File> | null | undefined,
): { file: File; extension: string } | null {
	for (const file of Array.from(files ?? [])) {
		const extension = PICTURE_EXTENSIONS[file.type.toLowerCase()];
		if (extension) return { file, extension };
	}
	return null;
}
