// Editing a text frame's words in place, over the page where they are drawn
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { FrameLayout } from '@liminal-hq/folio-protocol/generated/FrameLayout';
import type { InlineContent } from '@liminal-hq/folio-protocol/generated/InlineContent';
import type { Frame } from '@liminal-hq/folio-protocol/generated/Frame';
import { ContextMenu } from '@liminal-hq/folio-chrome/ContextMenu';
import type { MenuItem } from '@liminal-hq/folio-chrome/ContextMenu/types';
import { useCallback, useEffect, useRef, useState } from 'react';
import { t } from '../i18n/messages';
import { KEEP_WORDS_EVENT, type KeepWordsDetail } from './editCheckpoint';
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

/** A speller for the words being typed: where the misspellings are, what else a word might be. */
export interface Speller {
	/** The misspelt words of `text`, by UTF-16 range. */
	check(text: string): Promise<{ start: number; end: number; word: string }[]>;
	suggest(word: string): Promise<string[]>;
	/** Leaves the word alone from now on, in this book. */
	ignore(word: string): void;
	/** Adds the word to the dictionary for every book. */
	addToDictionary(word: string): void;
}

/** How long after the last key the words are checked again, in milliseconds. */
const CHECK_DELAY_MS = 200;

/** How long typing must pause before the words so far are kept as an undo step, in milliseconds. */
const KEEP_AFTER_MS = 1000;

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
	/**
	 * Keeps the words so far as an undo step and goes on editing. Sent after a pause in typing, around a
	 * paste, around a correction and before anything else changes the book, so that undo takes back
	 * one stretch of typing at a time.
	 */
	onCheckpoint?(text: string): void;
	/** Tab was pressed: the words are kept, then this is called. Without it, Tab moves on as usual. */
	onTab?(back: boolean): void;
	/** Underlines misspelt words, and offers corrections. Without it, nothing is checked. */
	speller?: Speller;
}

/**
 * A text area set in the frame's own font over the frame. Escape or leaving it keeps what was typed.
 * Along the way a pause, a paste and a correction each keep the words so far as a step of their own,
 * so undo takes back one stretch of typing at a time and not the whole session.
 */
export function TextEditor({
	frame,
	layout,
	scale,
	rotation = 0,
	at,
	onCommit,
	onCheckpoint,
	onTab,
	speller,
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
	/** The words last handed over, as a step or as the end of the editing. */
	const sent = useRef(initial);
	const pause = useRef<ReturnType<typeof setTimeout> | undefined>(undefined);

	useEffect(() => {
		const el = area.current;
		if (!el) return;
		el.focus();
		el.setSelectionRange(el.value.length, el.value.length);
	}, []);

	/** Keeps what is typed so far as an undo step, and goes on. */
	const keep = () => {
		clearTimeout(pause.current);
		if (done.current || typed.current === sent.current) return;
		sent.current = typed.current;
		onCheckpoint?.(typed.current);
	};
	const finish = () => {
		if (done.current) return;
		clearTimeout(pause.current);
		done.current = true;
		const text = typed.current;
		onCommit(text !== sent.current ? text : null);
		sent.current = text;
	};
	// Showing another spread or losing the frame closes the editor without a blur: keep the words.
	const latest = useRef(finish);
	latest.current = finish;
	// Only words that changed need keeping then: with none, there is nothing to send, and React's
	// development double run of effects must not close an editor that has only just opened.
	useEffect(
		() => () => {
			clearTimeout(pause.current);
			if (typed.current !== sent.current) latest.current();
		},
		[],
	);
	// Something is about to change the book: the words typed so far go in first, and an undo or redo
	// closes the editing, since the box would show words the book no longer holds.
	const kept = useRef(keep);
	kept.current = keep;
	useEffect(() => {
		const onKeep = (event: Event) => {
			const { end } = (event as CustomEvent<KeepWordsDetail>).detail;
			if (end) latest.current();
			else kept.current();
		};
		window.addEventListener(KEEP_WORDS_EVENT, onKeep);
		return () => window.removeEventListener(KEEP_WORDS_EVENT, onKeep);
	}, []);

	// Misspellings of what is typed now, checked a moment after the last key. A check that comes back
	// after more was typed is dropped, since its ranges no longer fit.
	const [text, setText] = useState(initial);
	const [misses, setMisses] = useState<{ start: number; end: number; word: string }[]>([]);
	const checking = useRef(0);
	useEffect(() => {
		if (!speller) {
			setMisses([]);
			return;
		}
		const turn = ++checking.current;
		const wait = setTimeout(
			() => {
				speller
					.check(text)
					.then((found) => {
						if (turn === checking.current) setMisses(found);
					})
					.catch(() => {
						if (turn === checking.current) setMisses([]);
					});
			},
			text === initial ? 0 : CHECK_DELAY_MS,
		);
		return () => clearTimeout(wait);
		// eslint-disable-next-line react-hooks/exhaustive-deps
	}, [text, speller]);

	const mirror = useRef<HTMLDivElement | null>(null);
	const [menu, setMenu] = useState<{
		at: { x: number; y: number };
		miss: { start: number; end: number; word: string };
		suggestions: string[];
		keyboard: boolean;
	} | null>(null);
	// While the menu is up the box loses focus to it, and that is not the end of the editing.
	const menuUp = useRef(false);
	menuUp.current = menu !== null;

	const missAt = useCallback(
		(index: number) => misses.find((m) => index >= m.start && index <= m.end),
		[misses],
	);
	const open = (
		miss: { start: number; end: number; word: string },
		at: { x: number; y: number },
		keyboard: boolean,
	) => {
		if (!speller) return;
		void speller
			.suggest(miss.word)
			.catch(() => [] as string[])
			.then((suggestions) => setMenu({ at, miss, suggestions: suggestions.slice(0, 6), keyboard }));
	};
	/** Where a misspelling's first line of letters is on screen, from the mirror's own spans. */
	const rectOf = (miss: { start: number }) =>
		mirror.current?.querySelector(`[data-miss="${miss.start}"]`)?.getBoundingClientRect();
	const replace = (miss: { start: number; end: number }, word: string) => {
		const el = area.current;
		if (!el) return;
		// What was typed up to here is one step, and the correction another.
		keep();
		// The word goes in as typing does, so the box's own undo history survives it: assigning the
		// value would empty that history, and Ctrl+Z would then do nothing for anything typed before.
		el.focus();
		el.setSelectionRange(miss.start, miss.end);
		const typedIn =
			typeof document.execCommand === 'function' && document.execCommand('insertText', false, word);
		const next = typedIn
			? el.value
			: el.value.slice(0, miss.start) + word + el.value.slice(miss.end);
		if (!typedIn) el.value = next;
		typed.current = next;
		setText(next);
		keep();
		const caret = miss.start + word.length;
		el.setSelectionRange(caret, caret);
	};
	const jumpToNext = () => {
		const el = area.current;
		if (!el || misses.length === 0) return;
		const after = misses.find((m) => m.start > el.selectionStart) ?? misses[0]!;
		el.setSelectionRange(after.start, after.end);
	};
	const menuItems = (): MenuItem[] => {
		if (!menu) return [];
		const words: MenuItem[] =
			menu.suggestions.length > 0
				? menu.suggestions.map((word, i) => ({ type: 'action', id: `fix:${i}`, label: word }))
				: [{ type: 'action', id: 'none', label: t('spell.noSuggestions'), disabled: true }];
		return [
			...words,
			{ type: 'separator' },
			{ type: 'action', id: 'ignore', label: t('spell.ignore') },
			{ type: 'action', id: 'add', label: t('spell.add') },
		];
	};

	// The mirror is the same words in the same box, with the misspelt ones underlined. It sits
	// under the box, which is clear, so the underline shows through and never takes a click.
	const parts: { text: string; miss: boolean; start: number }[] = [];
	let at0 = 0;
	for (const m of [...misses].sort((a, b) => a.start - b.start)) {
		if (m.start < at0 || m.end > text.length) continue;
		if (m.start > at0) parts.push({ text: text.slice(at0, m.start), miss: false, start: at0 });
		parts.push({ text: text.slice(m.start, m.end), miss: true, start: m.start });
		at0 = m.end;
	}
	if (at0 < text.length) parts.push({ text: text.slice(at0), miss: false, start: at0 });

	const box = {
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
	} as const;

	return (
		<>
			{speller && (
				<div ref={mirror} className={styles.mirror} style={box} aria-hidden="true">
					{parts.map((part) =>
						part.miss ? (
							<span key={part.start} className={styles.miss} data-miss={part.start}>
								{part.text}
							</span>
						) : (
							<span key={part.start}>{part.text}</span>
						),
					)}
					{'\u200b'}
				</div>
			)}
			<textarea
				ref={area}
				className={speller ? `${styles.editor} ${styles.clear}` : styles.editor}
				aria-label={t('stage.edit.text')}
				defaultValue={initial}
				spellCheck={false}
				onChange={(event) => {
					typed.current = event.target.value;
					setText(event.target.value);
					const how = (event.nativeEvent as InputEvent).inputType;
					// A paste, a drop or a cut is a step of its own, and typing is kept when it pauses.
					if (how === 'insertFromPaste' || how === 'insertFromDrop' || how === 'deleteByCut')
						keep();
					else {
						clearTimeout(pause.current);
						pause.current = setTimeout(() => kept.current(), KEEP_AFTER_MS);
					}
				}}
				onPaste={keep}
				onCut={keep}
				onBlur={() => {
					if (!menuUp.current) finish();
				}}
				onContextMenu={(event) => {
					if (!speller) return;
					const hit = [...(mirror.current?.querySelectorAll('[data-miss]') ?? [])].find((span) =>
						[...span.getClientRects()].some(
							(r) =>
								event.clientX >= r.left &&
								event.clientX <= r.right &&
								event.clientY >= r.top &&
								event.clientY <= r.bottom,
						),
					);
					const miss = hit
						? misses.find((m) => String(m.start) === hit.getAttribute('data-miss'))
						: undefined;
					if (!miss) return;
					event.preventDefault();
					open(miss, { x: event.clientX, y: event.clientY }, false);
				}}
				onKeyDown={(event) => {
					if (
						(event.ctrlKey || event.metaKey) &&
						!event.shiftKey &&
						!event.altKey &&
						event.key.toLowerCase() === 'y' &&
						typeof document.execCommand === 'function'
					) {
						// Redo is Ctrl+Shift+Z in some webviews and Ctrl+Y in others: both work in the box.
						event.preventDefault();
						document.execCommand('redo');
					} else if (event.key === 'Escape') {
						event.preventDefault();
						event.stopPropagation();
						finish();
					} else if (event.key === 'Tab' && onTab) {
						event.preventDefault();
						event.stopPropagation();
						finish();
						onTab(event.shiftKey);
					} else if (speller && event.key === 'F7' && event.altKey) {
						event.preventDefault();
						jumpToNext();
					} else if (
						speller &&
						((event.key === 'F10' && event.shiftKey) || event.key === 'ContextMenu')
					) {
						const miss = missAt(area.current?.selectionStart ?? 0);
						if (!miss) return;
						event.preventDefault();
						const rect = rectOf(miss);
						open(miss, { x: rect?.left ?? 0, y: rect?.bottom ?? 0 }, true);
					}
				}}
				style={box}
			/>
			{speller && (
				<span className={styles.hidden} role="status">
					{misses.length > 0 ? t('spell.count', { count: misses.length }) : ''}
				</span>
			)}
			{menu && (
				<ContextMenu
					items={menuItems()}
					position={menu.at}
					ariaLabel={t('spell.menu')}
					openedWithKeyboard={menu.keyboard}
					returnFocusTo={area.current}
					onSelect={(item) => {
						const { miss } = menu;
						setMenu(null);
						if (item.id.startsWith('fix:')) {
							const word = menu.suggestions[Number(item.id.slice(4))];
							if (word !== undefined) replace(miss, word);
						} else if (item.id === 'ignore' || item.id === 'add') {
							if (item.id === 'ignore') speller?.ignore(miss.word);
							else speller?.addToDictionary(miss.word);
							setMisses((now) => now.filter((m) => m.word !== miss.word));
						}
					}}
					onClose={() => setMenu(null)}
				/>
			)}
		</>
	);
}
