// The cover sheet as a page: back, spine and front drawn together, with words, pictures and shapes
// that can be picked up, moved across the folds, stretched and typed in
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { BackNote } from '@liminal-hq/folio-protocol/generated/BackNote';
import type { Book } from '@liminal-hq/folio-protocol/generated/Book';
import type { CoverCanvas as Sheet } from '@liminal-hq/folio-protocol/generated/CoverCanvas';
import type { Shape } from '@liminal-hq/folio-protocol/generated/Shape';
import { useEffect, useMemo, useRef, useState } from 'react';
import type { BooksClient } from '../books/booksClient';
import { useAssetUrls } from '../canvas/useAssetUrls';
import { PX_PER_POINT } from '../canvas/geometry';
import { SpreadCanvas } from '../canvas/SpreadCanvas';
import type { FrameBoxMove } from '../canvas/useFrameDrag';
import { t, type MessageKey } from '../i18n/messages';
import styles from './CoverCanvas.module.css';

/** The id a note has on the sheet: Folio's own frames on it are named `cover_…`. */
const NOTE = 'cover_note_';
const noteOf = (frame: string) => (frame.startsWith(NOTE) ? frame.slice(NOTE.length) : null);

interface CoverCanvasProps {
	client: BooksClient;
	id: string;
	/** Changes when the book does, so the sheet is read again. */
	revision: number;
	/** The book itself: its cover spread is where the front's frames live. */
	book: Book;
	/** Keeps new words, pictures and shapes on the sheet, whole (one undo step). */
	onNotes(notes: BackNote[]): void;
	/** New boxes for frames of the front, in the cover spread's own points. */
	onFrameBoxes(
		moves: { frame: string; box: { x: number; y: number; w: number; h: number } }[],
	): void;
	/** New words for a frame of the front. */
	onFrameText(frame: string, text: string): void;
	/** A frame of the front turned, in degrees clockwise. */
	onFrameTurn(frame: string, degrees: number): void;
	/** Takes frames of the front away. */
	onFrameDelete(frames: string[]): void;
	/** Asks for a picture and brings it into the book; its path, or `null` if none was chosen. */
	onPickPicture(): Promise<string | null>;
}

const SHAPES: Shape[] = ['rect', 'round', 'ellipse'];
const SHAPE_NAMES = {
	rect: 'coverSheet.shape.rect',
	round: 'coverSheet.shape.round',
	ellipse: 'coverSheet.shape.ellipse',
} as unknown as Record<Shape, MessageKey>;

export function CoverCanvas({
	client,
	id,
	revision,
	book,
	onNotes,
	onFrameBoxes,
	onFrameText,
	onFrameTurn,
	onFrameDelete,
	onPickPicture,
}: CoverCanvasProps) {
	const [sheet, setSheet] = useState<Sheet | null | undefined>(undefined);
	const [selection, setSelection] = useState<string[]>([]);
	const [shape, setShape] = useState<Shape>('rect');
	const host = useRef<HTMLDivElement | null>(null);
	const [width, setWidth] = useState(0);
	const notes = book.backNotes ?? [];

	useEffect(() => {
		let live = true;
		client
			.coverSheetCanvas(id)
			.then((found) => live && setSheet(found))
			.catch(() => live && setSheet(null));
		return () => {
			live = false;
		};
	}, [client, id, revision]);

	useEffect(() => {
		const el = host.current;
		if (!el) return;
		const measure = () => setWidth(el.getBoundingClientRect().width);
		measure();
		if (typeof ResizeObserver === 'undefined') return;
		const watch = new ResizeObserver(measure);
		watch.observe(el);
		return () => watch.disconnect();
	}, [sheet]);

	const pictures = useMemo(
		() =>
			(sheet?.book.spreads[0]?.frames ?? []).flatMap((f) =>
				f.type === 'image' && f.image.asset ? [f.image.asset] : [],
			),
		[sheet],
	);
	const images = useAssetUrls(client, id, pictures);

	if (sheet === undefined) return <p role="status">{t('coverSheet.drawing')}</p>;
	if (sheet === null) return <p>{t('coverSheet.none')}</p>;

	const spread = sheet.book.spreads[0]!;
	const sheetWidth = sheet.book.trim.w + 2 * sheet.book.bleed;
	const zoom = width > 0 ? Math.max(0.05, width / (sheetWidth * PX_PER_POINT)) : 1;
	/** How far the front's frames are moved along on the sheet from where the cover spread has them. */
	const along = sheet.cover + sheet.spine;
	const nextId = () =>
		`n${notes.reduce((top, n) => Math.max(top, Number(n.id.replace(/\D/g, '')) || 0), 0) + 1}`;
	const add = (more: Partial<BackNote>) => {
		const noteId = nextId();
		onNotes([
			...notes,
			{
				id: noteId,
				text: '',
				rect: { x: 36, y: 36, w: Math.min(160, Math.max(40, sheet.cover - 72)), h: 60 },
				size: 12,
				...more,
			},
		]);
		setSelection([`${NOTE}${noteId}`]);
	};
	const chosen = notes.find((n) => selection.length === 1 && `${NOTE}${n.id}` === selection[0]);

	const moveBoxes = async (moves: FrameBoxMove[]) => {
		const front: { frame: string; box: { x: number; y: number; w: number; h: number } }[] = [];
		let next = notes;
		for (const move of moves) {
			const note = noteOf(move.id);
			if (note !== null) {
				next = next.map((n) =>
					n.id === note
						? { ...n, rect: { x: move.rect.x, y: move.rect.y, w: move.rect.w, h: move.rect.h } }
						: n,
				);
			} else if (!move.id.startsWith('cover_')) {
				front.push({ frame: move.id, box: { ...move.rect, x: move.rect.x - along } });
			}
		}
		if (next !== notes) onNotes(next);
		if (front.length > 0) onFrameBoxes(front);
	};

	return (
		<div className={styles.canvas}>
			<div className={styles.tools}>
				<button type="button" onClick={() => add({ text: t('coverSheet.newNote') })}>
					{t('coverSheet.addNote')}
				</button>
				<button
					type="button"
					onClick={() =>
						void onPickPicture().then((picture) => {
							if (picture !== null) add({ picture, rect: { x: 36, y: 36, w: 120, h: 120 } });
						})
					}
				>
					{t('coverSheet.addPicture')}
				</button>
				<label>
					<span>{t('coverSheet.shapeKind')}</span>
					<select value={shape} onChange={(event) => setShape(event.target.value as Shape)}>
						{SHAPES.map((s) => (
							<option key={s} value={s}>
								{t(SHAPE_NAMES[s])}
							</option>
						))}
					</select>
				</label>
				<button
					type="button"
					onClick={() => add({ shape, fill: '#e8833a', rect: { x: 36, y: 36, w: 100, h: 100 } })}
				>
					{t('coverSheet.addShape')}
				</button>
				{chosen?.shape && (
					<label>
						<span>{t('coverSheet.fill')}</span>
						<input
							type="color"
							value={chosen.fill ?? '#e8833a'}
							onChange={(event) =>
								onNotes(
									notes.map((n) => (n.id === chosen.id ? { ...n, fill: event.target.value } : n)),
								)
							}
						/>
					</label>
				)}
			</div>
			<div ref={host} className={styles.page}>
				<SpreadCanvas
					book={sheet.book}
					spread={spread}
					layout={sheet.layout}
					zoom={zoom}
					label={t('coverSheet.canvas')}
					images={images}
					selection={selection}
					onSelect={setSelection}
					onMove={moveBoxes}
					onTurn={async (frame, degrees) => {
						const note = noteOf(frame);
						if (note !== null) {
							onNotes(notes.map((n) => (n.id === note ? { ...n, rotation: degrees } : n)));
						} else if (!frame.startsWith('cover_')) onFrameTurn(frame, degrees);
					}}
					onEditText={(frame, text) => {
						const note = noteOf(frame);
						if (note !== null) onNotes(notes.map((n) => (n.id === note ? { ...n, text } : n)));
						else if (!frame.startsWith('cover_')) onFrameText(frame, text);
					}}
					onCommand={(command, ids) => {
						if (command !== 'delete') return;
						const gone = new Set(ids.map(noteOf).filter((n): n is string => n !== null));
						if (gone.size > 0) onNotes(notes.filter((n) => !gone.has(n.id)));
						const front = ids.filter((f) => noteOf(f) === null && !f.startsWith('cover_'));
						if (front.length > 0) onFrameDelete(front);
						setSelection([]);
					}}
				/>
			</div>
			<p className={styles.hint}>{t('coverSheet.canvasHint')}</p>
		</div>
	);
}
