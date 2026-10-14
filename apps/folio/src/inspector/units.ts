// Lengths between points, which the book keeps, and the units the person reads
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

export type Units = 'inches' | 'millimetres';

const POINTS_PER_INCH = 72;
const MM_PER_INCH = 25.4;

/** The number a length is shown as, in `units`, with the decimals that matter and no trailing zeros. */
export function fromPoints(points: number, units: Units): number {
	const value =
		units === 'inches' ? points / POINTS_PER_INCH : (points / POINTS_PER_INCH) * MM_PER_INCH;
	return Number(value.toFixed(units === 'inches' ? 3 : 2));
}

/** A length typed in `units`, as points; `null` when it is not a number. */
export function toPoints(text: string, units: Units): number | null {
	const value = Number(text.trim().replace(',', '.'));
	if (text.trim() === '' || !Number.isFinite(value)) return null;
	return units === 'inches' ? value * POINTS_PER_INCH : (value / MM_PER_INCH) * POINTS_PER_INCH;
}

export const unitLabel = (units: Units): string => (units === 'inches' ? 'in' : 'mm');
