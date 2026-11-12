// The picker's data and labels, all handed in by the app so the picker has no domain imports
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { InkPercents } from './colourMaths';

/** A colour the picker offers. */
export interface ColourOption {
	/** What choosing it sets: `#rrggbb`, or a reference such as `swatch:<id>` the app resolves. */
	value: string;
	name: string;
	/** The `#rrggbb` to draw it in when `value` is a reference. */
	shown?: string;
	/** The ink it prints as, for a swatch that has some. */
	ink?: InkPercents | null;
}

export type ColourContrast = 'text' | 'graphic';

export interface ColourPickerLabels {
	tabSwatches: string;
	tabCustom: string;
	/** Accessible name of the tab list. */
	tabs: string;
	fromStyle: string;
	fromStyleInUse: string;
	fromStyleOverridden: string;
	reset: string;
	bookSwatches: string;
	blackAndWhite: string;
	themeColours: string;
	recent: string;
	recentColour: string;
	none: string;
	current: string;
	/** "Ink" in "Ink C0 M70 Y95 K0". */
	ink: string;
	saturationBrightness: string;
	/** Reads `{saturation}` and `{brightness}` as percentages. */
	saturationBrightnessValue: (saturation: number, brightness: number) => string;
	hue: string;
	hueValue: (degrees: number) => string;
	hex: string;
	hexInvalid: string;
	red: string;
	green: string;
	blue: string;
	eyedropper: string;
	eyedropperUnavailable: string;
	eyedropperCancelled: string;
	contrastText: string;
	contrastGraphic: string;
	contrastAaa: string;
	contrastAa: string;
	contrastLarge: string;
	contrastFail: string;
	contrastGraphicOk: string;
	contrastGraphicLow: string;
	/** Reads a ratio such as "4.5:1". */
	ratio: (ratio: string) => string;
	/** The colour's name and its value, for a swatch button's accessible name. */
	swatchName: (name: string, hex: string) => string;
}

export const defaultColourPickerLabels: ColourPickerLabels = {
	tabSwatches: 'Swatches',
	tabCustom: 'Custom',
	tabs: 'Colour source',
	fromStyle: 'From style',
	fromStyleInUse: 'in use',
	fromStyleOverridden: 'overridden here',
	reset: 'Reset',
	bookSwatches: 'Book swatches',
	blackAndWhite: 'Black and white',
	themeColours: 'Theme colours',
	recent: 'Recent',
	recentColour: 'Recent colour',
	none: 'None',
	current: 'Current',
	ink: 'ink',
	saturationBrightness:
		'Saturation and brightness. Arrow keys adjust; hold Shift for bigger steps.',
	saturationBrightnessValue: (saturation, brightness) =>
		`Saturation ${saturation} percent, brightness ${brightness} percent`,
	hue: 'Hue',
	hueValue: (degrees) => `${degrees} degrees`,
	hex: 'Hex',
	hexInvalid: 'Enter 3 or 6 hex digits, such as #1c1b20, 1c1b20 or #abc.',
	red: 'R',
	green: 'G',
	blue: 'B',
	eyedropper: 'Eyedropper',
	eyedropperUnavailable: 'Not available here: this webview cannot sample colours from the screen.',
	eyedropperCancelled: 'Sampling was cancelled.',
	contrastText: 'Text colour on this page',
	contrastGraphic: 'Against the page colour',
	contrastAaa: 'AAA',
	contrastAa: 'AA',
	contrastLarge: 'AA for large text only',
	contrastFail: 'Fails WCAG',
	contrastGraphicOk: 'Clears 3:1 for graphics',
	contrastGraphicLow: 'Below 3:1 for graphics',
	ratio: (ratio) => `${ratio}:1`,
	swatchName: (name, hex) => `${name}, ${hex}`,
};
