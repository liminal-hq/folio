// The cover sheet preview: back, spine and front as the paperback cover file will have them
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { Dialog } from '@liminal-hq/folio-chrome/Dialog/Dialog';
import { DialogActions, DialogButton } from '@liminal-hq/folio-chrome/Dialog/DialogActions';
import type { BackNote } from '@liminal-hq/folio-protocol/generated/BackNote';
import { useEffect, useRef, useState } from 'react';
import { CoverCanvas } from './CoverCanvas';
import type { Book } from '@liminal-hq/folio-protocol/generated/Book';
import type { CoverSheetPreview } from '@liminal-hq/folio-protocol/generated/CoverSheetPreview';
import type { Details } from '@liminal-hq/folio-protocol/generated/Details';
import type { BooksClient } from '../books/booksClient';
import { t } from '../i18n/messages';
import styles from './CoverSheetDialog.module.css';

interface CoverSheetDialogProps {
	open: boolean;
	onClose(): void;
	client: BooksClient;
	id: string;
	/** Changes when the book does, so the sheet is drawn again. */
	revision: number;
	/** The book, whose title, author and description are the words on the sheet. */
	book: Book;
	/** Keeps new words for the sheet as the book's details (one undo step). */
	onSave(details: Details): void;
	/** Keeps new words on the back cover, whole (one undo step). */
	onNotes(notes: BackNote[]): void;
	/** Asks for a picture and brings it into the book; its path, or `null` if none was chosen. */
	onPickPicture(): Promise<string | null>;
	/** New boxes for frames of the front, in the cover spread's own points. */
	onFrameBoxes(
		moves: { frame: string; box: { x: number; y: number; w: number; h: number } }[],
	): void;
	onFrameText(frame: string, text: string): void;
	onFrameTurn(frame: string, degrees: number): void;
	onFrameDelete(frames: string[]): void;
}

export function CoverSheetDialog({
	open,
	onClose,
	client,
	id,
	revision,
	book,
	onSave,
	onNotes,
	onPickPicture,
	onFrameBoxes,
	onFrameText,
	onFrameTurn,
	onFrameDelete,
}: CoverSheetDialogProps) {
	/** Whether the sheet is drawn as printed (a picture) or as a page to draw on. */
	const [mode, setMode] = useState<'preview' | 'edit'>('preview');
	const notes = book.backNotes ?? [];
	const [chosen, setChosen] = useState<string | null>(null);
	/** A note being dragged, and where it would land. */
	const [dragging, setDragging] = useState<{ id: string; x: number; y: number } | null>(null);
	const current = notes.find((n) => n.id === chosen) ?? null;
	const [draft, setDraft] = useState('');
	useEffect(() => setDraft(current?.text ?? ''), [current?.id, current?.text]);
	const change = (id: string, patch: Partial<BackNote>) =>
		onNotes(notes.map((n) => (n.id === id ? { ...n, ...patch } : n)));
	const addPicture = async (sheet: CoverSheetPreview) => {
		const asset = await onPickPicture();
		if (asset === null) return;
		const next =
			notes.reduce((top, n) => Math.max(top, Number(n.id.replace(/\D/g, '')) || 0), 0) + 1;
		const side = Math.min(160, Math.max(40, sheet.cover - 72));
		onNotes([
			...notes,
			{
				id: `n${next}`,
				text: '',
				rect: { x: 36, y: 36, w: side, h: side },
				size: 12,
				picture: asset,
			},
		]);
		setChosen(`n${next}`);
	};
	const addNote = (sheet: CoverSheetPreview) => {
		const next =
			notes.reduce((top, n) => Math.max(top, Number(n.id.replace(/\D/g, '')) || 0), 0) + 1;
		const id = `n${next}`;
		const trimHeight = sheet.height - 2 * sheet.edge;
		onNotes([
			...notes,
			{
				id,
				text: t('coverSheet.newNote'),
				rect: {
					x: 36,
					y: Math.max(36, (trimHeight - 60) / 2),
					w: Math.max(40, sheet.cover - 72),
					h: 60,
				},
				size: 12,
			},
		]);
		setChosen(id);
	};
	const [words, setWords] = useState(() => wordsOf(book));
	useEffect(() => {
		if (open) setWords(wordsOf(book));
		// The fields start from the book each time the sheet opens, not as the book changes under them.
		// eslint-disable-next-line react-hooks/exhaustive-deps
	}, [open]);
	const changed = JSON.stringify(words) !== JSON.stringify(wordsOf(book));
	const keep = () => {
		const [first, ...others] = book.authors;
		const name = words.author.trim();
		onSave({
			title: words.title.trim() || book.title,
			subtitle: book.subtitle ?? null,
			authors: name ? [{ name, role: first?.role ?? 'author' }, ...others] : others,
			description: words.description.trim(),
			isbn: book.metadata.isbn,
		});
	};
	const [picture, setPicture] = useState<CoverSheetPreview | null | undefined>(undefined);
	const [guides, setGuides] = useState(true);
	const [failed, setFailed] = useState(false);
	useEffect(() => {
		if (!open) return;
		let live = true;
		// The picture on show stays until the new one is drawn, so a change does not blink the dialog.
		setFailed(false);
		client
			.previewCoverSheet(id)
			.then((found) => live && setPicture(found))
			.catch(() => live && setFailed(true));
		return () => {
			live = false;
		};
	}, [open, client, id, revision]);

	return (
		<Dialog
			open={open}
			title={t('coverSheet.title')}
			size="large"
			onClose={onClose}
			footer={
				<DialogActions>
					<DialogButton variant="primary" onClick={onClose}>
						{t('coverSheet.done')}
					</DialogButton>
				</DialogActions>
			}
		>
			<p className={styles.hint}>{t('coverSheet.hint')}</p>
			{failed && (
				<p role="alert" className={styles.problem}>
					{t('coverSheet.failed')}
				</p>
			)}
			{picture === undefined && !failed && (
				// A box the shape of the sheet, so the dialog is its full size before the picture arrives.
				<div
					role="status"
					className={styles.waiting}
					style={{ aspectRatio: `${2 * book.trim.w + 12} / ${book.trim.h + 18}` }}
				>
					{t('coverSheet.drawing')}
				</div>
			)}
			{picture === null && <p className={styles.hint}>{t('coverSheet.none')}</p>}
			{picture && (
				<div className={styles.modes} role="group" aria-label={t('coverSheet.title')}>
					{(['preview', 'edit'] as const).map((m) => (
						<button
							key={m}
							type="button"
							aria-pressed={mode === m}
							className={mode === m ? styles.modeOn : styles.mode}
							onClick={() => setMode(m)}
						>
							{t(m === 'preview' ? 'coverSheet.modePreview' : 'coverSheet.modeEdit')}
						</button>
					))}
				</div>
			)}
			{picture && mode === 'edit' && (
				<CoverCanvas
					client={client}
					id={id}
					revision={revision}
					book={book}
					onNotes={onNotes}
					onFrameBoxes={onFrameBoxes}
					onFrameText={onFrameText}
					onFrameTurn={onFrameTurn}
					onFrameDelete={onFrameDelete}
					onPickPicture={onPickPicture}
				/>
			)}
			{picture && mode === 'preview' && (
				<label className={styles.toggle}>
					<input
						type="checkbox"
						checked={guides}
						onChange={(event) => setGuides(event.target.checked)}
					/>
					{t('coverSheet.guides')}
				</label>
			)}
			{picture && mode === 'preview' && (
				<div className={styles.stage}>
					<img className={styles.sheet} src={picture.image} alt={t('coverSheet.alt')} />
					{guides && <Guides sheet={picture} />}
					<NoteBoxes
						sheet={picture}
						notes={notes}
						chosen={chosen}
						dragging={dragging}
						onChoose={setChosen}
						onDrag={setDragging}
						onMove={(id, x, y) =>
							change(id, { rect: { ...notes.find((n) => n.id === id)!.rect, x, y } })
						}
						onResize={(id, w, h) =>
							change(id, { rect: { ...notes.find((n) => n.id === id)!.rect, w, h } })
						}
					/>
				</div>
			)}
			{picture && mode === 'preview' && (
				<div className={styles.notes}>
					<DialogButton onClick={() => addNote(picture)}>{t('coverSheet.addNote')}</DialogButton>
					<DialogButton onClick={() => void addPicture(picture)}>
						{t('coverSheet.addPicture')}
					</DialogButton>
					{current && (
						<>
							{current.picture === null || current.picture === undefined ? (
								<>
									<label className={styles.field}>
										<span>{t('coverSheet.noteText')}</span>
										<textarea
											rows={2}
											value={draft}
											onChange={(event) => setDraft(event.target.value)}
											onBlur={() => draft !== current.text && change(current.id, { text: draft })}
										/>
									</label>
									<PointsField
										label={t('coverSheet.noteSize')}
										value={current.size}
										min={6}
										max={72}
										onCommit={(size) => change(current.id, { size })}
									/>
								</>
							) : null}
							<PointsField
								label={t('coverSheet.noteWidth')}
								value={current.rect.w}
								min={20}
								max={Math.max(20, (picture?.cover ?? 400) - current.rect.x)}
								onCommit={(w) => change(current.id, { rect: { ...current.rect, w } })}
							/>
							<PointsField
								label={t('coverSheet.noteHeight')}
								value={current.rect.h}
								min={10}
								max={Math.max(
									10,
									(picture ? picture.height - 2 * picture.edge : 600) - current.rect.y,
								)}
								onCommit={(h) => change(current.id, { rect: { ...current.rect, h } })}
							/>
							<DialogButton
								onClick={() => {
									onNotes(notes.filter((n) => n.id !== current.id));
									setChosen(null);
								}}
							>
								{t('coverSheet.deleteNote')}
							</DialogButton>
						</>
					)}
				</div>
			)}
			{picture && (
				<div className={styles.words}>
					<label className={styles.field}>
						<span>{t('coverSheet.titleField')}</span>
						<input
							type="text"
							value={words.title}
							onChange={(event) => setWords({ ...words, title: event.target.value })}
						/>
					</label>
					<label className={styles.field}>
						<span>{t('coverSheet.authorField')}</span>
						<input
							type="text"
							value={words.author}
							onChange={(event) => setWords({ ...words, author: event.target.value })}
						/>
					</label>
					<label className={styles.field}>
						<span>{t('coverSheet.blurbField')}</span>
						<textarea
							rows={4}
							value={words.description}
							onChange={(event) => setWords({ ...words, description: event.target.value })}
						/>
					</label>
					<div>
						<DialogButton variant="primary" disabled={!changed} onClick={keep}>
							{t('coverSheet.update')}
						</DialogButton>
					</div>
				</div>
			)}
		</Dialog>
	);
}

/** A number of points, edited as typed and kept (within its limits) when the field is left. */
function PointsField({
	label,
	value,
	min,
	max,
	onCommit,
}: {
	label: string;
	value: number;
	min: number;
	max: number;
	onCommit(value: number): void;
}) {
	const [draft, setDraft] = useState(String(value));
	useEffect(() => setDraft(String(value)), [value]);
	return (
		<label className={styles.field}>
			<span>{label}</span>
			<input
				type="number"
				min={min}
				max={max}
				value={draft}
				onChange={(event) => setDraft(event.target.value)}
				onBlur={() => {
					const next = Math.round(Math.min(max, Math.max(min, Number(draft) || value)));
					setDraft(String(next));
					if (next !== value) onCommit(next);
				}}
			/>
		</label>
	);
}

/** Keeps the pointer on the box while it is dragged; a pointer that has gone is not worth stopping for. */
function capture(event: React.PointerEvent<SVGElement>) {
	try {
		event.currentTarget.setPointerCapture?.(event.pointerId);
	} catch {
		// The drag still works while the pointer stays over the box.
	}
}

/** The notes on the back cover as boxes over the sheet, which can be picked up and moved. */
function NoteBoxes({
	sheet,
	notes,
	chosen,
	dragging,
	onChoose,
	onDrag,
	onMove,
	onResize,
}: {
	sheet: CoverSheetPreview;
	notes: BackNote[];
	chosen: string | null;
	dragging: { id: string; x: number; y: number } | null;
	onChoose(id: string): void;
	onDrag(drag: { id: string; x: number; y: number } | null): void;
	onMove(id: string, x: number, y: number): void;
	onResize(id: string, w: number, h: number): void;
}) {
	/** A box being stretched by its corner, and the size it would have. */
	const [sizing, setSizing] = useState<{ id: string; w: number; h: number } | null>(null);
	const svg = useRef<SVGSVGElement | null>(null);
	const grab = useRef<{ x: number; y: number } | null>(null);
	/** Sheet points for each screen pixel. */
	const unit = () => sheet.width / (svg.current?.getBoundingClientRect().width || sheet.width);
	return (
		<svg
			ref={svg}
			className={styles.noteLayer}
			viewBox={`0 0 ${sheet.width} ${sheet.height}`}
			preserveAspectRatio="none"
		>
			{notes.map((note) => {
				const at = dragging?.id === note.id ? dragging : note.rect;
				return (
					<rect
						key={note.id}
						role="button"
						tabIndex={0}
						aria-label={
							note.picture
								? t('coverSheet.pictureBox')
								: t('coverSheet.noteBox', { text: note.text })
						}
						data-note={note.id}
						x={sheet.edge + at.x}
						y={sheet.edge + at.y}
						width={sizing?.id === note.id ? sizing.w : note.rect.w}
						height={sizing?.id === note.id ? sizing.h : note.rect.h}
						className={note.id === chosen ? styles.noteChosen : styles.note}
						onPointerDown={(event) => {
							capture(event);
							onChoose(note.id);
							grab.current = { x: event.clientX, y: event.clientY };
							onDrag({ id: note.id, x: note.rect.x, y: note.rect.y });
						}}
						onPointerMove={(event) => {
							if (!grab.current || dragging?.id !== note.id) return;
							const scale = unit();
							onDrag({
								id: note.id,
								x: Math.round(note.rect.x + (event.clientX - grab.current.x) * scale),
								y: Math.round(note.rect.y + (event.clientY - grab.current.y) * scale),
							});
						}}
						onPointerUp={() => {
							if (
								dragging?.id === note.id &&
								(dragging.x !== note.rect.x || dragging.y !== note.rect.y)
							)
								onMove(note.id, dragging.x, dragging.y);
							grab.current = null;
							onDrag(null);
						}}
					/>
				);
			})}
			{notes
				.filter((note) => note.id === chosen)
				.map((note) => {
					const w = sizing?.id === note.id ? sizing.w : note.rect.w;
					const h = sizing?.id === note.id ? sizing.h : note.rect.h;
					const room = {
						w: Math.max(20, sheet.cover - note.rect.x),
						h: Math.max(10, sheet.height - 2 * sheet.edge - note.rect.y),
					};
					return (
						<rect
							key={`${note.id}-corner`}
							role="button"
							aria-label={t('coverSheet.noteCorner')}
							data-corner={note.id}
							x={sheet.edge + note.rect.x + w - 5}
							y={sheet.edge + note.rect.y + h - 5}
							width={10}
							height={10}
							className={styles.corner}
							onPointerDown={(event) => {
								capture(event);
								grab.current = { x: event.clientX, y: event.clientY };
								setSizing({ id: note.id, w: note.rect.w, h: note.rect.h });
							}}
							onPointerMove={(event) => {
								if (!grab.current || sizing?.id !== note.id) return;
								const scale = unit();
								setSizing({
									id: note.id,
									w: Math.round(
										Math.min(
											room.w,
											Math.max(20, note.rect.w + (event.clientX - grab.current.x) * scale),
										),
									),
									h: Math.round(
										Math.min(
											room.h,
											Math.max(10, note.rect.h + (event.clientY - grab.current.y) * scale),
										),
									),
								});
							}}
							onPointerUp={() => {
								if (
									sizing?.id === note.id &&
									(sizing.w !== note.rect.w || sizing.h !== note.rect.h)
								)
									onResize(note.id, sizing.w, sizing.h);
								grab.current = null;
								setSizing(null);
							}}
						/>
					);
				})}
		</svg>
	);
}

/** The words the sheet carries, as the form holds them. */
function wordsOf(book: Book) {
	return {
		title: book.title,
		author: book.authors[0]?.name ?? '',
		description: book.metadata.description,
	};
}

/** Points from the edge of a trimmed cover the safe area keeps text and pictures that need to stay. */
const SAFE = 18;
/** The same beside each fold of the spine. */
const SPINE_SAFE = 4.5;

/** The folds, the safe areas and the barcode box over the sheet, in the sheet's own points. */
function Guides({ sheet }: { sheet: CoverSheetPreview }) {
	const { width, height, edge, cover, spine, barcode } = sheet;
	const backRight = edge + cover;
	const frontLeft = backRight + spine;
	const safe = (x: number, w: number, key: string) => (
		<rect
			key={key}
			x={x}
			y={edge + SAFE}
			width={w}
			height={height - 2 * (edge + SAFE)}
			className={styles.safe}
		/>
	);
	return (
		<svg
			className={styles.guides}
			viewBox={`0 0 ${width} ${height}`}
			preserveAspectRatio="none"
			aria-hidden="true"
			data-testid="cover-guides"
		>
			{safe(edge + SAFE, cover - 2 * SAFE, 'back')}
			{safe(frontLeft + SAFE, cover - 2 * SAFE, 'front')}
			{spine > 2 * SPINE_SAFE && safe(backRight + SPINE_SAFE, spine - 2 * SPINE_SAFE, 'spine')}
			{[backRight, frontLeft].map((x) => (
				<line key={x} x1={x} x2={x} y1={0} y2={height} className={styles.spineFold} />
			))}
			<rect
				x={barcode.x}
				y={barcode.y}
				width={barcode.w}
				height={barcode.h}
				className={styles.barcode}
			/>
		</svg>
	);
}
