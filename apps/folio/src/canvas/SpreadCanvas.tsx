// One spread drawn on its paper: pages, marks, frames and the text lines Rust positioned
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { Book } from '@liminal-hq/folio-protocol/generated/Book';
import type { Frame } from '@liminal-hq/folio-protocol/generated/Frame';
import type { FrameLayout } from '@liminal-hq/folio-protocol/generated/FrameLayout';
import type { Spread } from '@liminal-hq/folio-protocol/generated/Spread';
import type { SpreadLayout } from '@liminal-hq/folio-protocol/generated/SpreadLayout';
import styles from './SpreadCanvas.module.css';
import './fonts.css';
import { PX_PER_POINT, spreadGeometry } from './geometry';

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

function Furniture({ frame }: { frame: Frame }) {
	const { x, y, w, h } = frame.box;
	if (frame.type === 'image') {
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

export function SpreadCanvas({
	book,
	spread,
	layout,
	zoom,
	marks = DEFAULT_MARKS,
	label,
}: SpreadCanvasProps) {
	const geometry = spreadGeometry(book, spread);
	const { bounds } = geometry;
	const scale = zoom * PX_PER_POINT;
	const byFrame = new Map(layout?.frames.map((f) => [f.frame, f]));
	const grid = book.grid;

	return (
		<svg
			className={styles.spread}
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
					<line x1="0" y1="0" x2="0" y2="6" stroke="#d9534f" strokeOpacity="0.35" strokeWidth="1" />
				</pattern>
			</defs>

			<rect
				x={bounds.x}
				y={bounds.y}
				width={bounds.w}
				height={bounds.h}
				fill="var(--folio-paper)"
				className={styles.paper}
			/>
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

			{spread.frames.map((frame) => (
				<g key={frame.id}>
					<Furniture frame={frame} />
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
					{byFrame.has(frame.id) && <Lines frame={frame} layout={byFrame.get(frame.id)!} />}
				</g>
			))}
		</svg>
	);
}
