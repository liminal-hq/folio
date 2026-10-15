// The page shelf: every spread of the book as a small proof, drawn from the model
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { Book } from '@liminal-hq/folio-protocol/generated/Book';
import type { Frame } from '@liminal-hq/folio-protocol/generated/Frame';
import { ContextMenu } from '@liminal-hq/folio-chrome/ContextMenu';
import { useState } from 'react';
import { AddPagesIcon } from '../icons/AppIcons';
import { t } from '../i18n/messages';
import { shelfMenuItems, type ShelfCommand } from './shelfMenu';
import { spreadGeometry } from './geometry';
import { BackgroundLayer } from './BackgroundLayer';
import styles from './Shelf.module.css';

const THUMB_HEIGHT = 44;

function Ruled({ frame }: { frame: Frame }) {
	const { x, y, w, h } = frame.box;
	const step = Math.max(h / 6, 8);
	const rows = Math.max(1, Math.floor(h / step) - 1);
	return (
		<g stroke="#8a867d" strokeWidth={Math.max(step / 6, 1)} strokeLinecap="round">
			{Array.from({ length: rows }, (_, i) => (
				<line
					key={i}
					x1={x}
					x2={x + (i === rows - 1 ? w * 0.6 : w)}
					y1={y + step * (i + 0.7)}
					y2={y + step * (i + 0.7)}
				/>
			))}
		</g>
	);
}

function Thumb({ book, index }: { book: Book; index: number }) {
	const spread = book.spreads[index]!;
	const { bounds } = spreadGeometry(book, spread);
	const scale = THUMB_HEIGHT / bounds.h;
	return (
		<svg
			className={styles.thumb}
			aria-hidden="true"
			width={bounds.w * scale}
			height={THUMB_HEIGHT}
			viewBox={`${bounds.x} ${bounds.y} ${bounds.w} ${bounds.h}`}
		>
			<rect
				x={bounds.x}
				y={bounds.y}
				width={bounds.w}
				height={bounds.h}
				fill="var(--folio-paper)"
			/>
			<BackgroundLayer book={book} spread={spread} thumbnail />
			{spread.frames.map((frame) =>
				frame.type === 'text' ? (
					<Ruled key={frame.id} frame={frame} />
				) : frame.type === 'group' ? null : (
					<rect
						key={frame.id}
						x={frame.box.x}
						y={frame.box.y}
						width={frame.box.w}
						height={frame.box.h}
						fill="#c9c4b8"
						fillOpacity={0.7}
					/>
				),
			)}
		</svg>
	);
}

export interface ShelfProps {
	book: Book;
	labels: string[];
	selected: number;
	onSelect(index: number): void;
	onAdd?(): void;
	onCommand?(command: ShelfCommand, index: number): void;
	/** A thumbnail was dragged to `to`. */
	onMove?(from: number, to: number): void;
}

export function Shelf({ book, labels, selected, onSelect, onAdd, onCommand, onMove }: ShelfProps) {
	const [menu, setMenu] = useState<{ index: number; x: number; y: number } | null>(null);
	const [dragging, setDragging] = useState<number | null>(null);
	const [over, setOver] = useState<number | null>(null);

	return (
		<section className={styles.shelf} aria-label={t('region.shelf')}>
			<ul className={styles.list} aria-label={t('region.shelf.spreads')}>
				{labels.map((label, index) => {
					const fixed = book.spreads[index]!.kind === 'cover';
					return (
						<li
							key={book.spreads[index]!.id}
							draggable={!fixed && onMove !== undefined}
							data-over={over === index && dragging !== null && dragging !== index}
							onDragStart={(event) => {
								setDragging(index);
								event.dataTransfer?.setData('text/plain', String(index));
							}}
							onDragOver={(event) => {
								if (dragging === null || fixed) return;
								event.preventDefault();
								setOver(index);
							}}
							onDrop={(event) => {
								event.preventDefault();
								if (dragging !== null && !fixed && dragging !== index) onMove?.(dragging, index);
								setDragging(null);
								setOver(null);
							}}
							onDragEnd={() => {
								setDragging(null);
								setOver(null);
							}}
						>
							<button
								type="button"
								className={styles.item}
								aria-current={index === selected ? 'true' : undefined}
								onClick={() => onSelect(index)}
								onContextMenu={(event) => {
									event.preventDefault();
									onSelect(index);
									setMenu({ index, x: event.clientX, y: event.clientY });
								}}
							>
								<Thumb book={book} index={index} />
								<span className={styles.label}>{label}</span>
							</button>
						</li>
					);
				})}
				{onAdd && (
					<li>
						<button type="button" className={styles.add} onClick={onAdd}>
							<AddPagesIcon />
							{t('shelf.add')}
						</button>
					</li>
				)}
			</ul>
			{menu && (
				<ContextMenu
					items={shelfMenuItems(book, menu.index)}
					position={menu}
					ariaLabel={t('shelf.menu')}
					onSelect={(item) => {
						const index = menu.index;
						setMenu(null);
						onCommand?.(item.id as ShelfCommand, index);
					}}
					onClose={() => setMenu(null)}
				/>
			)}
		</section>
	);
}
