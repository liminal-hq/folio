// The Swatches tab: the colour from style, the book's swatches, black and white, theme colours, recents
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { useEffect, useMemo, useRef, useState, type KeyboardEvent } from 'react';
import styles from './ColourPicker.module.css';
import { moveInGrid, type GridKey, type GridPos } from './gridNav';
import type { ColourOption, ColourPickerLabels } from './types';
import type { InkPercents } from './colourMaths';

export const inkText = (ink: InkPercents) => `C${ink[0]} M${ink[1]} Y${ink[2]} K${ink[3]}`;

/** What a tile stands for: an option plus the hex it draws as. */
interface Tile extends ColourOption {
	shown: string;
}

interface SectionSpec {
	key: string;
	heading: string;
	kind: 'row' | 'grid';
	cols: number;
	tiles: Tile[];
}

export interface SwatchesTabProps {
	value: string | null;
	/** The `#rrggbb` the current value draws as. */
	currentHex: string | null;
	inherited?: ColourOption | null;
	swatches: ColourOption[];
	pinned: ColourOption[];
	theme: ColourOption[];
	recents: string[];
	allowNone?: boolean;
	disabled?: boolean;
	labels: ColourPickerLabels;
	focusFirst?: boolean;
	onPick(value: string, final: true): void;
	onNone?(): void;
	onReset?(): void;
}

const tile = (option: ColourOption): Tile => ({ ...option, shown: option.shown ?? option.value });
const NONE = '\u0000none';

export function SwatchesTab(props: SwatchesTabProps) {
	const { value, currentHex, inherited, labels, disabled } = props;
	const [over, setOver] = useState<Tile | null>(null);
	const [pos, setPos] = useState<GridPos | null>(null);
	const refs = useRef(new Map<string, HTMLButtonElement>());

	const sections = useMemo<SectionSpec[]>(() => {
		const out: SectionSpec[] = [];
		if (props.allowNone && props.onNone)
			out.push({
				key: 'none',
				heading: '',
				kind: 'row',
				cols: 1,
				tiles: [{ value: NONE, name: labels.none, shown: 'transparent' }],
			});
		out.push({
			key: 'book',
			heading: labels.bookSwatches,
			kind: 'row',
			cols: 1,
			tiles: props.swatches.map(tile),
		});
		out.push({
			key: 'pinned',
			heading: labels.blackAndWhite,
			kind: 'row',
			cols: 2,
			tiles: props.pinned.map(tile),
		});
		out.push({
			key: 'theme',
			heading: labels.themeColours,
			kind: 'grid',
			cols: 8,
			tiles: props.theme.map(tile),
		});
		out.push({
			key: 'recent',
			heading: labels.recent,
			kind: 'grid',
			cols: 8,
			tiles: props.recents.map((hex) => ({ value: hex, name: labels.recentColour, shown: hex })),
		});
		return out;
	}, [
		props.allowNone,
		props.onNone,
		props.swatches,
		props.pinned,
		props.theme,
		props.recents,
		labels,
	]);

	// The tab stop: the colour in use if it is here, else the first tile.
	const stop = useMemo<GridPos>(() => {
		if (pos && sections[pos.section]?.tiles[pos.index]) return pos;
		for (let s = 0; s < sections.length; s += 1) {
			const index = sections[s]!.tiles.findIndex((t) => t.value === value);
			if (index >= 0) return { section: s, index };
		}
		const first = sections.findIndex((s) => s.tiles.length > 0);
		return { section: Math.max(first, 0), index: 0 };
	}, [pos, sections, value]);

	const idOf = (p: GridPos) => `${p.section}:${p.index}`;
	useEffect(() => {
		if (props.focusFirst) refs.current.get(idOf(stop))?.focus();
		// Only when the tab first opens by keyboard.
		// eslint-disable-next-line react-hooks/exhaustive-deps
	}, []);

	const onKeyDown = (event: KeyboardEvent, from: GridPos) => {
		const keys: GridKey[] = ['ArrowLeft', 'ArrowRight', 'ArrowUp', 'ArrowDown', 'Home', 'End'];
		if (!keys.includes(event.key as GridKey)) return;
		event.preventDefault();
		const next = moveInGrid(
			sections.map((s) => ({ count: s.tiles.length, cols: s.cols })),
			from,
			event.key as GridKey,
		);
		if (!next) return;
		setPos(next);
		refs.current.get(idOf(next))?.focus();
	};

	// The status line: what is under the pointer or focus, else the colour in use.
	const current = currentHex ?? inherited?.shown ?? inherited?.value ?? null;
	const known = [...props.swatches, ...props.pinned, ...props.theme].find(
		(o) => o.value === value || (o.shown ?? o.value) === current,
	);
	const subject: { name?: string; hex: string | null; ink?: InkPercents | null } = over
		? { name: over.name, hex: over.shown === 'transparent' ? null : over.shown, ink: over.ink }
		: { name: known?.name, hex: current, ink: known?.ink };
	const status = [
		over ? null : labels.current,
		subject.name && subject.name !== labels.recentColour ? subject.name : null,
		subject.hex ?? labels.none,
		subject.ink ? `${labels.ink} ${inkText(subject.ink)}` : null,
	]
		.filter(Boolean)
		.join(' · ');

	const choose = (t: Tile) => {
		if (disabled) return;
		if (t.value === NONE) props.onNone?.();
		else props.onPick(t.value, true);
	};

	return (
		<div className={styles.panel}>
			{inherited && (
				<div className={styles.fromStyle} data-on={value === null ? '' : undefined}>
					<span
						className={styles.chip}
						style={{ background: inherited.shown ?? inherited.value }}
						aria-hidden="true"
					/>
					<span className={styles.fromText}>
						<b className={styles.ellipsis}>{labels.fromStyle}</b>
						<span className={`${styles.ellipsis} ${styles.sub}`}>
							{inherited.shown ?? inherited.value} ·{' '}
							{value === null ? labels.fromStyleInUse : labels.fromStyleOverridden}
						</span>
					</span>
					<button
						type="button"
						className={styles.button}
						disabled={disabled || value === null || !props.onReset}
						onMouseDown={(event) => event.preventDefault()}
						onClick={props.onReset}
					>
						{labels.reset}
					</button>
				</div>
			)}
			{sections.map((section, s) =>
				section.tiles.length === 0 ? null : (
					<section key={section.key} className={styles.section}>
						{section.heading && <h4 className={styles.heading}>{section.heading}</h4>}
						<div
							className={styles.grid}
							data-cols={section.cols}
							role="group"
							aria-label={section.heading || labels.none}
						>
							{section.tiles.map((t, i) => {
								const here: GridPos = { section: s, index: i };
								const pressed = t.value === NONE ? value === null && !inherited : t.value === value;
								const hex = t.shown === 'transparent' ? labels.none : t.shown;
								return (
									<button
										key={`${t.value}:${i}`}
										ref={(el) => {
											if (el) refs.current.set(idOf(here), el);
											else refs.current.delete(idOf(here));
										}}
										type="button"
										className={styles.swatch}
										data-kind={section.kind}
										data-ink={t.ink ? '' : undefined}
										tabIndex={stop.section === s && stop.index === i ? 0 : -1}
										title={t.name}
										aria-label={labels.swatchName(t.name, hex)}
										aria-pressed={pressed}
										disabled={disabled}
										style={section.kind === 'grid' ? { background: t.shown } : undefined}
										onMouseDown={(event) => event.preventDefault()}
										onClick={() => choose(t)}
										onKeyDown={(event) => onKeyDown(event, here)}
										onPointerEnter={() => setOver(t)}
										onPointerLeave={() => setOver(null)}
										onFocus={() => {
											setPos(here);
											setOver(t);
										}}
										onBlur={() => setOver(null)}
									>
										{section.kind === 'row' && (
											<>
												<span
													className={styles.chip}
													style={{ background: t.shown }}
													aria-hidden="true"
												/>
												<span className={`${styles.rowName} ${styles.ellipsis}`}>{t.name}</span>
												{t.ink && (
													<span className={`${styles.rowInk} ${styles.ellipsis}`}>
														{inkText(t.ink)}
													</span>
												)}
												{t.shown !== 'transparent' && (
													<span className={styles.rowHex}>{t.shown}</span>
												)}
											</>
										)}
									</button>
								);
							})}
						</div>
					</section>
				),
			)}
			<p className={`${styles.status} ${styles.ellipsis}`} title={status} aria-live="polite">
				{status}
			</p>
		</div>
	);
}
