// The stage: a spread on the table with its title and zoom, fitted to the room there is
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { Book } from '@liminal-hq/folio-protocol/generated/Book';
import type { Spread } from '@liminal-hq/folio-protocol/generated/Spread';
import type { SpreadLayout } from '@liminal-hq/folio-protocol/generated/SpreadLayout';
import { ContextMenu } from '@liminal-hq/folio-chrome/ContextMenu';
import { useState } from 'react';
import { t } from '../i18n/messages';
import { frameMenuItems, type FrameCommand } from './frameMenu';
import { fitZoom, spreadGeometry, type PlainRect } from './geometry';
import { SpreadCanvas, type Marks } from './SpreadCanvas';
import styles from './Stage.module.css';
import { useElementSize } from './useElementSize';
import { useStageZoom } from './useStageZoom';

export interface StageProps {
	book: Book;
	spread: Spread;
	/** What the shelf calls this spread: "Cover", "Pages 2–3". */
	label: string;
	layout: SpreadLayout | null;
	marks?: Marks;
	selection?: string[];
	onSelect?(ids: string[]): void;
	onMove?(id: string, rect: PlainRect): Promise<void>;
	onCommand?(command: FrameCommand, ids: string[]): void;
	onEditText?(id: string, text: string): void;
}

export function Stage({
	book,
	spread,
	label,
	layout,
	marks,
	selection,
	onSelect,
	onMove,
	onCommand,
	onEditText,
}: StageProps) {
	const [ref, size] = useElementSize<HTMLDivElement>();
	const geometry = spreadGeometry(book, spread);
	const zoom = useStageZoom(fitZoom(geometry.bounds, size));
	const percent = Math.round(zoom.zoom * 100);
	const [menu, setMenu] = useState<{ x: number; y: number } | null>(null);
	const chosen = spread.frames.filter((f) => selection?.includes(f.id));

	return (
		<>
			<div className={styles.header}>
				<h1 className={styles.title}>{t('stage.spread', { label })}</h1>
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
						selection={selection}
						onSelect={onSelect}
						onMove={onMove}
						onCommand={onCommand}
						onEditText={onEditText}
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
		</>
	);
}
