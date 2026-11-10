// Editing words on the page: a caret and a selection over the page's own lines, and an invisible input for the keys
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { EditRequest } from '@liminal-hq/folio-protocol/generated/EditRequest';
import type { FrameLayout } from '@liminal-hq/folio-protocol/generated/FrameLayout';
import type { RunChange } from '@liminal-hq/folio-protocol/generated/RunChange';
import type { TextPos } from '@liminal-hq/folio-protocol/generated/TextPos';
import type { TextTarget } from '@liminal-hq/folio-protocol/generated/TextTarget';
import { useCallback, useEffect, useLayoutEffect, useRef, useState } from 'react';
import { setTextSelection } from '../canvas/textSelection';
import { t } from '../i18n/messages';
import { caretRect, hitTest, selectionRects } from './caretGeometry';
import { clampPos, endOf, flatOffset, ordered, replaceInTexts, samePos, textBetween } from './edit';
import { moveGrapheme, moveLineEdge, moveVertical, moveWord, wordAt } from './navigate';
import { cpLength, snapToGrapheme } from './units';
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
	onUndo?(): void;
	onRedo?(): void;
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
	useEffect(() => () => setTextSelection(null), []);

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
		}
	}, [layout, start, setSelection]);

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
					insert: { type: 'text', text: change.text },
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

	/** Replaces the selection with `text`. */
	const type = useCallback(
		(text: string, step: 'typing' | 'own') => {
			const clean = text.replace(/\r\n?/g, '\n');
			if (clean === '') return;
			queue(() => {
				const [from, to] = ordered(selRef.current.anchor, selRef.current.focus);
				return { from, to, text: clean, step };
			});
		},
		[queue],
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
			if (shift) onRedo?.();
			else onUndo?.();
		} else if (mod && key.toLowerCase() === 'y') {
			handled();
			onRedo?.();
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
					onUndo?.();
					break;
				case 'historyRedo':
					onRedo?.();
					break;
				default:
					break;
			}
		};
		el.addEventListener('beforeinput', before);
		return () => el.removeEventListener('beforeinput', before);
	}, [type, remove, onUndo, onRedo]);

	const copy = (event: React.ClipboardEvent) => {
		event.preventDefault();
		const [lo, hi] = ordered(selRef.current.anchor, selRef.current.focus);
		if (samePos(lo, hi)) return;
		event.clipboardData.setData('text/plain', textBetween(textsRef.current, lo, hi));
	};
	const cut = (event: React.ClipboardEvent) => {
		copy(event);
		if (!samePos(selRef.current.anchor, selRef.current.focus)) remove('grapheme', -1);
	};
	const paste = (event: React.ClipboardEvent) => {
		event.preventDefault();
		type(event.clipboardData.getData('text/plain'), 'own');
	};

	// Composition: the sink shows what is being composed, and the finished words are one edit.
	const compositionEnd = (event: React.CompositionEvent<HTMLTextAreaElement>) => {
		const text = event.nativeEvent.data ?? '';
		setComposing(null);
		if (sink.current) sink.current.value = '';
		if (text) type(text, 'typing');
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
		</div>
	);
}
