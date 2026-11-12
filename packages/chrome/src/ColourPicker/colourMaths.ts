// Colour maths for the picker: hex parsing and formatting, HSV, WCAG contrast and plain CMYK
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

export type Rgb = readonly [r: number, g: number, b: number];
export interface Hsv {
	/** Degrees, 0 up to (not including) 360. */
	h: number;
	/** 0 to 1. */
	s: number;
	/** 0 to 1. */
	v: number;
}
/** Percent of each plate, 0 to 100. */
export type InkPercents = readonly [c: number, m: number, y: number, k: number];

export const clamp = (n: number, low: number, high: number) => Math.min(high, Math.max(low, n));

/**
 * The stored form (`#rrggbb`, lower case) of typed text, or `null` if it is not a colour. Takes
 * `#abc`, `abc`, `#aabbcc`, `AABBCC` and `rgb(1, 2, 3)`; anything else, including 4 and 8 digit hex
 * (Folio keeps no alpha in a colour), is refused.
 */
export function parseColour(text: string): string | null {
	const typed = text.trim().toLowerCase();
	const hex = /^#?([0-9a-f]{3}|[0-9a-f]{6})$/.exec(typed);
	if (hex) {
		const digits = hex[1]!;
		return `#${digits.length === 3 ? [...digits].map((c) => c + c).join('') : digits}`;
	}
	const rgb = /^rgb\(\s*(\d{1,3})[\s,]+(\d{1,3})[\s,]+(\d{1,3})\s*\)$/.exec(typed);
	if (rgb) {
		const channels = [Number(rgb[1]), Number(rgb[2]), Number(rgb[3])] as const;
		return channels.every((n) => n <= 255) ? rgbToHex(channels) : null;
	}
	return null;
}

export function rgbToHex([r, g, b]: Rgb): string {
	return `#${[r, g, b].map((n) => clamp(Math.round(n), 0, 255).toString(16).padStart(2, '0')).join('')}`;
}

/** The channels of a `#rrggbb` colour; a malformed value reads as black. */
export function hexToRgb(hex: string): [number, number, number] {
	const value = parseColour(hex) ?? '#000000';
	return [1, 3, 5].map((i) => parseInt(value.slice(i, i + 2), 16)) as [number, number, number];
}

export function rgbToHsv([r, g, b]: Rgb): Hsv {
	const [rn, gn, bn] = [r / 255, g / 255, b / 255];
	const max = Math.max(rn, gn, bn);
	const delta = max - Math.min(rn, gn, bn);
	let h = 0;
	if (delta > 0) {
		if (max === rn) h = ((gn - bn) / delta) % 6;
		else if (max === gn) h = (bn - rn) / delta + 2;
		else h = (rn - gn) / delta + 4;
		h *= 60;
		if (h < 0) h += 360;
	}
	return { h, s: max === 0 ? 0 : delta / max, v: max };
}

export function hsvToRgb({ h, s, v }: Hsv): [number, number, number] {
	const channel = (n: number) => {
		const k = (n + h / 60) % 6;
		return (v - v * s * Math.max(0, Math.min(k, 4 - k, 1))) * 255;
	};
	return [channel(5), channel(3), channel(1)];
}

/** The WCAG 2 relative luminance of a `#rrggbb` colour, 0 (black) to 1 (white). */
export function relativeLuminance(hex: string): number {
	const [r, g, b] = hexToRgb(hex).map((c) => {
		const n = c / 255;
		return n <= 0.03928 ? n / 12.92 : ((n + 0.055) / 1.055) ** 2.4;
	}) as [number, number, number];
	return 0.2126 * r + 0.7152 * g + 0.0722 * b;
}

/** The WCAG contrast ratio of two colours, 1 to 21. */
export function contrastRatio(a: string, b: string): number {
	const [x, y] = [relativeLuminance(a), relativeLuminance(b)];
	return (Math.max(x, y) + 0.05) / (Math.min(x, y) + 0.05);
}

export type ContrastGrade = 'aaa' | 'aa' | 'large' | 'fail';

/** How well text of this ratio reads: 7 is AAA, 4.5 is AA, 3 is enough for large text only. */
export function contrastGrade(ratio: number): ContrastGrade {
	if (ratio >= 7) return 'aaa';
	if (ratio >= 4.5) return 'aa';
	if (ratio >= 3) return 'large';
	return 'fail';
}

/** Ink as a screen shows it: the plain inverse, `(1 - plate)(1 - k)`, as `Ink::to_rgb` in Rust does. */
export function inkToHex([c, m, y, k]: InkPercents): string {
	const key = 1 - k / 100;
	return rgbToHex([c, m, y].map((p) => 255 * (1 - p / 100) * key) as [number, number, number]);
}

/** The plain-inverse ink for a screen colour, as a guide only; Folio keeps the sRGB colour. */
export function hexToInk(hex: string): [number, number, number, number] {
	const [r, g, b] = hexToRgb(hex).map((n) => n / 255) as [number, number, number];
	const k = 1 - Math.max(r, g, b);
	if (k >= 1) return [0, 0, 0, 100];
	const plate = (n: number) => Math.round(((1 - n - k) / (1 - k)) * 100);
	return [plate(r), plate(g), plate(b), Math.round(k * 100)];
}
