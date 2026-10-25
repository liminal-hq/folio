// One spread drawn on its paper: pages, marks, frames and the text lines Rust positioned
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { Book } from '@liminal-hq/folio-protocol/generated/Book';
import type { Frame } from '@liminal-hq/folio-protocol/generated/Frame';
import type { TableEdit } from '@liminal-hq/folio-protocol/generated/TableEdit';
import type { FrameLayout } from '@liminal-hq/folio-protocol/generated/FrameLayout';
import type { Spread } from '@liminal-hq/folio-protocol/generated/Spread';
import type { SpreadLayout } from '@liminal-hq/folio-protocol/generated/SpreadLayout';
import { useEffect, useRef, useState } from 'react';
import { FocusedImage } from './FocusedImage';
import { BackgroundLayer } from './BackgroundLayer';
import { TableArt } from './TableArt';
import { cellAt, cellId, cellText, nextCell, parseCellId } from './tableCells';
import { turned } from './turned';
import { turnedCursor, turnedHandle, turnPoint } from './turnedHandles';
import { ShapeArt } from './ShapeArt';
import styles from './SpreadCanvas.module.css';
import './fonts.css';
import { PX_PER_POINT, spreadGeometry, type PlainRect } from './geometry';
import type { FrameCommand } from './frameMenu';
import { canEditText, TextEditor, type Speller } from './TextEditor';
import { boxOf, linesOf } from '../story/rangeEdit';
import { DEFAULT_MARK_COLOURS, DEFAULT_MARKS, type MarkColours, type Marks } from './marksModel';
import { masterGuidesOn } from './masterGuides';
import { masterFurniture } from './masterItems';
import { HANDLES } from './snap';
import type { GuideAxis } from './guides';
import type { GuideDragState } from './useGuideDrag';
import { movable, useFrameDrag, type FrameBoxMove } from './useFrameDrag';

export interface SpreadCanvasProps {
	book: Book;
	spread: Spread;
	/** `null` until Rust has laid the spread out; the frames are drawn without their text. */
	layout: SpreadLayout | null;
	/** 1 is 100%. */
	zoom: number;
	marks?: Marks;
	/** The colour of each mark. */
	colours?: MarkColours;
	/** The ids of the selected frames. The canvas is read-only without `onSelect`. */
	selection?: string[];
	/** Whether dragging snaps; on unless said otherwise. */
	snap?: boolean;
	onSelect?(ids: string[]): void;
	/** Commits a frame's new box; resolves once the book has it. */
	onMove?(moves: FrameBoxMove[]): Promise<void>;
	/** Commits a frame's new turn, in degrees clockwise; resolves once the book has it. */
	onTurn?(id: string, degrees: number): Promise<void>;
	onCommand?(command: FrameCommand, ids: string[]): void;
	/** The context menu was asked for on a frame, at these viewport coordinates. */
	onFrameMenu?(frame: Frame, at: { x: number; y: number }): void;
	/** A press on a guide that is not locked, to move it. */
	onGuidePointerDown?(axis: GuideAxis, index: number, event: { button: number }): void;
	/** The context menu was asked for on a guide, at these viewport coordinates. */
	onGuideMenu?(index: number, at: { x: number; y: number }): void;
	/** A guide being pulled out or moved, drawn where it would land. */
	guideDrag?: GuideDragState | null;
	/** The menu was asked for on the bare page: where on screen, and where on the spread in points. */
	/** An address to draw for each picture's asset path, once it has been read. */
	images?: Record<string, string>;
	onPageMenu?(at: { x: number; y: number }, point: { x: number; y: number }): void;
	/** A frame was double-clicked: `true` when something took it, and it is not edited in place. */
	onActivate?(id: string): boolean;
	/** A line of grey words drawn in the middle of a box, for what can be done there; never printed. */
	hint?: { box: { x: number; y: number; w: number; h: number }; text: string };
	/** New words for a text frame, after it was edited in place. */
	onEditText?(id: string, text: string): void;
	/** A change to a table, from editing one of its cells. */
	onEditTable?(id: string, edit: TableEdit): void;
	/** A frame in a story was opened for editing: the story's words are edited as a whole, apart from the page. */
	onEditStory?(flow: string, paragraph?: number): void;
	/**
	 * The paragraphs of a story from `from` on, as they were (`before`) and as the person has them
	 * now (`after`), after the words of a frame of it were edited in place.
	 */
	onEditParagraphs?(flow: string, from: number, before: string[], after: string[]): void;
	/** Underlines misspelt words in the editor, when spelling is on. */
	speller?: Speller;
	/** The cell being edited has changed: its row and column, or `null` when none is. */
	onEditingCell?(cell: { row: number; col: number } | null): void;
	/** The accessible name of the page area. */
	label: string;
}

const INK = '#1c1b20';

/** A wavy line from `x1` to `x2` at height `y`, with waves of `wave` points up and down. */
export function squiggle(x1: number, x2: number, y: number, wave: number): string {
	const step = wave * 2;
	let d = `M${x1} ${y}`;
	let up = true;
	for (let x = x1; x < x2; x += step) {
		const next = Math.min(x + step, x2);
		d += `L${next} ${up ? y - wave : y + wave}`;
		up = !up;
	}
	return d;
}

function Lines({
	id,
	at,
	layout,
}: {
	id: string;
	at: { x: number; y: number };
	layout: FrameLayout;
}) {
	return (
		<g transform={`translate(${at.x} ${at.y})`} data-frame={id}>
			{layout.lines.map((line, index) => (
				<g key={index} transform={`translate(${line.x} ${line.baseline})`}>
					{line.runs.map((run, i) => (
						<g key={i}>
							<text
								x={run.x}
								y={0}
								fontFamily={`'${run.family}', serif`}
								fontSize={run.size}
								fontWeight={run.weight}
								fontStyle={run.italic ? 'italic' : 'normal'}
								fill={run.colour ?? INK}
								textLength={run.width}
								lengthAdjust="spacing"
								xmlSpace="preserve"
							>
								{run.text}
							</text>
							{run.miss?.map(([from, to]) => {
								// The marked part of a word, along the word as it is drawn.
								const x1 = run.x + (run.width * from) / (run.text.length || 1);
								const x2 = run.x + (run.width * to) / (run.text.length || 1);
								return (
									<path
										key={`${from}-${to}`}
										data-miss-mark=""
										d={squiggle(x1, x2, run.size * 0.16, run.size * 0.05)}
										fill="none"
										stroke="#d64545"
										strokeWidth={Math.max(0.4, run.size * 0.045)}
										strokeLinejoin="round"
									/>
								);
							})}
							{run.hyphen && (
								<text
									x={run.x + run.width}
									y={0}
									fontFamily={`'${run.family}', serif`}
									fontSize={run.size}
									fontWeight={run.weight}
									fontStyle={run.italic ? 'italic' : 'normal'}
									fill={run.colour ?? INK}
								>
									-
								</text>
							)}
						</g>
					))}
				</g>
			))}
		</g>
	);
}

function Furniture({ frame, images }: { frame: Frame; images: Record<string, string> }) {
	const { x, y, w, h } = frame.box;
	if (frame.type === 'image') {
		const url = frame.image.asset ? images[frame.image.asset] : undefined;
		if (url) {
			const clip = `folio-clip-${frame.id}`;
			// Fill covers the frame and crops the rest around the focus point, Fit shows the whole
			// picture inside it, and Stretch pulls it to the frame's shape.
			const fit =
				frame.image.fit === 'fit' ? 'fit' : frame.image.fit === 'stretch' ? 'stretch' : 'fill';
			return (
				<g role="img" aria-label={frame.description ?? frame.placeholderLabel ?? undefined}>
					<clipPath id={clip}>
						<rect x={x} y={y} width={w} height={h} />
					</clipPath>
					<g clipPath={`url(#${clip})`}>
						<FocusedImage href={url} box={{ x, y, w, h }} fit={fit} focus={frame.image.focus} />
					</g>
				</g>
			);
		}
		const label = frame.placeholderLabel ?? frame.description ?? '';
		return (
			<g role="img" aria-label={label || undefined}>
				<rect x={x} y={y} width={w} height={h} fill="#c9c4b8" fillOpacity={0.55} />
				{label && (
					<text
						x={x + w / 2}
						y={y + h / 2}
						textAnchor="middle"
						fontFamily="Inter, sans-serif"
						fontSize={10}
						fill="#5d5a54"
					>
						{label}
					</text>
				)}
			</g>
		);
	}
	if (frame.type === 'shape') return <ShapeArt frame={frame} />;
	if (frame.type === 'table') return <TableArt frame={frame} />;
	return null;
}

const HANDLE_PX = 8;
/** How far the turn handle stands out from the top edge, in screen pixels. */
const TURN_HANDLE_PX = 22;
/** The least a frame's click area is across, in points, so a line with no height can still be picked. */
const MIN_HIT = 10;

function hitBox(box: PlainRect): { x: number; y: number; width: number; height: number } {
	const width = Math.max(box.w, MIN_HIT);
	const height = Math.max(box.h, MIN_HIT);
	return {
		x: box.x - (width - box.w) / 2,
		y: box.y - (height - box.h) / 2,
		width,
		height,
	};
}

/** What a frame is called to a screen reader: its description, its label, or the start of its words. */
export function frameLabel(frame: Frame): string {
	if (frame.description) return frame.description;
	if (frame.type === 'image') return frame.placeholderLabel ?? 'Picture';
	if (frame.type === 'text') {
		const words = frame.content?.inline.map((r) => r.text).join(' ') ?? '';
		return words ? words.slice(0, 60) : 'Text';
	}
	if (frame.type === 'table') return 'Table';
	return frame.type === 'shape' ? 'Shape' : 'Group';
}

export function SpreadCanvas({
	book,
	spread,
	layout,
	zoom,
	marks = DEFAULT_MARKS,
	colours = DEFAULT_MARK_COLOURS,
	label,
	selection = [],
	snap = true,
	onSelect,
	onMove,
	onTurn,
	onCommand,
	onFrameMenu,
	onGuidePointerDown,
	onGuideMenu,
	guideDrag = null,
	onActivate,
	hint,
	onEditText,
	onEditStory,
	onEditParagraphs,
	onEditTable,
	onEditingCell,
	speller,
	onPageMenu,
	images = {},
}: SpreadCanvasProps) {
	const svg = useRef<SVGSVGElement | null>(null);
	const geometry = spreadGeometry(book, spread);
	const { bounds } = geometry;
	const masterGuides = masterGuidesOn(book, spread);
	const scale = zoom * PX_PER_POINT;
	const drag = useFrameDrag({
		svg,
		geometry,
		frames: spread.frames,
		scale,
		selection,
		snap,
		guides: marks.guides
			? [
					...spread.guides.map((g) => ({ axis: g.axis, pos: g.pos })),
					...masterGuides.map((g) => ({ axis: g.axis, pos: g.pos })),
				]
			: [],
		onSelect,
		onMove,
		onTurn,
		onCommand,
	});
	const frames = spread.frames.map((f) => {
		const box = drag.preview[f.id];
		const turn = drag.turning[f.id];
		return box || turn !== undefined
			? ({ ...f, box: box ?? f.box, rotation: turn ?? f.rotation } as Frame)
			: f;
	});
	const editable = onSelect !== undefined;
	const [editing, setEditing] = useState<string | null>(null);
	const editingCell = editing === null ? null : parseCellId(editing);
	const editedFrame = frames.find((f) => f.id === (editingCell?.frame ?? editing));
	if (editing !== null && !editedFrame) setEditing(null);
	// Which cell is being edited, for the ribbon's row and column buttons.
	useEffect(() => {
		onEditingCell?.(editingCell && { row: editingCell.row, col: editingCell.col });
		// eslint-disable-next-line react-hooks/exhaustive-deps
	}, [editing]);
	/** Where on the page a pointer event is, in points. */
	const pagePoint = (event: { clientX: number; clientY: number }) => {
		const box = svg.current?.getBoundingClientRect();
		return {
			x: bounds.x + (event.clientX - (box?.left ?? 0)) / scale,
			y: bounds.y + (event.clientY - (box?.top ?? 0)) / scale,
		};
	};
	/** A paragraph of a story being edited over the frame it is in. */
	const [storyEdit, setStoryEdit] = useState<{ frame: string; flow: string; from: number } | null>(
		null,
	);
	/** The paragraphs of the stretch being edited as the book was last told them. */
	const lastLines = useRef<string[]>([]);
	const startEditing = (frame: Frame, at?: { x: number; y: number }) => {
		if (onActivate?.(frame.id)) return;
		if (frame.type === 'table') {
			if (!onEditTable || frame.locked || frame.fromMaster) return;
			const cell = at ? cellAt(frame, at) : { frame: frame.id, row: 0, col: 0 };
			if (cell && cellText(frame, cell.row, cell.col) !== null) setEditing(cellId(cell));
			return;
		}
		if (frame.type === 'text' && frame.flow && !frame.locked && !frame.fromMaster) {
			const laid = byFrame.get(frame.id);
			// The paragraph under the click: the line it fell on, else the nearest one, else the first
			// the frame shows.
			const y = at ? at.y - frame.box.y : 0;
			const near = at
				? laid?.lines.reduce<(typeof laid.lines)[number] | undefined>((best, line) => {
						const gap = (l: typeof line) => Math.abs(y - (l.top + l.height / 2));
						return best === undefined || gap(line) < gap(best) ? line : best;
					}, undefined)
				: undefined;
			const para = near?.paragraph ?? laid?.firstParagraph ?? undefined;
			// The whole of every paragraph the frame shows is edited together, one to a line.
			const shown = laid?.lines.flatMap((l) => (l.paragraph == null ? [] : [l.paragraph])) ?? [];
			const story = (book.flows ?? []).find((f) => f.id === frame.flow?.id);
			if (onEditParagraphs && story && shown.length > 0) {
				const from = Math.min(...shown);
				const count = Math.max(...shown) - from + 1;
				lastLines.current = boxOf(story.paragraphs, from, count).split('\n');
				setStoryEdit({ frame: frame.id, flow: frame.flow.id, from });
				return;
			}
			if (onEditStory) onEditStory(frame.flow.id, para ?? undefined);
			return;
		}
		if (onEditText && (frame.type === 'text' || frame.type === 'shape') && canEditText(frame))
			setEditing(frame.id);
	};
	const byFrame = new Map(layout?.frames.map((f) => [f.frame, f]));
	const furniture = masterFurniture(book, spread);
	const grid = book.grid;

	return (
		<div className={styles.frame}>
			<svg
				ref={svg}
				className={styles.spread}
				tabIndex={editable ? 0 : undefined}
				onKeyDown={
					editable
						? (event) => {
								const only =
									selection.length === 1 ? frames.find((f) => f.id === selection[0]) : undefined;
								if (event.key === 'Enter' && only) {
									event.preventDefault();
									startEditing(only);
								} else drag.onKeyDown(event);
							}
						: undefined
				}
				onPointerMove={editable ? drag.onPointerMove : undefined}
				onPointerUp={editable ? drag.onPointerUp : undefined}
				onPointerCancel={editable ? drag.onPointerCancel : undefined}
				onLostPointerCapture={editable ? drag.onPointerCancel : undefined}
				role="group"
				aria-label={label}
				width={bounds.w * scale}
				height={bounds.h * scale}
				viewBox={`${bounds.x} ${bounds.y} ${bounds.w} ${bounds.h}`}
			>
				<defs>
					<pattern
						id="folio-bleed-hatch"
						width="6"
						height="6"
						patternUnits="userSpaceOnUse"
						patternTransform="rotate(45)"
					>
						<line
							x1="0"
							y1="0"
							x2="0"
							y2="6"
							stroke={colours.bleed}
							strokeOpacity="0.35"
							strokeWidth="1"
						/>
					</pattern>
				</defs>

				<rect
					x={bounds.x}
					y={bounds.y}
					width={bounds.w}
					height={bounds.h}
					fill="var(--folio-paper)"
					className={styles.paper}
					onPointerDown={editable ? drag.onBackgroundPointerDown : undefined}
					onContextMenu={(event) => {
						event.preventDefault();
						onSelect?.([]);
						const box = svg.current?.getBoundingClientRect();
						onPageMenu?.(
							{ x: event.clientX, y: event.clientY },
							{
								x: bounds.x + (event.clientX - (box?.left ?? 0)) / scale,
								y: bounds.y + (event.clientY - (box?.top ?? 0)) / scale,
							},
						);
					}}
				/>
				<BackgroundLayer book={book} spread={spread} images={images} />
				{/* The master pages' own text sits behind the page's frames. */}
				{layout?.frames.map((laid) =>
					// A master shape's words are drawn over the shape, below.
					laid.origin && !furniture.some((f) => f.id === laid.frame) ? (
						<Lines key={laid.frame} id={laid.frame} at={laid.origin} layout={laid} />
					) : null,
				)}
				{furniture.map((frame) => {
					const words = layout?.frames.find((l) => l.frame === frame.id && l.origin);
					return (
						<g
							key={frame.id}
							data-master-item={frame.id}
							pointerEvents="none"
							transform={turned(frame)}
						>
							<Furniture frame={frame} images={images} />
							{words?.origin && <Lines id={words.frame} at={words.origin} layout={words} />}
						</g>
					);
				})}
				{frames.map((frame) => (
					<g key={frame.id} transform={turned(frame)}>
						<Furniture frame={frame} images={images} />
						{marks.frameEdges && (
							<rect
								data-mark="frame-edges"
								x={frame.box.x}
								y={frame.box.y}
								width={frame.box.w}
								height={frame.box.h}
								fill="none"
								stroke={colours.frameEdges}
								strokeWidth={0.5}
							/>
						)}
						{byFrame.has(frame.id) && frame.id !== editing && (
							<Lines id={frame.id} at={frame.box} layout={byFrame.get(frame.id)!} />
						)}
						{frame.type === 'table' &&
							layout?.frames
								.filter((cell) => cell.frame.startsWith(`${frame.id}#`) && cell.frame !== editing)
								.map((cell) => (
									<Lines key={cell.frame} id={cell.frame} at={frame.box} layout={cell} />
								))}
					</g>
				))}
				{marks.bleed && (
					<path
						data-mark="bleed"
						fillRule="evenodd"
						fill="url(#folio-bleed-hatch)"
						d={`M${bounds.x} ${bounds.y}h${bounds.w}v${bounds.h}h${-bounds.w}z M0 0v${geometry.height}h${geometry.width}v${-geometry.height}z`}
					/>
				)}

				{geometry.pages.map((page, index) => (
					<g key={index} data-page={index}>
						<rect
							x={page.trim.x}
							y={page.trim.y}
							width={page.trim.w}
							height={page.trim.h}
							fill="none"
							stroke={INK}
							strokeOpacity={0.25}
							strokeWidth={0.5}
						/>
						{marks.margins && (
							<rect
								data-mark="margins"
								x={page.safe.x}
								y={page.safe.y}
								width={page.safe.w}
								height={page.safe.h}
								fill="none"
								stroke={colours.margins}
								strokeOpacity={0.7}
								strokeWidth={0.5}
								strokeDasharray="3 2"
							/>
						)}
						{marks.columns &&
							grid.columns > 1 &&
							Array.from({ length: grid.columns }, (_, c) => {
								const width = (page.safe.w - grid.gutter * (grid.columns - 1)) / grid.columns;
								return (
									<rect
										key={c}
										data-mark="columns"
										x={page.safe.x + c * (width + grid.gutter)}
										y={page.safe.y}
										width={width}
										height={page.safe.h}
										fill={colours.columns}
										fillOpacity={0.07}
										stroke={colours.columns}
										strokeOpacity={0.4}
										strokeWidth={0.4}
									/>
								);
							})}
						{marks.baseline &&
							grid.baseline &&
							grid.baseline.increment > 0 &&
							Array.from(
								{
									length: Math.max(
										0,
										Math.floor((page.safe.h - grid.baseline.offset) / grid.baseline.increment) + 1,
									),
								},
								(_, row) => {
									const yy = page.safe.y + grid.baseline!.offset + row * grid.baseline!.increment;
									return (
										<line
											key={row}
											data-mark="baseline"
											x1={page.safe.x}
											x2={page.safe.x + page.safe.w}
											y1={yy}
											y2={yy}
											stroke={colours.baseline}
											strokeOpacity={0.45}
											strokeWidth={0.3}
										/>
									);
								},
							)}
					</g>
				))}
				{hint && (
					<text
						data-testid="canvas-hint"
						x={hint.box.x + hint.box.w / 2}
						y={hint.box.y + hint.box.h / 2}
						textAnchor="middle"
						dominantBaseline="middle"
						fontSize={6.5}
						fill="#8a8f98"
						pointerEvents="none"
					>
						{hint.text}
					</text>
				)}
				{editable && (
					<g role="listbox" aria-multiselectable="true" aria-label={label}>
						{frames.map((frame) => (
							<rect
								key={frame.id}
								role="option"
								aria-selected={selection.includes(frame.id)}
								aria-label={frameLabel(frame)}
								data-hit={frame.id}
								{...hitBox(frame.box)}
								transform={turned(frame)}
								fill="transparent"
								style={{ cursor: movable(frame) ? 'move' : 'default' }}
								onPointerDown={(event) => drag.onFramePointerDown(event, frame)}
								onDoubleClick={(event) => startEditing(frame, pagePoint(event))}
								onContextMenu={(event) => {
									event.preventDefault();
									if (!selection.includes(frame.id)) onSelect?.([frame.id]);
									onFrameMenu?.(frame, { x: event.clientX, y: event.clientY });
								}}
							/>
						))}
					</g>
				)}
				{marks.guides &&
					masterGuides.map((guide, i) => {
						const vertical = guide.axis === 'x';
						const page = geometry.pages[guide.page]!.trim;
						return (
							<line
								key={`master-${i}`}
								data-master-guide={guide.master}
								x1={vertical ? guide.pos : page.x}
								x2={vertical ? guide.pos : page.x + page.w}
								y1={vertical ? bounds.y : guide.pos}
								y2={vertical ? bounds.y + bounds.h : guide.pos}
								stroke={colours.margins}
								strokeWidth={1 / scale}
								pointerEvents="none"
							/>
						);
					})}
				{marks.guides &&
					spread.guides.map((guide, index) => {
						if (guideDrag?.index === index) return null;
						const vertical = guide.axis === 'x';
						const line = {
							x1: vertical ? guide.pos : bounds.x,
							x2: vertical ? guide.pos : bounds.x + bounds.w,
							y1: vertical ? bounds.y : guide.pos,
							y2: vertical ? bounds.y + bounds.h : guide.pos,
						};
						return (
							<g key={index} data-guide={index} data-locked={guide.locked || undefined}>
								<line
									{...line}
									stroke={colours.guides}
									strokeWidth={1 / scale}
									strokeDasharray={guide.locked ? `${4 / scale} ${3 / scale}` : undefined}
									pointerEvents="none"
								/>
								{editable && (
									<line
										{...line}
										data-guide-hit={index}
										stroke="transparent"
										strokeWidth={8 / scale}
										style={{
											cursor: guide.locked ? 'default' : vertical ? 'col-resize' : 'row-resize',
										}}
										onPointerDown={(event) => {
											event.stopPropagation();
											if (!guide.locked) onGuidePointerDown?.(guide.axis, index, event);
										}}
										onContextMenu={(event) => {
											event.preventDefault();
											event.stopPropagation();
											onGuideMenu?.(index, { x: event.clientX, y: event.clientY });
										}}
									/>
								)}
							</g>
						);
					})}
				{guideDrag && (
					<line
						data-guide-drag={guideDrag.axis}
						x1={guideDrag.axis === 'x' ? guideDrag.pos : bounds.x}
						x2={guideDrag.axis === 'x' ? guideDrag.pos : bounds.x + bounds.w}
						y1={guideDrag.axis === 'y' ? guideDrag.pos : bounds.y}
						y2={guideDrag.axis === 'y' ? guideDrag.pos : bounds.y + bounds.h}
						stroke={guideDrag.remove ? 'var(--folio-danger)' : colours.guides}
						strokeWidth={1 / scale}
						pointerEvents="none"
					/>
				)}
				{frames
					.filter((f) => selection.includes(f.id))
					.map((frame) => (
						<g
							key={frame.id}
							data-selected={frame.id}
							pointerEvents="none"
							transform={turned(frame)}
						>
							<rect
								x={frame.box.x}
								y={frame.box.y}
								width={frame.box.w}
								height={frame.box.h}
								fill="none"
								stroke="var(--folio-accent)"
								strokeWidth={1.5 / scale}
								strokeDasharray={frame.locked ? `${4 / scale} ${3 / scale}` : undefined}
							/>
						</g>
					))}
				{selection.length === 1 &&
					frames
						.filter((f) => f.id === selection[0] && movable(f))
						.map((frame) => {
							const size = HANDLE_PX / scale;
							const top = turnedHandle(frame.box, frame.rotation, 'n');
							const centre = {
								x: frame.box.x + frame.box.w / 2,
								y: frame.box.y + frame.box.h / 2,
							};
							// The turn handle stands out from the top edge, along the frame's own up.
							const reach = TURN_HANDLE_PX / scale;
							const out = turnPoint(
								{ x: centre.x, y: frame.box.y - reach },
								centre,
								frame.rotation,
							);
							return (
								<g key={frame.id}>
									<line
										x1={top.x}
										y1={top.y}
										x2={out.x}
										y2={out.y}
										stroke="var(--folio-accent)"
										strokeWidth={1 / scale}
										pointerEvents="none"
									/>
									<circle
										data-handle="turn"
										cx={out.x}
										cy={out.y}
										r={size * 0.7}
										fill="#fff"
										stroke="var(--folio-accent)"
										strokeWidth={1.5 / scale}
										style={{ cursor: 'grab' }}
										onPointerDown={(event) => drag.onTurnPointerDown(event, frame.id)}
									/>
									{HANDLES.map((handle) => {
										const at = turnedHandle(frame.box, frame.rotation, handle);
										return (
											<rect
												key={handle}
												data-handle={handle}
												x={at.x - size / 2}
												y={at.y - size / 2}
												width={size}
												height={size}
												fill="#fff"
												stroke="var(--folio-accent)"
												strokeWidth={1.5 / scale}
												style={{ cursor: turnedCursor(handle, frame.rotation) }}
												onPointerDown={(event) => drag.onHandlePointerDown(event, frame.id, handle)}
											/>
										);
									})}
								</g>
							);
						})}
				{drag.lines.map((line, i) => (
					<line
						key={i}
						data-snap={line.axis}
						x1={line.axis === 'x' ? line.pos : bounds.x}
						x2={line.axis === 'x' ? line.pos : bounds.x + bounds.w}
						y1={line.axis === 'y' ? line.pos : bounds.y}
						y2={line.axis === 'y' ? line.pos : bounds.y + bounds.h}
						stroke="#e0368f"
						strokeWidth={1 / scale}
						pointerEvents="none"
					/>
				))}
			</svg>
			{(editedFrame?.type === 'text' || editedFrame?.type === 'shape') && (
				<TextEditor
					key={editing}
					speller={speller}
					frame={editedFrame}
					frameId={editedFrame.id}
					layout={byFrame.get(editedFrame.id)}
					scale={scale}
					rotation={editedFrame.rotation}
					at={{
						x: (editedFrame.box.x - bounds.x) * scale,
						y: (editedFrame.box.y - bounds.y) * scale,
					}}
					onCheckpoint={(text) => onEditText?.(editedFrame.id, text)}
					onCommit={(text) => {
						setEditing(null);
						if (text !== null) onEditText?.(editedFrame.id, text);
						svg.current?.focus();
					}}
				/>
			)}
			{storyEdit &&
				(() => {
					const host = spread.frames.find((f) => f.id === storyEdit.frame);
					if (host?.type !== 'text') return null;
					const keep = (text: string) => {
						const after = linesOf(text);
						const before = lastLines.current;
						if (after.join('\n') === before.join('\n')) return;
						onEditParagraphs?.(storyEdit.flow, storyEdit.from, before, after);
						lastLines.current = after;
					};
					return (
						<TextEditor
							key={`${storyEdit.flow}:${storyEdit.from}`}
							speller={speller}
							frame={{
								type: 'text',
								box: host.box,
								content: {
									inline: [
										{ text: lastLines.current.join('\n'), style: null, bold: false, italic: false },
									],
								},
							}}
							layout={byFrame.get(host.id)}
							scale={scale}
							rotation={host.rotation}
							at={{
								x: (host.box.x - bounds.x) * scale,
								y: (host.box.y - bounds.y) * scale,
							}}
							onCheckpoint={keep}
							onCommit={(text) => {
								setStoryEdit(null);
								if (text !== null) keep(text);
								svg.current?.focus();
							}}
						/>
					);
				})()}
			{editedFrame?.type === 'table' &&
				editingCell &&
				editingCell.row < editedFrame.rows.length &&
				editingCell.col < editedFrame.cols.length && (
					<TextEditor
						key={editing}
						speller={speller}
						frame={{
							type: 'cell',
							box: editedFrame.box,
							content: editedFrame.rows[editingCell.row]![editingCell.col]!.content,
						}}
						layout={byFrame.get(editing!)}
						scale={scale}
						rotation={editedFrame.rotation}
						at={{
							x: (editedFrame.box.x - bounds.x) * scale,
							y: (editedFrame.box.y - bounds.y) * scale,
						}}
						onCheckpoint={(text) =>
							onEditTable?.(editedFrame.id, {
								type: 'setCell',
								row: editingCell.row,
								col: editingCell.col,
								text,
							})
						}
						onCommit={(text) => {
							setEditing(null);
							if (text !== null) {
								onEditTable?.(editedFrame.id, {
									type: 'setCell',
									row: editingCell.row,
									col: editingCell.col,
									text,
								});
							}
							svg.current?.focus();
						}}
						onTab={(back) => {
							const next = nextCell(editedFrame, editingCell, back);
							if (next) setEditing(cellId(next));
							else if (!back) {
								// Tab out of the last cell makes a row for what comes next.
								onEditTable?.(editedFrame.id, { type: 'addRow', at: editedFrame.rows.length });
								setEditing(cellId({ frame: editedFrame.id, row: editedFrame.rows.length, col: 0 }));
							}
						}}
					/>
				)}
		</div>
	);
}
