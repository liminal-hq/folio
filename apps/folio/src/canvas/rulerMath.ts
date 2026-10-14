// The maths of a ruler: its units, where its ticks fall at a zoom, and how a position reads
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { RulerUnit } from '@liminal-hq/folio-protocol/generated/RulerUnit';

export interface RulerUnitInfo {
	/** Points in one of the unit. */
	points: number;
	/** The short name shown beside a position: "in", "mm". */
	suffix: string;
	/** The step a guide snaps to when nothing nearer pulls it, in points. */
	fineStep: number;
	/** Decimal places a position is written to. */
	decimals: number;
}

export const RULER_UNITS: Readonly<Record<RulerUnit, RulerUnitInfo>> = {
	inches: { points: 72, suffix: 'in', fineStep: 72 / 16, decimals: 3 },
	millimetres: { points: 72 / 25.4, suffix: 'mm', fineStep: 72 / 25.4, decimals: 1 },
	centimetres: { points: 72 / 2.54, suffix: 'cm', fineStep: 72 / 25.4, decimals: 2 },
	points: { points: 1, suffix: 'pt', fineStep: 6, decimals: 1 },
	picas: { points: 12, suffix: 'pc', fineStep: 6, decimals: 2 },
	pixels: { points: 0.75, suffix: 'px', fineStep: 3.75, decimals: 0 },
};

export const RULER_UNIT_ORDER: readonly RulerUnit[] = [
	'inches',
	'millimetres',
	'centimetres',
	'points',
	'picas',
	'pixels',
];

/** Major ticks are kept between these distances apart on screen, in pixels. */
export const MAJOR_MIN_PX = 44;
export const MAJOR_MAX_PX = 160;

export interface Tick {
	/** Where it is, in points from the ruler's zero. */
	pos: number;
	major: boolean;
	/** Written beside a major tick. */
	label?: string;
}

interface Step {
	/** The distance between major ticks, in the unit. */
	major: number;
	/** How many minor spaces make a major one. */
	parts: number;
}

/** The nicest major step, in the unit, that is at least `MAJOR_MIN_PX` wide at this zoom. */
export function stepFor(unit: RulerUnit, scale: number): Step {
	const pxPerUnit = RULER_UNITS[unit].points * scale;
	const wanted = MAJOR_MIN_PX / pxPerUnit;
	const power = Math.floor(Math.log10(wanted));
	for (const exp of [power, power + 1]) {
		const base = 10 ** exp;
		for (const [mantissa, parts] of [
			[1, 10],
			[2, 4],
			[5, 5],
		] as const) {
			if (mantissa * base >= wanted * (1 - 1e-9)) return { major: mantissa * base, parts };
		}
	}
	return { major: 10 ** (power + 1), parts: 10 };
}

/** A number written with only the decimals it needs: 0.5, 12, 0.25. */
export function plain(value: number, decimals: number): string {
	const text = value.toFixed(decimals);
	return (text.includes('.') ? text.replace(/\.?0+$/, '') : text).replace(/^-0$/, '0');
}

/** The ticks from `from` to `to` points (inclusive of the edges they land on) at `scale` px per point. */
export function rulerTicks(from: number, to: number, scale: number, unit: RulerUnit): Tick[] {
	if (!(to > from) || !(scale > 0)) return [];
	const info = RULER_UNITS[unit];
	const { major, parts } = stepFor(unit, scale);
	const minorUnits = major / parts;
	const minorPoints = minorUnits * info.points;
	const first = Math.ceil(from / minorPoints - 1e-9) + 0; // `+ 0` turns -0 into 0
	const last = Math.floor(to / minorPoints + 1e-9);
	const places = Math.max(0, -Math.floor(Math.log10(major)) + 0);
	const ticks: Tick[] = [];
	for (let n = first; n <= last; n++) {
		const isMajor = n % parts === 0;
		const tick: Tick = { pos: n * minorPoints, major: isMajor };
		if (isMajor) tick.label = plain(n * minorUnits, Math.min(places, 4));
		ticks.push(tick);
	}
	return ticks;
}

/** A position in points as it reads in the unit: `1.25 in`. */
export function formatPosition(points: number, unit: RulerUnit): string {
	const info = RULER_UNITS[unit];
	return `${plain(points / info.points, info.decimals)} ${info.suffix}`;
}
