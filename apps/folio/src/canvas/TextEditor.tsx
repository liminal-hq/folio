// Editing a text frame's words in place, over the page where they are drawn
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { FrameLayout } from '@liminal-hq/folio-protocol/generated/FrameLayout';
import type { TextFrame } from '@liminal-hq/folio-protocol/generated/TextFrame';
import { useEffect, useRef } from 'react';
import { t } from '../i18n/messages';
import styles from './TextEditor.module.css';

/** The words of a frame as plain text, or `null` when they carry emphasis that plain text would lose. */
export function plainTextOf(frame: TextFrame): string | null {
	const runs = frame.content?.inline ?? [];
	if (runs.some((r) => r.bold || r.italic || r.style)) return null;
	return runs.map((r) => r.text).join('');
}

/** Whether a frame's words can be edited in place: its own, and plain. */
export function canEditText(frame: TextFrame): boolean {
	return !frame.locked && !frame.fromMaster && !frame.flow && plainTextOf(frame) !== null;
}

export interface TextEditorProps {
	frame: TextFrame;
	/** How the frame was laid out, for the font and line height the words are drawn in. */
	layout: FrameLayout | undefined;
	/** Screen pixels per point. */
	scale: number;
	/** Where the frame sits in the page's own pixels. */
	at: { x: number; y: number };
	/** The new words, or `null` when nothing changed. */
	onCommit(text: string | null): void;
}

/**
 * A text area set in the frame's own font over the frame. Escape or leaving it keeps what was typed
 * as one change, so one undo takes it all back.
 */
export function TextEditor({ frame, layout, scale, at, onCommit }: TextEditorProps) {
	const area = useRef<HTMLTextAreaElement | null>(null);
	const first = layout?.lines[0];
	const run = first?.runs[0];
	const initial = plainTextOf(frame) ?? '';
	const done = useRef(false);

	useEffect(() => {
		const el = area.current;
		if (!el) return;
		el.focus();
		el.setSelectionRange(el.value.length, el.value.length);
	}, []);

	const finish = () => {
		if (done.current) return;
		done.current = true;
		const text = area.current?.value ?? initial;
		onCommit(text !== initial ? text : null);
	};

	return (
		<textarea
			ref={area}
			className={styles.editor}
			aria-label={t('stage.edit.text')}
			defaultValue={initial}
			spellCheck={false}
			onBlur={finish}
			onKeyDown={(event) => {
				if (event.key === 'Escape') {
					event.preventDefault();
					event.stopPropagation();
					finish();
				}
			}}
			style={{
				left: at.x,
				top: at.y,
				width: frame.box.w * scale,
				height: frame.box.h * scale,
				paddingLeft: (first?.x ?? 0) * scale,
				paddingTop: Math.max(0, (first?.top ?? 0) * scale),
				fontFamily: run ? `'${run.family}', serif` : undefined,
				fontSize: (run?.size ?? 12) * scale,
				fontWeight: run?.weight ?? 400,
				fontStyle: run?.italic ? 'italic' : 'normal',
				lineHeight: first ? `${first.height * scale}px` : 1.3,
			}}
		/>
	);
}
