// The inspector card: what is selected and the few things to change about it (Easy mode)
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { Book } from '@liminal-hq/folio-protocol/generated/Book';
import type { Frame } from '@liminal-hq/folio-protocol/generated/Frame';
import type { Intent } from '@liminal-hq/folio-protocol/generated/Intent';
import type { Spread } from '@liminal-hq/folio-protocol/generated/Spread';
import type { SpreadLayout } from '@liminal-hq/folio-protocol/generated/SpreadLayout';
import type { TextFrame } from '@liminal-hq/folio-protocol/generated/TextFrame';
import { Segmented } from '../app/Segmented';
import { formatLength, formatTrim, pageCount } from '../books/format';
import { plainTextOf } from '../canvas/TextEditor';
import { spreadGeometry } from '../canvas/geometry';
import type { Marks } from '../canvas/marksModel';
import { t } from '../i18n/messages';
import styles from './Inspector.module.css';
import { LengthField } from './LengthField';
import { unitLabel } from './units';
import { styleChoices } from './styleChoices';
import { effectiveSize, withOverrides } from './textFacts';

export interface InspectorProps {
	book: Book;
	/** The book's `styles.json`. */
	styleFile: unknown;
	kindLabel: string;
	units: 'inches' | 'millimetres';
	mode: 'easy' | 'expert';
	spread: Spread;
	layout: SpreadLayout | null;
	selected: Frame[];
	marks: Marks;
	onMarks(next: Marks): void;
	apply(intent: Intent): void;
	/** Asks for a picture file and puts it in the frame. */
	choosePicture?(frame: string): void;
}

const MARK_ROWS: { key: keyof Marks; label: string }[] = [
	{ key: 'bleed', label: t('inspector.helpers.bleed') },
	{ key: 'margins', label: t('inspector.helpers.margins') },
	{ key: 'columns', label: t('inspector.helpers.columns') },
	{ key: 'baseline', label: t('inspector.helpers.baseline') },
	{ key: 'frameEdges', label: t('inspector.helpers.frameEdges') },
];

function Row({ label, value }: { label: string; value: string }) {
	return (
		<div className={styles.row}>
			<dt>{label}</dt>
			<dd>{value}</dd>
		</div>
	);
}

function Header({ title, badges }: { title: string; badges: string[] }) {
	return (
		<header className={styles.header}>
			<h2 className={styles.title}>{title}</h2>
			<span className={styles.badges}>
				{badges.map((badge) => (
					<span key={badge} className={styles.badge}>
						{badge}
					</span>
				))}
			</span>
		</header>
	);
}

function BookSection({ book, kindLabel, units, mode, apply, marks, onMarks }: InspectorProps) {
	const pages = pageCount(book);
	return (
		<>
			<Header
				title={t('inspector.title.book')}
				badges={[book.layout === 'flowing' ? t('layout.flowing') : t('layout.fixed')]}
			/>
			<dl className={styles.facts}>
				<Row label={t('inspector.book.kind')} value={kindLabel} />
				<Row label={t('inspector.book.size')} value={formatTrim(book.trim, units)} />
				<Row
					label={t('inspector.book.pages')}
					value={pages === 1 ? t('status.pages.one') : t('status.pages.many', { count: pages })}
				/>
				<Row label={t('inspector.book.bleed')} value={formatLength(book.bleed, units)} />
			</dl>
			{mode === 'expert' && <GeometrySection book={book} units={units} apply={apply} />}
			<fieldset className={styles.group}>
				<legend>{t('inspector.helpers')}</legend>
				{MARK_ROWS.map((row) => (
					<label key={row.key} className={styles.check}>
						<input
							type="checkbox"
							checked={marks[row.key]}
							onChange={(event) => onMarks({ ...marks, [row.key]: event.target.checked })}
						/>
						{row.label}
					</label>
				))}
			</fieldset>
		</>
	);
}

function GeometrySection({ book, units, apply }: Pick<InspectorProps, 'book' | 'units' | 'apply'>) {
	const margins = (change: Partial<Book['margins']>): Intent => ({
		type: 'SetBookGeometry',
		trim: null,
		margins: { ...book.margins, ...change },
		bleed: null,
		grid: null,
	});
	const rows: { key: keyof Book['margins']; label: string }[] = [
		{ key: 'top', label: t('inspector.margin.top') },
		{ key: 'bottom', label: t('inspector.margin.bottom') },
		{ key: 'inside', label: t('inspector.margin.inside') },
		{ key: 'outside', label: t('inspector.margin.outside') },
	];
	return (
		<fieldset className={styles.lengths}>
			<legend>
				{t('inspector.geometry')} ({unitLabel(units)})
			</legend>
			{rows.map((row) => (
				<LengthField
					key={row.key}
					label={row.label}
					points={book.margins[row.key]}
					units={units}
					onCommit={(points) => apply(margins({ [row.key]: points }))}
				/>
			))}
			<LengthField
				label={t('inspector.bleed')}
				points={book.bleed}
				units={units}
				onCommit={(points) =>
					apply({ type: 'SetBookGeometry', trim: null, margins: null, bleed: points, grid: null })
				}
			/>
			<LengthField
				label={t('inspector.gutter')}
				points={book.grid.gutter}
				units={units}
				onCommit={(points) =>
					apply({
						type: 'SetBookGeometry',
						trim: null,
						margins: null,
						bleed: null,
						grid: { ...book.grid, gutter: points },
					})
				}
			/>
			<label className={styles.length}>
				<span>{t('inspector.columns')}</span>
				<input
					type="number"
					min={1}
					max={12}
					value={book.grid.columns}
					aria-label={t('inspector.columns')}
					onChange={(event) => {
						const columns = Math.round(Number(event.target.value));
						if (columns >= 1 && columns <= 12 && columns !== book.grid.columns) {
							apply({
								type: 'SetBookGeometry',
								trim: null,
								margins: null,
								bleed: null,
								grid: { ...book.grid, columns },
							});
						}
					}}
				/>
			</label>
		</fieldset>
	);
}

function PositionSection({
	frame,
	units,
	spread,
	apply,
}: Pick<InspectorProps, 'units' | 'spread' | 'apply'> & { frame: Frame }) {
	const box = frame.box;
	const locked = frame.locked || frame.fromMaster != null;
	const move = (change: Partial<typeof box>): Intent => ({
		type: 'MoveFrame',
		spread: spread.id,
		frame: frame.id,
		box: { x: box.x, y: box.y, w: box.w, h: box.h, ...change },
	});
	const rows: { key: 'x' | 'y' | 'w' | 'h'; label: string }[] = [
		{ key: 'x', label: 'X' },
		{ key: 'y', label: 'Y' },
		{ key: 'w', label: t('inspector.width') },
		{ key: 'h', label: t('inspector.height') },
	];
	return (
		<fieldset className={styles.lengths}>
			<legend>
				{t('inspector.position')} ({unitLabel(units)})
			</legend>
			{rows.map((row) => (
				<LengthField
					key={row.key}
					label={row.label}
					points={box[row.key]}
					units={units}
					disabled={locked}
					onCommit={(points) => {
						const size = row.key === 'w' || row.key === 'h';
						if (size && points < 4) return;
						apply(move({ [row.key]: points }));
					}}
				/>
			))}
		</fieldset>
	);
}

const ALIGNS = ['left', 'center', 'right', 'justify'] as const;

function TextSection({ frame, props }: { frame: TextFrame; props: InspectorProps }) {
	const { layout, styleFile, apply } = props;
	const choices = styleChoices(styleFile);
	const size = effectiveSize(frame, layout, choices);
	const words = plainTextOf(frame);
	const overrides = (change: Partial<NonNullable<TextFrame['overrides']>>): Intent => ({
		type: 'OverrideStyle',
		frame: frame.id,
		overrides: withOverrides(frame, change),
	});
	const align = frame.overrides?.align ?? 'left';
	return (
		<>
			<Header
				title={t('inspector.title.text')}
				badges={[frame.flow ? t('inspector.badge.flowing') : t('inspector.badge.text')]}
			/>
			<label className={styles.field}>
				<span>{t('inspector.words')}</span>
				<textarea
					key={`${frame.id}:${words}`}
					className={styles.words}
					defaultValue={words ?? ''}
					disabled={words === null || frame.flow != null || frame.locked}
					rows={4}
					onBlur={(event) => {
						if (event.target.value !== words) {
							apply({ type: 'SetText', frame: frame.id, text: event.target.value });
						}
					}}
				/>
			</label>
			{words === null && <p className={styles.note}>{t('inspector.words.rich')}</p>}
			{frame.flow != null && <p className={styles.note}>{t('inspector.words.flow')}</p>}
			{choices.length > 0 && (
				<div className={styles.field} role="group" aria-label={t('inspector.style')}>
					<span>{t('inspector.style')}</span>
					<div className={styles.chips}>
						{choices.map((choice) => (
							<button
								key={choice.id}
								type="button"
								className={styles.chip}
								aria-pressed={choice.id === frame.style}
								disabled={frame.locked}
								onClick={() => apply({ type: 'SetStyle', frame: frame.id, style: choice.id })}
							>
								{choice.name}
							</button>
						))}
					</div>
				</div>
			)}
			<div className={styles.field} role="group" aria-label={t('inspector.size')}>
				<span>{t('inspector.size')}</span>
				<div className={styles.stepper}>
					<button
						type="button"
						disabled={frame.locked || size <= 4}
						onClick={() => apply(overrides({ size: Math.max(4, size - 1) }))}
					>
						{t('inspector.size.smaller')}
					</button>
					<output aria-label={t('inspector.size')}>{size} pt</output>
					<button
						type="button"
						disabled={frame.locked || size >= 200}
						onClick={() => apply(overrides({ size: Math.min(200, size + 1) }))}
					>
						{t('inspector.size.bigger')}
					</button>
				</div>
			</div>
			<div className={styles.field}>
				<span id="inspector-align">{t('inspector.align')}</span>
				<Segmented
					label={t('inspector.align')}
					value={align}
					options={ALIGNS.map((a) => ({ value: a, label: t(`inspector.align.${a}`) }))}
					onChange={(next) => !frame.locked && apply(overrides({ align: next }))}
				/>
			</div>
		</>
	);
}

function PictureSection({ frame, props }: { frame: Frame; props: InspectorProps }) {
	const { book, spread, apply } = props;
	const hasPicture = frame.type === 'image' && !!frame.image.asset;
	const stretch = () => {
		const geometry = spreadGeometry(book, spread);
		const centre = frame.box.x + frame.box.w / 2;
		const page =
			geometry.pages.find((p) => centre >= p.trim.x && centre <= p.trim.x + p.trim.w) ??
			geometry.pages[0]!;
		const first = page === geometry.pages[0];
		const last = page === geometry.pages[geometry.pages.length - 1];
		// The page's trim grown by the bleed on the sides that are the edge of the paper.
		const x = page.trim.x - (first ? book.bleed : 0);
		const right = page.trim.x + page.trim.w + (last ? book.bleed : 0);
		apply({
			type: 'MoveFrame',
			spread: spread.id,
			frame: frame.id,
			box: { x, y: -book.bleed, w: right - x, h: page.trim.h + book.bleed * 2 },
		});
	};
	return (
		<>
			<Header
				title={t('inspector.title.picture')}
				badges={[
					frame.type === 'image' && frame.image.asset
						? t('inspector.badge.picture')
						: t('inspector.badge.placeholder'),
				]}
			/>
			{!hasPicture && <p className={styles.note}>{t('inspector.picture.none')}</p>}
			<button
				type="button"
				className={styles.action}
				disabled={frame.locked || !props.choosePicture}
				onClick={() => props.choosePicture?.(frame.id)}
			>
				{t('inspector.picture.choose')}
			</button>
			{hasPicture && (
				<div className={styles.field}>
					<span>{t('inspector.picture.fit')}</span>
					<Segmented
						label={t('inspector.picture.fit')}
						value={frame.type === 'image' && frame.image.fit === 'fit' ? 'fit' : 'fill'}
						options={[
							{ value: 'fill', label: t('inspector.picture.fill') },
							{ value: 'fit', label: t('inspector.picture.inside') },
						]}
						onChange={(fit) =>
							!frame.locked && apply({ type: 'SetImage', frame: frame.id, asset: null, fit })
						}
					/>
				</div>
			)}
			<button type="button" className={styles.action} disabled={frame.locked} onClick={stretch}>
				{t('inspector.picture.stretch')}
			</button>
		</>
	);
}

export function Inspector(props: InspectorProps) {
	const { selected } = props;
	let body;
	if (selected.length === 0) {
		body = <BookSection {...props} />;
	} else if (selected.length > 1) {
		body = (
			<>
				<Header title={t('inspector.title.many', { count: selected.length })} badges={[]} />
				<p className={styles.note}>{t('inspector.many')}</p>
			</>
		);
	} else {
		const frame = selected[0]!;
		const common = frame.fromMaster ? (
			<p className={styles.note}>{t('inspector.master')}</p>
		) : frame.locked ? (
			<p className={styles.note}>{t('inspector.locked')}</p>
		) : null;
		body = (
			<>
				{frame.type === 'text' && <TextSection frame={frame} props={props} />}
				{frame.type === 'image' && <PictureSection frame={frame} props={props} />}
				{(frame.type === 'shape' || frame.type === 'group') && (
					<Header
						title={t(frame.type === 'shape' ? 'inspector.title.shape' : 'inspector.title.group')}
						badges={[]}
					/>
				)}
				{props.mode === 'expert' && (
					<PositionSection
						frame={frame}
						units={props.units}
						spread={props.spread}
						apply={props.apply}
					/>
				)}
				{common}
			</>
		);
	}
	return (
		<aside className={styles.card} aria-label={t('region.inspector')}>
			{body}
		</aside>
	);
}
