// What the colour pickers offer in Folio: the theme colours, black and white, the book's swatches and the labels
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { ColourOption, ColourPickerLabels } from '@liminal-hq/folio-chrome/ColourPicker/types';
import type { Book } from '@liminal-hq/folio-protocol/generated/Book';
import { t, type MessageKey } from '../i18n/messages';

/** The ink a book's text is set in until a style or the words say otherwise. */
export const DEFAULT_INK = '#1c1b20';
/** The colour a page is when nothing sets one. */
export const DEFAULT_PAGE = '#ffffff';

/** Pure black and white, always offered first. */
export function pinnedColours(): ColourOption[] {
	return [
		{ value: '#000000', name: t('colour.black') },
		{ value: '#ffffff', name: t('colour.white') },
	];
}

const GREYS: ReadonlyArray<readonly [hex: string, name: MessageKey]> = [
	['#262626', 'colour.charcoal'],
	['#4d4d4d', 'colour.graphite'],
	['#808080', 'colour.midGrey'],
	['#a6a6a6', 'colour.silver'],
	['#cccccc', 'colour.lightGrey'],
	['#f0f0f0', 'colour.paperGrey'],
];

/** Folio's own colours, which read well on paper: ink first. */
const PALETTE: ReadonlyArray<readonly [hex: string, name: MessageKey]> = [
	[DEFAULT_INK, 'colour.ink'],
	['#5b2a4a', 'colour.plum'],
	['#1f4e5f', 'colour.teal'],
	['#2f5d3a', 'colour.forest'],
	['#a23b2a', 'colour.brick'],
	['#c98b2b', 'colour.marigold'],
	['#2b3a67', 'colour.navy'],
	['#4a4a4a', 'colour.slate'],
];

/** The greys and then the palette: the grid under "Theme colours". */
export function themeColours(): ColourOption[] {
	return [...GREYS, ...PALETTE].map(([value, name]) => ({ value, name: t(name) }));
}

/** The name of a colour Folio offers by default, for a status line or a tooltip. */
export function colourName(hex: string): string | null {
	const found = [...pinnedColours(), ...themeColours()].find((c) => c.value === hex);
	return found?.name ?? null;
}

/** A swatch of the book as the picker lists it. */
export interface SwatchInput {
	id: string;
	name: string;
	/** The `#rrggbb` it draws as. */
	display?: string;
	ink?: { c: number; m: number; y: number; k: number } | null;
}

export function swatchOptions(swatches: readonly SwatchInput[]): ColourOption[] {
	return swatches.map((s) => ({
		value: `swatch:${s.id}`,
		name: s.name,
		...(s.display ? { shown: s.display } : {}),
		...(s.ink ? { ink: [s.ink.c, s.ink.m, s.ink.y, s.ink.k] as const } : {}),
	}));
}

/** The swatches of `book` for a picker. */
export function bookSwatchOptions(book: Book): ColourOption[] {
	return swatchOptions(book.styles?.swatches ?? []);
}

/** The picker's labels in the app's language. */
export function colourPickerLabels(): ColourPickerLabels {
	return {
		tabSwatches: t('colour.picker.tabSwatches'),
		tabCustom: t('colour.picker.tabCustom'),
		tabs: t('colour.picker.tabs'),
		fromStyle: t('colour.picker.fromStyle'),
		fromStyleInUse: t('colour.picker.inUse'),
		fromStyleOverridden: t('colour.picker.overridden'),
		reset: t('colour.picker.reset'),
		bookSwatches: t('colour.picker.book'),
		blackAndWhite: t('colour.picker.blackAndWhite'),
		themeColours: t('colour.picker.theme'),
		recent: t('colour.picker.recent'),
		recentColour: t('colour.picker.recentColour'),
		none: t('colour.none'),
		current: t('colour.picker.current'),
		ink: t('colour.picker.ink'),
		saturationBrightness: t('colour.picker.square'),
		saturationBrightnessValue: (saturation, brightness) =>
			t('colour.picker.squareValue', { saturation, brightness }),
		hue: t('colour.picker.hue'),
		hueValue: (degrees) => t('colour.picker.hueValue', { degrees }),
		hex: t('colour.picker.hex'),
		hexInvalid: t('colour.picker.hexInvalid'),
		red: 'R',
		green: 'G',
		blue: 'B',
		eyedropper: t('colour.picker.eyedropper'),
		eyedropperUnavailable: t('colour.picker.eyedropperUnavailable'),
		eyedropperCancelled: t('colour.picker.eyedropperCancelled'),
		contrastText: t('colour.picker.contrastText'),
		contrastGraphic: t('colour.picker.contrastGraphic'),
		contrastAaa: t('colour.picker.aaa'),
		contrastAa: t('colour.picker.aa'),
		contrastLarge: t('colour.picker.large'),
		contrastFail: t('colour.picker.fail'),
		contrastGraphicOk: t('colour.picker.graphicOk'),
		contrastGraphicLow: t('colour.picker.graphicLow'),
		ratio: (ratio) => t('colour.picker.ratio', { ratio }),
		swatchName: (name, hex) => t('colour.picker.swatchName', { name, hex }),
	};
}
