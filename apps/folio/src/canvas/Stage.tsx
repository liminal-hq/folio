// The stage: a spread on the table with its title and zoom, fitted to the room there is
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { Book } from '@liminal-hq/folio-protocol/generated/Book';
import type { Spread } from '@liminal-hq/folio-protocol/generated/Spread';
import type { SpreadLayout } from '@liminal-hq/folio-protocol/generated/SpreadLayout';
import { ContextMenu } from '@liminal-hq/folio-chrome/ContextMenu';
import { useState } from 'react';
import type { MutableRefObject } from 'react';
import { t } from '../i18n/messages';
import { frameMenuItems, type FrameCommand } from './frameMenu';
import { pageMenuItems, type PageCommand } from './pageMenu';
import { fitZoom, spreadGeometry } from './geometry';
import { MarksMenu } from './MarksMenu';
import {
	DEFAULT_MARK_COLOURS,
	DEFAULT_MARKS,
	type MarkColours,
	type MarkKey,
	type Marks,
} from './marksModel';
import { SpreadCanvas } from './SpreadCanvas';
import styles from './Stage.module.css';
import { useElementSize } from './useElementSize';
import type { FrameBoxMove } from './useFrameDrag';
import { useStageZoom, type StageZoom } from './useStageZoom';

export type ZoomApi = Pick<StageZoom, 'zoomIn' | 'zoomOut' | 'fit'>;

export interface StageProps {
	book: Book;
	spread: Spread;
	/** What the shelf calls this spread: "Cover", "Pages 2–3". */
	label: string;
	layout: SpreadLayout | null;
	marks?: Marks;
	colours?: MarkColours;
	onMarks?(next: Partial<Marks>): void;
	onMarkColour?(key: MarkKey, index: number): void;
	snap?: boolean;
	/** Filled in with the stage's zoom controls, for the ribbon and the keys to use. */
	zoomApi?: MutableRefObject<ZoomApi | null>;
	images?: Record<string, string>;
	selection?: string[];
	onSelect?(ids: string[]): void;
	onMove?(moves: FrameBoxMove[]): Promise<void>;
	onCommand?(command: FrameCommand, ids: string[]): void;
	onEditText?(id: string, text: string): void;
	/** A choice from the page menu; `point` is where it was asked for, in points on the spread. */
	onPageCommand?(command: PageCommand, point: { x: number; y: number }): void;
}

export function Stage({
	book,
	spread,
	label,
	layout,
	marks,
	colours,
	onMarks,
	onMarkColour,
	snap,
	zoomApi,
	images,
	selection,
	onSelect,
	onMove,
	onCommand,
	onEditText,
	onPageCommand,
}: StageProps) {
	const [ref, size] = useElementSize<HTMLDivElement>();
	const geometry = spreadGeometry(book, spread);
	const zoom = useStageZoom(fitZoom(geometry.bounds, size));
	if (zoomApi) zoomApi.current = zoom;
	const percent = Math.round(zoom.zoom * 100);
	const [menu, setMenu] = useState<{ x: number; y: number } | null>(null);
	const [pageMenu, setPageMenu] = useState<{
		at: { x: number; y: number };
		point: { x: number; y: number };
	} | null>(null);
	const chosen = spread.frames.filter((f) => selection?.includes(f.id));

	return (
		<>
			<div className={styles.header}>
				<h1 className={styles.title}>{t('stage.spread', { label })}</h1>
				<div className={styles.tools}>
					{onMarks && onMarkColour && (
						<MarksMenu
							marks={marks ?? DEFAULT_MARKS}
							colours={colours ?? DEFAULT_MARK_COLOURS}
							onMarks={onMarks}
							onColour={onMarkColour}
						/>
					)}
					<div className={styles.zoom} role="group" aria-label={t('stage.zoom.group')}>
						<button type="button" onClick={zoom.zoomOut} aria-label={t('stage.zoom.out')}>
							−
						</button>
						<output aria-label={t('stage.zoom.level', { percent })}>{percent}%</output>
						<button type="button" onClick={zoom.zoomIn} aria-label={t('stage.zoom.in')}>
							+
						</button>
						<button type="button" onClick={zoom.fit} aria-pressed={zoom.fitting}>
							{t('stage.zoom.fit')}
						</button>
					</div>
				</div>
			</div>
			<div
				ref={ref}
				className={styles.table}
				onPointerDown={(event) => {
					// A press on the bare table, not on the page, lets go of the selection.
					if (!(event.target as Element).closest('svg')) onSelect?.([]);
				}}
			>
				<div className={styles.sheet}>
					<SpreadCanvas
						book={book}
						spread={spread}
						layout={layout}
						zoom={zoom.zoom}
						label={t('region.stage')}
						marks={marks}
						colours={colours}
						snap={snap}
						images={images}
						selection={selection}
						onSelect={onSelect}
						onMove={onMove}
						onCommand={onCommand}
						onEditText={onEditText}
						onPageMenu={(at, point) => setPageMenu({ at, point })}
						onFrameMenu={(_, at) => setMenu(at)}
					/>
				</div>
			</div>
			{menu && chosen.length > 0 && (
				<ContextMenu
					items={frameMenuItems(chosen)}
					position={menu}
					ariaLabel={t('frame.menu')}
					onSelect={(item) => {
						setMenu(null);
						onCommand?.(
							item.id as FrameCommand,
							chosen.map((f) => f.id),
						);
					}}
					onClose={() => setMenu(null)}
				/>
			)}
			{pageMenu && (
				<ContextMenu
					items={pageMenuItems(marks ?? DEFAULT_MARKS, spread.kind !== 'cover')}
					position={pageMenu.at}
					ariaLabel={t('page.menu')}
					onSelect={(item) => {
						const { point } = pageMenu;
						setPageMenu(null);
						onPageCommand?.(item.id as PageCommand, point);
					}}
					onClose={() => setPageMenu(null)}
				/>
			)}
		</>
	);
}
