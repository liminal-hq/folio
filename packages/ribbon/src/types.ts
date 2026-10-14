// The ribbon's model: tabs, groups and items, and the commands it reports
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

export type RibbonValue = string | number | boolean;

/** What a control does when it is used. */
export type RibbonItemKind = 'button' | 'toggle' | 'menu' | 'select' | 'stepper';

export interface RibbonItem {
	id: string;
	label: string;
	/** A 16 px SVG path (stroked, round caps), drawn at 22 px for large buttons and 14 px for small ones. */
	d: string;
	size?: 'large' | 'small';
	kind?: RibbonItemKind;
	/** The pressed state of a toggle. When set, the app owns it; otherwise the ribbon keeps it. */
	active?: boolean;
	/** Toggles that share a `radio` key are mutually exclusive (alignment, lists). */
	radio?: string;
	disabled?: boolean;
	/** Hidden in Easy mode. */
	expert?: boolean;
	/** The shortcut shown in the tooltip. */
	key?: string;
	/** The one-line description shown in the tooltip. */
	desc?: string;
	/** A colour (any CSS colour) drawn as a bar under the glyph. */
	swatch?: string;
	/** The choices of a menu or select, as `[value, label]`. */
	options?: [value: string, label: string][];
	value?: string | number;
	/** Pixel width of a select. */
	width?: number;
	step?: number;
	min?: number;
	max?: number;
}

export interface RibbonGroup {
	id: string;
	label: string;
	/** Hidden in Easy mode. */
	expert?: boolean;
	items?: RibbonItem[];
	/** A dense layout: rows, each made of clusters, each made of items. */
	rows?: RibbonItem[][][];
}

export interface RibbonTab {
	id: string;
	label: string;
	/** Shown only while the app lists its id in `contextual`. */
	contextual?: boolean;
	groups: RibbonGroup[];
}

export interface RibbonModel {
	file?: { label: string; items?: [value: string, label: string][] };
	tabs: RibbonTab[];
}

/** Everything the ribbon reports. `tab`, `mode` and `file` are the ribbon's own; the rest are item ids. */
export interface RibbonCommand {
	id: string;
	value?: RibbonValue;
	item?: RibbonItem;
}

export type RibbonMode = 'easy' | 'expert';
export type RibbonVariant = 'full' | 'simplified' | 'collapsed';
export type RibbonDensity = 'comfortable' | 'compact';

/** The classic toolbar: one row of icon buttons in groups, split by hairlines. */
export interface ToolbarModel {
	groups: RibbonItem[][];
}
