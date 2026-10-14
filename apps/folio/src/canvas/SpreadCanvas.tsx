// One spread drawn on its paper: pages, marks, frames and the text lines Rust positioned
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { Book } from '@liminal-hq/folio-protocol/generated/Book';
import type { Frame } from '@liminal-hq/folio-protocol/generated/Frame';
import type { FrameLayout } from '@liminal-hq/folio-protocol/generated/FrameLayout';
import type { Spread } from '@liminal-hq/folio-protocol/generated/Spread';
import type { SpreadLayout } from '@liminal-hq/folio-protocol/generated/SpreadLayout';
import { useRef, useState } from 'react';
import styles from './SpreadCanvas.module.css';
import './fonts.css';
import { PX_PER_POINT, spreadGeometry, type PlainRect } from './geometry';
import type { FrameCommand } from './frameMenu';
import { canEditText, TextEditor } from './TextEditor';
import { HANDLES, type Handle } from './snap';
import { movable, useFrameDrag } from './useFrameDrag';

export interface Marks {
	bleed: boolean;
	margins: boolean;
	columns: boolean;
	baseline: boolean;
	frameEdges: boolean;
}

export const DEFAULT_MARKS: Marks = {
	bleed: true,
	margins: true,
	columns: false,
	baseline: false,
	frameEdges: false,
};

export interface SpreadCanvasProps {
	book: Book;
	spread: Spread;
	/** `null` until Rust has laid the spread out; the frames are drawn without their text. */
	layout: SpreadLayout | null;
	/** 1 is 100%. */
	zoom: number;
	marks?: Marks;
	/** The ids of the selected frames. The canvas is read-only without `onSelect`. */
	selection?: string[];
	/** Whether dragging snaps; on unless said otherwise. */
	snap?: boolean;
	onSelect?(ids: string[]): void;
	/** Commits a frame's new box; resolves once the book has it. */
	onMove?(id: string, rect: PlainRect): Promise<void>;
	onCommand?(command: FrameCommand, ids: string[]): void;
	/** The context menu was asked for on a frame, at these viewport coordinates. */
	onFrameMenu?(frame: Frame, at: { x: number; y: number }): void;
	/** The menu was asked for on the bare page: where on screen, and where on the spread in points. */
	/** An address to draw for each picture's asset path, once it has been read. */
	images?: Record<string, string>;
	onPageMenu?(at: { x: number; y: number }, point: { x: number; y: number }): void;
	/** New words for a text frame, after it was edited in place. */
	onEditText?(id: string, text: string): void;
	/** The accessible name of the page area. */
	label: string;
}

const INK = '#1c1b20';

function Lines({ frame, layout }: { frame: Frame; layout: FrameLayout }) {
	return (
		<g transform={`translate(${frame.box.x} ${frame.box.y})`} data-frame={frame.id}>
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
								fill={INK}
								textLength={run.width}
								lengthAdjust="spacing"
								xmlSpace="preserve"
							>
								{run.text}
							</text>
							{run.hyphen && (
								<text
									x={run.x + run.width}
									y={0}
									fontFamily={`'${run.family}', serif`}
									fontSize={run.size}
									fontWeight={run.weight}
									fontStyle={run.italic ? 'italic' : 'normal'}
									fill={INK}
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
			// Fill covers the frame and crops the rest, Fit shows the whole picture inside it, and
			// Stretch pulls it to the frame's shape.
			const fit =
				frame.image.fit === 'fit'
					? 'xMidYMid meet'
					: frame.image.fit === 'stretch'
						? 'none'
						: 'xMidYMid slice';
			return (
				<g role="img" aria-label={frame.description ?? frame.placeholderLabel ?? undefined}>
					<clipPath id={clip}>
						<rect x={x} y={y} width={w} height={h} />
					</clipPath>
					<image
						href={url}
						x={x}
						y={y}
						width={w}
						height={h}
						preserveAspectRatio={fit}
						clipPath={`url(#${clip})`}
					/>
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
	if (frame.type === 'shape') {
		const fill = frame.fill ?? '#c9c4b8';
		if (frame.shape === 'ellipse') {
			return <ellipse cx={x + w / 2} cy={y + h / 2} rx={w / 2} ry={h / 2} fill={fill} />;
		}
		return (
			<rect
				x={x}
				y={y}
				width={w}
				height={h}
				rx={frame.shape === 'rounded' ? Math.min(w, h) * 0.12 : 0}
				fill={fill}
			/>
		);
	}
	return null;
}

const HANDLE_PX = 8;

/** What a frame is called to a screen reader: its description, its label, or the start of its words. */
export function frameLabel(frame: Frame): string {
	if (frame.description) return frame.description;
	if (frame.type === 'image') return frame.placeholderLabel ?? 'Picture';
	if (frame.type === 'text') {
		const words = frame.content?.inline.map((r) => r.text).join(' ') ?? '';
		return words ? words.slice(0, 60) : 'Text';
	}
	return frame.type === 'shape' ? 'Shape' : 'Group';
}

function handlePosition(box: PlainRect, handle: Handle) {
	const x = handle.includes('w') ? box.x : handle.includes('e') ? box.x + box.w : box.x + box.w / 2;
	const y = handle.includes('n') ? box.y : handle.includes('s') ? box.y + box.h : box.y + box.h / 2;
	return { x, y };
}

const CURSORS: Record<Handle, string> = {
	nw: 'nwse-resize',
	se: 'nwse-resize',
	ne: 'nesw-resize',
	sw: 'nesw-resize',
	n: 'ns-resize',
	s: 'ns-resize',
	e: 'ew-resize',
	w: 'ew-resize',
};

export function SpreadCanvas({
	book,
	spread,
	layout,
	zoom,
	marks = DEFAULT_MARKS,
	label,
	selection = [],
	snap = true,
	onSelect,
	onMove,
	onCommand,
	onFrameMenu,
	onEditText,
	onPageMenu,
	images = {},
}: SpreadCanvasProps) {
	const svg = useRef<SVGSVGElement | null>(null);
	const geometry = spreadGeometry(book, spread);
	const { bounds } = geometry;
	const scale = zoom * PX_PER_POINT;
	const drag = useFrameDrag({
		svg,
		geometry,
		frames: spread.frames,
		scale,
		selection,
		snap,
		onSelect,
		onMove,
		onCommand,
	});
	const frames = spread.frames.map((f) =>
		drag.preview[f.id] ? ({ ...f, box: drag.preview[f.id] } as Frame) : f,
	);
	const editable = onSelect !== undefined;
	const [editing, setEditing] = useState<string | null>(null);
	const editedFrame = frames.find((f) => f.id === editing);
	const startEditing = (frame: Frame) => {
		if (onEditText && frame.type === 'text' && canEditText(frame)) setEditing(frame.id);
	};
	const byFrame = new Map(layout?.frames.map((f) => [f.frame, f]));
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
							stroke="#d9534f"
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
				{frames.map((frame) => (
					<g key={frame.id}>
						<Furniture frame={frame} images={images} />
						{marks.frameEdges && (
							<rect
								data-mark="frame-edges"
								x={frame.box.x}
								y={frame.box.y}
								width={frame.box.w}
								height={frame.box.h}
								fill="none"
								stroke="#2f9e7a"
								strokeWidth={0.5}
							/>
						)}
						{byFrame.has(frame.id) && frame.id !== editing && (
							<Lines frame={frame} layout={byFrame.get(frame.id)!} />
						)}
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
								stroke="#5b7fd6"
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
										fill="#ff8a1f"
										fillOpacity={0.07}
										stroke="#ff8a1f"
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
											stroke="#d95fa0"
											strokeOpacity={0.45}
											strokeWidth={0.3}
										/>
									);
								},
							)}
					</g>
				))}
				{editable && (
					<g role="listbox" aria-multiselectable="true" aria-label={label}>
						{frames.map((frame) => (
							<rect
								key={frame.id}
								role="option"
								aria-selected={selection.includes(frame.id)}
								aria-label={frameLabel(frame)}
								data-hit={frame.id}
								x={frame.box.x}
								y={frame.box.y}
								width={frame.box.w}
								height={frame.box.h}
								fill="transparent"
								style={{ cursor: movable(frame) ? 'move' : 'default' }}
								onPointerDown={(event) => drag.onFramePointerDown(event, frame)}
								onDoubleClick={() => startEditing(frame)}
								onContextMenu={(event) => {
									event.preventDefault();
									if (!selection.includes(frame.id)) onSelect?.([frame.id]);
									onFrameMenu?.(frame, { x: event.clientX, y: event.clientY });
								}}
							/>
						))}
					</g>
				)}
				{frames
					.filter((f) => selection.includes(f.id))
					.map((frame) => (
						<g key={frame.id} data-selected={frame.id} pointerEvents="none">
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
						.map((frame) =>
							HANDLES.map((handle) => {
								const at = handlePosition(frame.box, handle);
								const size = HANDLE_PX / scale;
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
										style={{ cursor: CURSORS[handle] }}
										onPointerDown={(event) => drag.onHandlePointerDown(event, frame.id, handle)}
									/>
								);
							}),
						)}
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
			{editedFrame?.type === 'text' && (
				<TextEditor
					frame={editedFrame}
					layout={byFrame.get(editedFrame.id)}
					scale={scale}
					at={{
						x: (editedFrame.box.x - bounds.x) * scale,
						y: (editedFrame.box.y - bounds.y) * scale,
					}}
					onCommit={(text) => {
						setEditing(null);
						if (text !== null) onEditText?.(editedFrame.id, text);
						svg.current?.focus();
					}}
				/>
			)}
		</div>
	);
}
