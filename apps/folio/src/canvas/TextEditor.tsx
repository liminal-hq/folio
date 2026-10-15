// Editing a text frame's words in place, over the page where they are drawn
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { FrameLayout } from '@liminal-hq/folio-protocol/generated/FrameLayout';
import type { InlineContent } from '@liminal-hq/folio-protocol/generated/InlineContent';
import type { Frame } from '@liminal-hq/folio-protocol/generated/Frame';
import { useEffect, useRef } from 'react';
import { t } from '../i18n/messages';
import styles from './TextEditor.module.css';

/** A frame with words of its own to edit: a text frame or a shape. */
export type WordFrame = Extract<Frame, { type: 'text' | 'shape' }>;

/** The words of a frame as plain text, or `null` when they carry emphasis that plain text would lose. */
export function plainTextOf(frame: { content?: InlineContent | null }): string | null {
	const runs = frame.content?.inline ?? [];
	if (runs.some((r) => r.bold || r.italic || r.style)) return null;
	return runs.map((r) => r.text).join('');
}

/** Whether a frame's words can be edited in place: its own, and plain. */
export function canEditText(frame: WordFrame): boolean {
	if (frame.locked || frame.fromMaster || plainTextOf(frame) === null) return false;
	// A shape's words are always its own; a line has no inside to put them in.
	return frame.type === 'shape' ? frame.shape !== 'line' : !frame.flow;
}

/** What an editor needs of the words it edits: where they sit, and what they are. */
export interface EditedWords {
	/** A table's cell has its own room inside the table, as a shape's words do. */
	type: 'text' | 'shape' | 'cell';
	/** The box the words' room is measured from and the turn is about: a cell's is its table's. */
	box: { w: number; h: number };
	content?: InlineContent | null;
}

export interface TextEditorProps {
	frame: EditedWords;
	/** How the frame was laid out, for the font and line height the words are drawn in. */
	layout: FrameLayout | undefined;
	/** Screen pixels per point. */
	scale: number;
	/** How far the frame is turned, in degrees clockwise. */
	rotation?: number;
	/** Where the frame sits in the page's own pixels. */
	at: { x: number; y: number };
	/** The new words, or `null` when nothing changed. */
	onCommit(text: string | null): void;
	/** Tab was pressed: the words are kept, then this is called. Without it, Tab moves on as usual. */
	onTab?(back: boolean): void;
}

/**
 * A text area set in the frame's own font over the frame. Escape or leaving it keeps what was typed
 * as one change, so one undo takes it all back.
 */
export function TextEditor({
	frame,
	layout,
	scale,
	rotation = 0,
	at,
	onCommit,
	onTab,
}: TextEditorProps) {
	const area = useRef<HTMLTextAreaElement | null>(null);
	const first = layout?.lines[0];
	const run = first?.runs[0];
	const initial = plainTextOf(frame) ?? '';
	// A shape's words have a room of their own inside it, and the lines are already placed in it.
	const room = frame.type !== 'text' ? layout?.textBox : undefined;
	// The words are set as the layout's style sets them, whatever the shape.
	const align = frame.type !== 'text' ? layout?.textAlign : undefined;
	const centred = align === 'center';
	const done = useRef(false);
	/** What is in the box now, kept as it is typed because the box is gone by the time an unmount cleans up. */
	const typed = useRef(initial);

	useEffect(() => {
		const el = area.current;
		if (!el) return;
		el.focus();
		el.setSelectionRange(el.value.length, el.value.length);
	}, []);

	const finish = () => {
		if (done.current) return;
		done.current = true;
		const text = typed.current;
		onCommit(text !== initial ? text : null);
	};
	// Showing another spread or losing the frame closes the editor without a blur: keep the words.
	const latest = useRef(finish);
	latest.current = finish;
	useEffect(() => () => latest.current(), []);

	return (
		<textarea
			ref={area}
			className={styles.editor}
			aria-label={t('stage.edit.text')}
			defaultValue={initial}
			spellCheck={false}
			onChange={(event) => {
				typed.current = event.target.value;
			}}
			onBlur={finish}
			onKeyDown={(event) => {
				if (event.key === 'Escape') {
					event.preventDefault();
					event.stopPropagation();
					finish();
				} else if (event.key === 'Tab' && onTab) {
					event.preventDefault();
					event.stopPropagation();
					finish();
					onTab(event.shiftKey);
				}
			}}
			style={{
				left: at.x + (room?.x ?? 0) * scale,
				top: at.y + (room?.y ?? 0) * scale,
				width: (room?.w ?? frame.box.w) * scale,
				height: (room?.h ?? frame.box.h) * scale,
				paddingLeft: centred ? 0 : ((first?.x ?? 0) - (room?.x ?? 0)) * scale,
				paddingTop: Math.max(0, ((first?.top ?? 0) - (room?.y ?? 0)) * scale),
				textAlign: align ?? undefined,
				// Turned with the frame, about the frame's middle.
				transform: rotation ? `rotate(${rotation}deg)` : undefined,
				transformOrigin: rotation
					? `${(frame.box.w / 2 - (room?.x ?? 0)) * scale}px ${(frame.box.h / 2 - (room?.y ?? 0)) * scale}px`
					: undefined,
				fontFamily: run ? `'${run.family}', serif` : undefined,
				fontSize: (run?.size ?? 12) * scale,
				fontWeight: run?.weight ?? 400,
				fontStyle: run?.italic ? 'italic' : 'normal',
				lineHeight: first ? `${first.height * scale}px` : 1.3,
			}}
		/>
	);
}
