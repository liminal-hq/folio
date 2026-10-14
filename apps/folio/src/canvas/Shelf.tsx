// The page shelf: every spread of the book as a small proof, drawn from the model
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { Book } from '@liminal-hq/folio-protocol/generated/Book';
import type { Frame } from '@liminal-hq/folio-protocol/generated/Frame';
import { t } from '../i18n/messages';
import { spreadGeometry } from './geometry';
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
}

export function Shelf({ book, labels, selected, onSelect }: ShelfProps) {
	return (
		<section className={styles.shelf} aria-label={t('region.shelf')}>
			<ul className={styles.list} aria-label={t('region.shelf.spreads')}>
				{labels.map((label, index) => (
					<li key={book.spreads[index]!.id}>
						<button
							type="button"
							className={styles.item}
							aria-current={index === selected ? 'true' : undefined}
							onClick={() => onSelect(index)}
						>
							<Thumb book={book} index={index} />
							<span className={styles.label}>{label}</span>
						</button>
					</li>
				))}
			</ul>
		</section>
	);
}
