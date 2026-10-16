// The stage: a spread on the table with its title and zoom, fitted to the room there is
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { TableEdit } from '@liminal-hq/folio-protocol/generated/TableEdit';
import type { Speller } from './TextEditor';
import type { Book } from '@liminal-hq/folio-protocol/generated/Book';
import type { Spread } from '@liminal-hq/folio-protocol/generated/Spread';
import type { SpreadLayout } from '@liminal-hq/folio-protocol/generated/SpreadLayout';
import { ContextMenu } from '@liminal-hq/folio-chrome/ContextMenu';
import type { RulerUnit } from '@liminal-hq/folio-protocol/generated/RulerUnit';
import { useState } from 'react';
import type { MutableRefObject } from 'react';
import { t } from '../i18n/messages';
import { frameMenuItems, type FrameCommand } from './frameMenu';
import { pageMenuItems, type PageCommand } from './pageMenu';
import { fitZoom, PX_PER_POINT, spreadGeometry } from './geometry';
import { guideMenuItems, isGuideCommand } from './guideMenu';
import type { GuideAxis, GuideCommand } from './guides';
import { MarksMenu } from './MarksMenu';
import { masterAt, masterItemsAt } from './masterItems';
import { canPromote, masterGuidesOn } from './masterGuides';
import { isMasterSpread } from './masterView';
import {
	DEFAULT_MARK_COLOURS,
	DEFAULT_MARKS,
	type MarkColours,
	type MarkKey,
	type Marks,
} from './marksModel';
import { Ruler, RULER_SIZE } from './Ruler';
import { formatPosition, RULER_UNITS } from './rulerMath';
import { rulerUnitItems } from './rulerMenu';
import { snapTargets } from './snap';
import { SpreadCanvas } from './SpreadCanvas';
import styles from './Stage.module.css';
import { useElementSize } from './useElementSize';
import type { FrameBoxMove } from './useFrameDrag';
import { useGuideDrag } from './useGuideDrag';
import { usePageOrigin } from './usePageOrigin';
import { useStageZoom, type StageZoom } from './useStageZoom';

export type ZoomApi = Pick<StageZoom, 'zoomIn' | 'zoomOut' | 'fit'>;

export interface StageProps {
	book: Book;
	spread: Spread;
	/** What the shelf calls this spread: "Cover", "Pages 2–3". */
	label: string;
	/** Replaces the "Spread …" title, for a page that is not one of the book's spreads. */
	heading?: string;
	layout: SpreadLayout | null;
	marks?: Marks;
	colours?: MarkColours;
	onMarks?(next: Partial<Marks>): void;
	onMarkColour?(key: MarkKey, index: number): void;
	snap?: boolean;
	/** Whether the rulers show, and what they count in. */
	rulers?: { on: boolean; unit: RulerUnit };
	onRulerUnit?(unit: RulerUnit): void;
	/** A guide was pulled out of a ruler, moved, or dragged back onto one. */
	onAddGuide?(axis: GuideAxis, pos: number): void;
	onMoveGuide?(index: number, pos: number): void;
	onRemoveGuide?(index: number): void;
	/** A choice from a guide's menu. */
	onGuideCommand?(command: GuideCommand, index: number): void;
	/** Filled in with the stage's zoom controls, for the ribbon and the keys to use. */
	zoomApi?: MutableRefObject<ZoomApi | null>;
	images?: Record<string, string>;
	selection?: string[];
	onSelect?(ids: string[]): void;
	onMove?(moves: FrameBoxMove[]): Promise<void>;
	onTurn?(id: string, degrees: number): Promise<void>;
	onCommand?(command: FrameCommand, ids: string[]): void;
	onEditText?(id: string, text: string): void;
	onEditTable?(id: string, edit: TableEdit): void;
	speller?: Speller;
	onEditingCell?(cell: { row: number; col: number } | null): void;
	/** A choice from the page menu; `point` is where it was asked for, in points on the spread. */
	onPageCommand?(command: PageCommand, point: { x: number; y: number }): void;
	/** The spread a master's guide goes back onto from its menu, while a master is edited. */
	demoteTo?: string | null;
	/** Whether the page menu offers to edit the master page under the pointer (Expert). */
	editableMasters?: boolean;
}

export function Stage({
	book,
	spread,
	label,
	heading,
	layout,
	marks,
	colours,
	onMarks,
	onMarkColour,
	snap,
	rulers,
	onRulerUnit,
	onAddGuide,
	onMoveGuide,
	onRemoveGuide,
	onGuideCommand,
	zoomApi,
	images,
	selection,
	onSelect,
	onMove,
	onTurn,
	onCommand,
	onEditText,
	onEditTable,
	onEditingCell,
	speller,
	onPageCommand,
	editableMasters = false,
	demoteTo = null,
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
	const rulersOn = rulers?.on === true;
	const rulerUnit = rulers?.unit ?? 'inches';
	const [unitMenu, setUnitMenu] = useState<{ x: number; y: number } | null>(null);
	const scale = zoom.zoom * PX_PER_POINT;
	const origin = usePageOrigin(ref, scale, book.bleed);
	const extent = chosen.length
		? {
				x: {
					from: Math.min(...chosen.map((f) => f.box.x)),
					to: Math.max(...chosen.map((f) => f.box.x + f.box.w)),
				},
				y: {
					from: Math.min(...chosen.map((f) => f.box.y)),
					to: Math.max(...chosen.map((f) => f.box.y + f.box.h)),
				},
			}
		: null;
	const [guideMenu, setGuideMenu] = useState<{
		index: number;
		at: { x: number; y: number };
	} | null>(null);
	const guideDrag = useGuideDrag({
		stage: ref,
		bounds: geometry.bounds,
		scale,
		targets: (moving) =>
			snapTargets(
				geometry,
				spread.frames,
				[],
				[
					...spread.guides.flatMap((g, i) => (i === moving ? [] : [{ axis: g.axis, pos: g.pos }])),
					...masterGuidesOn(book, spread).map((g) => ({ axis: g.axis, pos: g.pos })),
				],
			),
		fineStep: RULER_UNITS[rulerUnit].fineStep,
		snap: snap ?? true,
		onAdd: (axis, pos) => onAddGuide?.(axis, pos),
		onMove: (index, pos) => onMoveGuide?.(index, pos),
		onRemove: (index) => onRemoveGuide?.(index),
	});
	const askUnit = (event: { preventDefault(): void; clientX: number; clientY: number }) => {
		event.preventDefault();
		setUnitMenu({ x: event.clientX, y: event.clientY });
	};

	return (
		<>
			<div className={styles.header}>
				<h1 className={styles.title}>{heading ?? t('stage.spread', { label })}</h1>
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
			<div className={styles.area}>
				{rulersOn && (
					<>
						<button
							type="button"
							className={styles.corner}
							style={{ inlineSize: RULER_SIZE, blockSize: RULER_SIZE }}
							onClick={(event) => {
								const box = event.currentTarget.getBoundingClientRect();
								setUnitMenu({ x: box.left, y: box.bottom });
							}}
							aria-label={t('ruler.units')}
							title={t('ruler.units')}
						>
							{RULER_UNITS[rulerUnit].suffix}
						</button>
						<Ruler
							axis="x"
							unit={rulerUnit}
							scale={scale}
							origin={origin.x}
							length={size.w}
							extent={extent?.x}
							onPointerDown={guideDrag.fromRuler('x')}
							onContextMenu={askUnit}
						/>
						<Ruler
							axis="y"
							unit={rulerUnit}
							scale={scale}
							origin={origin.y}
							length={size.h}
							extent={extent?.y}
							onPointerDown={guideDrag.fromRuler('y')}
							onContextMenu={askUnit}
						/>
					</>
				)}
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
							onTurn={onTurn}
							onCommand={onCommand}
							onEditText={onEditText}
							onEditTable={onEditTable}
							speller={speller}
							onEditingCell={onEditingCell}
							onPageMenu={(at, point) => setPageMenu({ at, point })}
							onFrameMenu={(_, at) => setMenu(at)}
							onGuidePointerDown={(axis, index, event) => guideDrag.fromGuide(axis, index)(event)}
							onGuideMenu={(index, at) => setGuideMenu({ index, at })}
							guideDrag={guideDrag.state}
						/>
					</div>
				</div>
			</div>
			{guideDrag.state && (
				<div
					className={styles.tag}
					data-remove={guideDrag.state.remove || undefined}
					style={{ left: guideDrag.state.pointer.x + 14, top: guideDrag.state.pointer.y + 14 }}
				>
					{guideDrag.state.remove && guideDrag.state.index !== null
						? t('guide.release')
						: formatPosition(guideDrag.state.pos, rulerUnit)}
				</div>
			)}
			{guideMenu && spread.guides[guideMenu.index] && (
				<ContextMenu
					items={guideMenuItems(spread.guides[guideMenu.index]!, {
						// Every-page guides are made and unmade in Expert, where masters can be edited.
						canPromote:
							isMasterSpread(spread) || !editableMasters
								? undefined
								: canPromote(book, spread, spread.guides[guideMenu.index]!),
						canDemote: isMasterSpread(spread)
							? demoteTo != null && !spread.guides[guideMenu.index]!.locked
							: undefined,
					})}
					position={guideMenu.at}
					ariaLabel={t('guide.menu')}
					onSelect={(item) => {
						const { index } = guideMenu;
						setGuideMenu(null);
						if (isGuideCommand(item.id)) onGuideCommand?.(item.id, index);
					}}
					onClose={() => setGuideMenu(null)}
				/>
			)}
			{unitMenu && (
				<ContextMenu
					items={rulerUnitItems(rulerUnit)}
					position={unitMenu}
					ariaLabel={t('ruler.units')}
					onSelect={(item) => {
						setUnitMenu(null);
						onRulerUnit?.(item.id as RulerUnit);
					}}
					onClose={() => setUnitMenu(null)}
				/>
			)}
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
					items={pageMenuItems(
						marks ?? DEFAULT_MARKS,
						spread.kind !== 'cover',
						masterItemsAt(book, spread, pageMenu.point),
						isMasterSpread(spread),
						editableMasters && !isMasterSpread(spread)
							? masterAt(book, spread, pageMenu.point)
							: null,
					)}
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
