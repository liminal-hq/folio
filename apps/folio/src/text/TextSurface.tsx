// Editing words on the page: a caret and a selection over the page's own lines, and an invisible input for the keys
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { InlineContent } from '@liminal-hq/folio-protocol/generated/InlineContent';
import type { StoryPara } from '@liminal-hq/folio-protocol/generated/StoryPara';
import type { EditCaret } from '@liminal-hq/folio-protocol/generated/EditCaret';
import type { EditRequest } from '@liminal-hq/folio-protocol/generated/EditRequest';
import type { FrameLayout } from '@liminal-hq/folio-protocol/generated/FrameLayout';
import type { RunChange } from '@liminal-hq/folio-protocol/generated/RunChange';
import type { TextPos } from '@liminal-hq/folio-protocol/generated/TextPos';
import type { TextTarget } from '@liminal-hq/folio-protocol/generated/TextTarget';
import { useCallback, useEffect, useLayoutEffect, useRef, useState } from 'react';
import {
	clearPendingLook,
	getPendingLook,
	setTextCaret,
	setTextSelection,
} from '../canvas/textSelection';
import { t } from '../i18n/messages';
import { FOLIO_TEXT_MIME, fromClipboard, sliceParagraphs, toHtml, toPlain } from './clipboard';
import { ContextMenu } from '@liminal-hq/folio-chrome/ContextMenu';
import type { MenuItem } from '@liminal-hq/folio-chrome/ContextMenu/types';
import type { Speller } from '../canvas/TextEditor';
import { caretRect, hitTest, misspellings, selectionRects, type Misspelt } from './caretGeometry';
import { clampPos, endOf, flatOffset, ordered, replaceInTexts, samePos, textBetween } from './edit';
import { moveGrapheme, moveLineEdge, moveVertical, moveWord, wordAt } from './navigate';
import { cpLength, cpSlice, snapToGrapheme } from './units';
import styles from './TextSurface.module.css';

/** The two ends of a selection: where it began and where the caret is. */
export interface Selection {
	anchor: TextPos;
	focus: TextPos;
}

export interface TextSurfaceProps {
	target: TextTarget;
	/** The frame's id, so the ribbon can style the selected words. */
	frameId: string;
	/** The words as paragraphs, as the book holds them. */
	texts: readonly string[];
	/** The words with their looks, for copying them with the look they have. */
	content?: InlineContent | null;
	/** The frame laid out as the page shows it. */
	layout: FrameLayout | undefined;
	/** The frame's box, in points. */
	box: { w: number; h: number };
	/** Where the frame sits over the page, in screen pixels. */
	at: { x: number; y: number };
	/** Screen pixels per point. */
	scale: number;
	/** How far the frame is turned, in degrees clockwise. */
	rotation?: number;
	/** Where the editing began, in the frame's points; without it the caret starts at the end. */
	start?: { x: number; y: number };
	/** Makes an edit; answers once the book has it, with where the caret goes. */
	onEdit(edit: EditRequest): Promise<{ caret: TextPos }>;
	/** The editing is over: Escape, or the focus left. */
	onClose(): void;
	/**
	 * Undoes or redoes one step of the book. Answers where the caret goes, when the step was an edit
	 * to words; the editor stays open either way.
	 */
	onUndo?(): Promise<EditCaret | null> | void;
	onRedo?(): Promise<EditCaret | null> | void;
	/** Offers corrections for misspelt words (the layout marks them) and Alt+F7 to find the next. */
	speller?: Speller;
	/** Tab was pressed; without this it does nothing. */
	onTab?(back: boolean): void;
}

/** What one keystroke asks of the words. */
interface Change {
	from: TextPos;
	to: TextPos;
	text: string;
	step: 'typing' | 'own';
	look?: RunChange;
	/** Paragraphs with their own looks, in place of `text`, which is then their plain words. */
	rich?: StoryPara[];
}

/**
 * Edits a frame's words in place. It draws no text of its own: the page already shows the words,
 * laid out by Rust, and this adds a caret and a selection over them, so what is edited wraps
 * exactly as the page does. Keys and input methods go to an invisible text area at the caret;
 * every change goes out as an `EditText` edit and the caret follows the answer.
 */
export function TextSurface({
	target,
	frameId,
	texts,
	content,
	layout,
	box,
	at,
	scale,
	rotation = 0,
	start,
	onEdit,
	onClose,
	onUndo,
	onRedo,
	onTab,
	speller,
}: TextSurfaceProps) {
	const surface = useRef<HTMLDivElement | null>(null);
	const sink = useRef<HTMLTextAreaElement | null>(null);
	const textsRef = useRef<string[]>([...texts]);
	const layoutRef = useRef(layout);
	layoutRef.current = layout;
	const [selection, setSelectionState] = useState<Selection>(() => {
		const end = endOf(texts);
		return { anchor: end, focus: end };
	});
	const selRef = useRef(selection);
	/** The x the caret keeps to while it goes up and down. */
	const goalX = useRef<number | undefined>(undefined);
	const [composing, setComposing] = useState<string | null>(null);
	const [blink, setBlink] = useState(0);

	const setSelection = useCallback((next: Selection, keepGoal = false) => {
		selRef.current = next;
		if (!keepGoal) goalX.current = undefined;
		setSelectionState(next);
		setBlink((n) => n + 1);
	}, []);

	// The book changed under the editor (an undo, an agent): the words are the book's, and the
	// selection is kept inside them.
	const wordsKey = texts.join('\u0000');
	useEffect(() => {
		textsRef.current = [...texts];
		const clamp = {
			anchor: clampPos(texts, selRef.current.anchor),
			focus: clampPos(texts, selRef.current.focus),
		};
		if (
			!samePos(clamp.anchor, selRef.current.anchor) ||
			!samePos(clamp.focus, selRef.current.focus)
		)
			setSelection(clamp);
		// eslint-disable-next-line react-hooks/exhaustive-deps
	}, [wordsKey, setSelection]);

	// The ribbon styles the selected words, so it is told which they are.
	useEffect(() => {
		if (target.type === 'frame' || target.type === 'shape') {
			const [a, b] = ordered(selection.anchor, selection.focus);
			setTextSelection({
				frame: frameId,
				start: flatOffset(textsRef.current, a),
				end: flatOffset(textsRef.current, b),
			});
		}
	}, [selection, target.type, frameId]);
	useEffect(() => {
		if (target.type !== 'frame' && target.type !== 'shape') return;
		const collapsed = samePos(selection.anchor, selection.focus);
		setTextCaret(
			collapsed ? { frame: frameId, offset: flatOffset(textsRef.current, selection.focus) } : null,
		);
	}, [selection, target.type, frameId]);
	useEffect(
		() => () => {
			setTextSelection(null);
			setTextCaret(null);
			clearPendingLook();
		},
		[],
	);

	// Where the editing began: the point that was clicked, once the layout can say what is there.
	const placed = useRef(false);
	useLayoutEffect(() => {
		if (placed.current || !layout) return;
		placed.current = true;
		if (start) {
			const hit = hitTest(layout, start.x, start.y, (p, o) =>
				snapToGrapheme(textsRef.current[p] ?? '', o),
			);
			if (hit) setSelection({ anchor: hit, focus: hit });
		} else if (target.type === 'story') {
			// A frame of a story shows only a part of it: the caret starts at what the frame shows first.
			const first = layout.lines.find((l) => l.paragraph != null);
			if (first && first.paragraph != null) {
				const pos = { para: first.paragraph, offset: first.start };
				setSelection({ anchor: pos, focus: pos });
			}
		}
	}, [layout, start, setSelection, target.type]);

	useEffect(() => {
		sink.current?.focus({ preventScroll: true });
	}, []);

	// Edits go one at a time: a key pressed while one is in flight waits, and takes its place from
	// the answer before it.
	const chain = useRef<Promise<void>>(Promise.resolve());
	const edit = useRef(onEdit);
	edit.current = onEdit;
	const queue = useCallback(
		(build: () => Change | null) => {
			chain.current = chain.current.then(async () => {
				const change = build();
				if (!change) return;
				const request: EditRequest = {
					target,
					from: change.from,
					to: change.to,
					insert: change.rich
						? { type: 'rich', paragraphs: change.rich }
						: { type: 'text', text: change.text },
					step: change.step,
					...(change.look ? { look: change.look } : {}),
				};
				try {
					const answer = await edit.current(request);
					textsRef.current = replaceInTexts(textsRef.current, change.from, change.to, change.text);
					const caret = clampPos(textsRef.current, answer.caret);
					setSelection({ anchor: caret, focus: caret });
				} catch (reason) {
					console.warn('The edit was refused:', reason);
				}
			});
		},
		[target, setSelection],
	);

	/** Undoes or redoes a step after the edits before it, and puts the caret where the step had it. */
	const undoer = useRef(onUndo);
	undoer.current = onUndo;
	const redoer = useRef(onRedo);
	redoer.current = onRedo;
	const history = useCallback(
		(which: 'undo' | 'redo') => {
			chain.current = chain.current.then(async () => {
				try {
					const caret = await (which === 'undo' ? undoer.current : redoer.current)?.();
					if (caret && JSON.stringify(caret.target) === JSON.stringify(target)) {
						setSelection({ anchor: caret.anchor, focus: caret.focus });
					}
				} catch (reason) {
					console.warn(`Could not ${which}:`, reason);
				}
			});
		},
		[target, setSelection],
	);

	/** Replaces the selection with `text`. */
	const type = useCallback(
		(text: string, step: 'typing' | 'own') => {
			const clean = text.replace(/\r\n?/g, '\n');
			if (clean === '') return;
			queue(() => {
				const [from, to] = ordered(selRef.current.anchor, selRef.current.focus);
				// A look set at the caret goes with the first words typed after it, as a step of its own.
				const look = getPendingLook(frameId) ?? undefined;
				if (look) clearPendingLook();
				return { from, to, text: clean, step: look ? 'own' : step, ...(look ? { look } : {}) };
			});
		},
		[queue, frameId],
	);

	/** Deletes the selection, or a grapheme or word from the caret. */
	const remove = useCallback(
		(unit: 'grapheme' | 'word' | 'line', dir: -1 | 1) => {
			queue(() => {
				const { anchor, focus } = selRef.current;
				if (!samePos(anchor, focus)) {
					const [from, to] = ordered(anchor, focus);
					return { from, to, text: '', step: 'own' };
				}
				const words = textsRef.current;
				const far =
					unit === 'grapheme'
						? moveGrapheme(words, focus, dir)
						: unit === 'word'
							? moveWord(words, focus, dir)
							: layoutRef.current
								? moveLineEdge(layoutRef.current, focus, dir < 0 ? 'home' : 'end')
								: focus;
				if (samePos(far, focus)) return null;
				const [from, to] = ordered(focus, far);
				return { from, to, text: '', step: unit === 'grapheme' ? 'typing' : 'own' };
			});
		},
		[queue],
	);

	const snap = (para: number, offset: number) =>
		snapToGrapheme(textsRef.current[para] ?? '', offset);

	/** Where a pointer is in the frame's points, undoing the frame's turn. */
	const toLocal = (clientX: number, clientY: number) => {
		const rect = surface.current?.getBoundingClientRect();
		if (!rect) return { x: 0, y: 0 };
		const dx = clientX - (rect.left + rect.width / 2);
		const dy = clientY - (rect.top + rect.height / 2);
		const a = (rotation * Math.PI) / 180;
		const x = dx * Math.cos(a) + dy * Math.sin(a);
		const y = -dx * Math.sin(a) + dy * Math.cos(a);
		return { x: (x + (box.w * scale) / 2) / scale, y: (y + (box.h * scale) / 2) / scale };
	};

	const dragging = useRef(false);
	useEffect(() => {
		const move = (event: MouseEvent) => {
			if (!dragging.current || !layoutRef.current) return;
			const p = toLocal(event.clientX, event.clientY);
			const hit = hitTest(layoutRef.current, p.x, p.y, snap);
			if (hit) setSelection({ anchor: selRef.current.anchor, focus: hit });
		};
		const up = () => {
			dragging.current = false;
		};
		window.addEventListener('mousemove', move);
		window.addEventListener('mouseup', up);
		return () => {
			window.removeEventListener('mousemove', move);
			window.removeEventListener('mouseup', up);
		};
		// eslint-disable-next-line react-hooks/exhaustive-deps
	}, [scale, rotation, box.w, box.h]);

	const onMouseDown = (event: React.MouseEvent) => {
		if (event.button !== 0 || !layout) return;
		// The focus stays in the sink, whatever is clicked.
		event.preventDefault();
		event.stopPropagation();
		clearPendingLook();
		sink.current?.focus({ preventScroll: true });
		const p = toLocal(event.clientX, event.clientY);
		const hit = hitTest(layout, p.x, p.y, snap);
		if (!hit) return;
		const words = textsRef.current;
		if (event.detail >= 3) {
			setSelection({
				anchor: { para: hit.para, offset: 0 },
				focus: { para: hit.para, offset: cpLength(words[hit.para] ?? '') },
			});
		} else if (event.detail === 2) {
			const w = wordAt(words[hit.para] ?? '', hit.offset);
			setSelection({
				anchor: { para: hit.para, offset: w.start },
				focus: { para: hit.para, offset: w.end },
			});
		} else {
			setSelection({ anchor: event.shiftKey ? selRef.current.anchor : hit, focus: hit });
			dragging.current = true;
		}
	};

	/** Moves the caret, or the selection's far end with Shift. */
	const go = (focus: TextPos, shift: boolean, keepGoal = false) => {
		clearPendingLook();
		setSelection({ anchor: shift ? selRef.current.anchor : focus, focus }, keepGoal);
	};

	const onKeyDown = (event: React.KeyboardEvent) => {
		if (event.nativeEvent.isComposing || event.keyCode === 229) return;
		const mod = event.ctrlKey || event.metaKey;
		const { shift } = { shift: event.shiftKey };
		const words = textsRef.current;
		const { anchor, focus } = selRef.current;
		const collapsed = samePos(anchor, focus);
		const [lo, hi] = ordered(anchor, focus);
		const key = event.key;
		const lay = layoutRef.current;
		const handled = () => {
			event.preventDefault();
			event.stopPropagation();
		};
		if (key === 'ArrowLeft' || key === 'ArrowRight') {
			handled();
			const dir = key === 'ArrowLeft' ? -1 : 1;
			if (!shift && !collapsed && !mod) go(dir < 0 ? lo : hi, false);
			else go(mod ? moveWord(words, focus, dir) : moveGrapheme(words, focus, dir), shift);
		} else if (key === 'ArrowUp' || key === 'ArrowDown') {
			handled();
			if (!lay) return;
			const moved = moveVertical(lay, focus, key === 'ArrowUp' ? -1 : 1, goalX.current);
			goalX.current = moved.goalX;
			go(moved.pos, shift, true);
		} else if (key === 'Home' || key === 'End') {
			handled();
			if (mod) go(key === 'Home' ? { para: 0, offset: 0 } : endOf(words), shift);
			else if (lay) go(moveLineEdge(lay, focus, key === 'Home' ? 'home' : 'end'), shift);
		} else if (mod && key.toLowerCase() === 'a') {
			handled();
			setSelection({ anchor: { para: 0, offset: 0 }, focus: endOf(words) });
		} else if (speller && key === 'F7' && event.altKey) {
			handled();
			nextMiss();
		} else if (speller && ((key === 'F10' && shift) || key === 'ContextMenu')) {
			const word = missAt(focus);
			if (!word) return;
			handled();
			const rect = lay ? caretRect(lay, focus) : undefined;
			const bounds = surface.current?.getBoundingClientRect();
			openMenu(
				word,
				{
					x: (bounds?.left ?? 0) + (rect?.x ?? 0) * scale,
					y: (bounds?.top ?? 0) + ((rect?.y ?? 0) + (rect?.h ?? 0)) * scale,
				},
				true,
			);
		} else if (key === 'Enter') {
			handled();
			type('\n', 'own');
		} else if (key === 'Backspace' || key === 'Delete') {
			handled();
			remove(mod ? 'word' : 'grapheme', key === 'Backspace' ? -1 : 1);
		} else if (key === 'Escape') {
			handled();
			onClose();
		} else if (key === 'Tab') {
			handled();
			onTab?.(shift);
		} else if (mod && key.toLowerCase() === 'z') {
			handled();
			history(shift ? 'redo' : 'undo');
		} else if (mod && key.toLowerCase() === 'y') {
			handled();
			history('redo');
		}
	};

	// `beforeinput` carries what was typed, as the keyboard layout and dead keys made it. React's own
	// onBeforeInput is a keypress shim, so the real event is listened for.
	useEffect(() => {
		const el = sink.current;
		if (!el) return;
		const before = (event: InputEvent) => {
			if (event.isComposing || event.inputType === 'insertCompositionText') return;
			event.preventDefault();
			switch (event.inputType) {
				case 'insertText':
					if (event.data) type(event.data, 'typing');
					break;
				case 'insertLineBreak':
				case 'insertParagraph':
					type('\n', 'own');
					break;
				case 'insertFromPaste':
				case 'insertFromDrop':
				case 'insertReplacementText':
					type(event.data ?? event.dataTransfer?.getData('text/plain') ?? '', 'own');
					break;
				case 'deleteContentBackward':
					remove('grapheme', -1);
					break;
				case 'deleteContentForward':
					remove('grapheme', 1);
					break;
				case 'deleteWordBackward':
					remove('word', -1);
					break;
				case 'deleteWordForward':
					remove('word', 1);
					break;
				case 'deleteSoftLineBackward':
				case 'deleteHardLineBackward':
					remove('line', -1);
					break;
				case 'deleteSoftLineForward':
				case 'deleteHardLineForward':
					remove('line', 1);
					break;
				case 'historyUndo':
					history('undo');
					break;
				case 'historyRedo':
					history('redo');
					break;
				default:
					break;
			}
		};
		el.addEventListener('beforeinput', before);
		return () => el.removeEventListener('beforeinput', before);
	}, [type, remove, history]);

	const copy = (event: React.ClipboardEvent) => {
		event.preventDefault();
		const [lo, hi] = ordered(selRef.current.anchor, selRef.current.focus);
		if (samePos(lo, hi)) return;
		const data = event.clipboardData;
		if (content) {
			// Folio keeps the look; other programs get HTML; everything else gets the words.
			const paras = sliceParagraphs(content, lo, hi);
			data.setData('text/plain', toPlain(paras));
			data.setData('text/html', toHtml(paras));
			data.setData(FOLIO_TEXT_MIME, JSON.stringify(paras));
		} else {
			data.setData('text/plain', textBetween(textsRef.current, lo, hi));
		}
	};
	const cut = (event: React.ClipboardEvent) => {
		copy(event);
		if (!samePos(selRef.current.anchor, selRef.current.focus)) remove('grapheme', -1);
	};
	const paste = (event: React.ClipboardEvent) => {
		event.preventDefault();
		const rich = fromClipboard(event.clipboardData.getData(FOLIO_TEXT_MIME));
		if (rich) {
			queue(() => {
				const [from, to] = ordered(selRef.current.anchor, selRef.current.focus);
				return { from, to, text: toPlain(rich), step: 'own', rich };
			});
			return;
		}
		type(event.clipboardData.getData('text/plain'), 'own');
	};

	// Composition: the sink shows what is being composed, and the finished words are one edit.
	const compositionEnd = (event: React.CompositionEvent<HTMLTextAreaElement>) => {
		const text = event.nativeEvent.data ?? '';
		setComposing(null);
		if (sink.current) sink.current.value = '';
		if (text) type(text, 'typing');
	};

	// The spelling menu: the word, what else it might be and where on screen to open.
	const [menu, setMenu] = useState<{
		at: { x: number; y: number };
		word: Misspelt;
		text: string;
		suggestions: string[];
		keyboard: boolean;
	} | null>(null);
	const menuUp = useRef(false);
	menuUp.current = menu !== null;

	const openMenu = (word: Misspelt, at: { x: number; y: number }, keyboard: boolean) => {
		if (!speller) return;
		const text = cpSlice(textsRef.current[word.para] ?? '', word.start, word.end);
		void speller
			.suggest(text)
			.catch(() => [] as string[])
			.then((suggestions) =>
				setMenu({ at, word, text, suggestions: suggestions.slice(0, 6), keyboard }),
			);
	};
	const missAt = (pos: TextPos) =>
		layoutRef.current
			? misspellings(layoutRef.current).find(
					(m) => m.para === pos.para && pos.offset >= m.start && pos.offset <= m.end,
				)
			: undefined;
	const onContextMenu = (event: React.MouseEvent) => {
		if (!speller || !layout) return;
		const p = toLocal(event.clientX, event.clientY);
		const hit = hitTest(layout, p.x, p.y, snap);
		const word = hit && missAt(hit);
		if (!word) return;
		event.preventDefault();
		openMenu(word, { x: event.clientX, y: event.clientY }, false);
	};
	/** Alt+F7: the next misspelt word after the caret, selected. */
	const nextMiss = () => {
		if (!layoutRef.current) return;
		const all = misspellings(layoutRef.current);
		const at = selRef.current.focus;
		const next =
			all.find((m) => m.para > at.para || (m.para === at.para && m.start > at.offset)) ?? all[0];
		if (next) {
			setSelection({
				anchor: { para: next.para, offset: next.start },
				focus: { para: next.para, offset: next.end },
			});
		}
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

	const caret = layout ? caretRect(layout, selection.focus) : undefined;
	const rects =
		layout && !samePos(selection.anchor, selection.focus)
			? selectionRects(layout, selection.anchor, selection.focus)
			: [];
	const line = layout?.lines.find((l) => l.paragraph === selection.focus.para) ?? layout?.lines[0];
	const run = line?.runs[0];
	const size = (run?.size ?? 12) * scale;

	return (
		<div
			ref={surface}
			className={styles.surface}
			data-text-surface=""
			style={{
				left: at.x,
				top: at.y,
				width: box.w * scale,
				height: box.h * scale,
				transform: rotation ? `rotate(${rotation}deg)` : undefined,
				transformOrigin: rotation ? `${(box.w * scale) / 2}px ${(box.h * scale) / 2}px` : undefined,
			}}
			onMouseDown={onMouseDown}
			onContextMenu={onContextMenu}
		>
			{rects.map((r, i) => (
				<div
					key={i}
					className={styles.selection}
					data-selection=""
					style={{ left: r.x * scale, top: r.y * scale, width: r.w * scale, height: r.h * scale }}
				/>
			))}
			{caret && composing === null && samePos(selection.anchor, selection.focus) && (
				<div
					key={blink}
					className={styles.caret}
					data-caret=""
					style={{ left: caret.x * scale, top: caret.y * scale, height: caret.h * scale }}
				/>
			)}
			<textarea
				ref={sink}
				className={composing === null ? styles.sink : `${styles.sink} ${styles.composing}`}
				aria-label={t('stage.edit.text')}
				rows={1}
				wrap="off"
				spellCheck={false}
				autoCapitalize="off"
				autoCorrect="off"
				autoComplete="off"
				onKeyDown={onKeyDown}
				onCopy={copy}
				onCut={cut}
				onPaste={paste}
				onCompositionStart={() => setComposing('')}
				onCompositionUpdate={(event) => setComposing(event.nativeEvent.data ?? '')}
				onCompositionEnd={compositionEnd}
				onChange={() => undefined}
				onBlur={(event) => {
					// A list in the ribbon takes the focus to open, and the words stay selected for it to style.
					const to = event.relatedTarget;
					if (to instanceof Element && to.closest('[data-ribbon]')) return;
					// The spelling menu takes the focus while it is up; that is not the end of the editing.
					if (menuUp.current) return;
					onClose();
				}}
				style={{
					left: (caret?.x ?? 0) * scale,
					top: (caret?.y ?? 0) * scale,
					height: (caret?.h ?? 14) * scale,
					fontFamily: run ? `'${run.family}', serif` : undefined,
					fontSize: size,
					lineHeight: `${(caret?.h ?? 14) * scale}px`,
					width: composing === null ? 2 : Math.max(4, (composing.length + 1) * size * 0.6),
				}}
			/>
			{menu && (
				<ContextMenu
					items={menuItems()}
					position={menu.at}
					ariaLabel={t('spell.menu')}
					openedWithKeyboard={menu.keyboard}
					returnFocusTo={sink.current}
					onSelect={(item) => {
						const { word, text } = menu;
						setMenu(null);
						if (item.id.startsWith('fix:')) {
							const fix = menu.suggestions[Number(item.id.slice(4))];
							if (fix !== undefined) {
								queue(() => ({
									from: { para: word.para, offset: word.start },
									to: { para: word.para, offset: word.end },
									text: fix,
									step: 'own',
								}));
							}
						} else if (item.id === 'ignore') speller?.ignore(text);
						else if (item.id === 'add') speller?.addToDictionary(text);
						sink.current?.focus({ preventScroll: true });
					}}
					onClose={() => {
						setMenu(null);
						sink.current?.focus({ preventScroll: true });
					}}
				/>
			)}
		</div>
	);
}
