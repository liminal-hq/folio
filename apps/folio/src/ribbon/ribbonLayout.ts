// What each ribbon tab holds: ids, kinds and which parts are for Expert mode
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

/** One control in the layout. Its label, glyph and behaviour are looked up by `id`. */
export interface LayoutItem {
	id: string;
	kind?: 'button' | 'toggle' | 'menu' | 'select' | 'stepper';
	large?: boolean;
	expert?: boolean;
	radio?: string;
	/** The glyph, when it is not named after the id. */
	icon?: string;
}

export interface LayoutGroup {
	id: string;
	expert?: boolean;
	items?: LayoutItem[];
	rows?: LayoutItem[][][];
}

export interface LayoutTab {
	id: string;
	contextual?: boolean;
	groups: LayoutGroup[];
}

export const RIBBON_LAYOUT: readonly LayoutTab[] = [
	{
		id: 'home',
		groups: [
			{
				id: 'home.history',
				items: [
					{
						id: 'undo',
					},
					{
						id: 'redo',
					},
				],
			},
			{
				id: 'home.font',
				rows: [
					[
						[
							{
								id: 'paragraphStyle',
								kind: 'select',
								icon: 'text',
							},
							{
								id: 'font',
								kind: 'select',
								icon: 'text',
							},
							{
								id: 'size',
								kind: 'stepper',
								icon: 'text',
							},
						],
					],
					[
						[
							{
								id: 'bold',
								kind: 'toggle',
							},
							{
								id: 'italic',
								kind: 'toggle',
							},
							{
								id: 'underline',
								kind: 'toggle',
							},
							{
								id: 'strike',
								kind: 'toggle',
							},
						],
						[
							{
								id: 'smallcaps',
								kind: 'toggle',
							},
							{
								id: 'sup',
								kind: 'toggle',
							},
							{
								id: 'sub',
								kind: 'toggle',
							},
						],
						[
							{
								id: 'colour',
								kind: 'menu',
							},
							{
								id: 'highlight',
								kind: 'menu',
							},
						],
						[
							{
								id: 'clear',
							},
						],
					],
				],
			},
			{
				id: 'home.paragraph',
				rows: [
					[
						[
							{
								id: 'alignL',
								kind: 'toggle',
								radio: 'align',
							},
							{
								id: 'alignC',
								kind: 'toggle',
								radio: 'align',
							},
							{
								id: 'alignR',
								kind: 'toggle',
								radio: 'align',
							},
							{
								id: 'justify',
								expert: true,
								kind: 'toggle',
								radio: 'align',
							},
						],
						[
							{
								id: 'lineSpacing',
								kind: 'menu',
							},
							{
								id: 'paraSpacing',
								kind: 'menu',
							},
						],
					],
					[
						[
							{
								id: 'bullets',
								kind: 'toggle',
								radio: 'list',
							},
							{
								id: 'numbers',
								kind: 'toggle',
								radio: 'list',
							},
						],
						[
							{
								id: 'indentLess',
							},
							{
								id: 'indentMore',
							},
							{
								id: 'firstIndent',
							},
						],
						[
							{
								id: 'dropCap',
								kind: 'toggle',
							},
							{
								id: 'tracking',
								expert: true,
								kind: 'menu',
							},
						],
					],
				],
			},
			{
				id: 'home.add',
				items: [
					{ id: 'text', large: true },
					{ id: 'image', large: true },
					{ id: 'table' },
					{ id: 'shapeMenu', kind: 'menu', icon: 'round' },
					{ id: 'bubble', icon: 'speech' },
				],
			},
			{
				id: 'home.editing',
				items: [
					{
						id: 'find',
						large: true,
					},
					{
						id: 'spell',
					},
				],
			},
			{
				id: 'home.help',
				items: [
					{
						id: 'askJax',
						large: true,
						icon: 'jax',
					},
				],
			},
		],
	},
	{
		id: 'insert',
		groups: [
			{
				id: 'insert.frames',
				items: [
					{
						id: 'text',
						large: true,
					},
					{
						id: 'image',
						large: true,
					},
					{
						id: 'table',
						large: true,
					},
				],
			},
			{
				id: 'insert.shapes',
				items: [
					{
						id: 'rect',
					},
					{
						id: 'round',
					},
					{
						id: 'ellipse',
					},
					{
						id: 'line',
					},
					{
						id: 'shape',
					},
				],
			},
			{
				id: 'insert.bubbles',
				items: [
					{
						id: 'speech',
					},
					{
						id: 'thought',
					},
					{
						id: 'shout',
					},
				],
			},
			{
				id: 'insert.decorate',
				items: [
					{
						id: 'ornament',
						large: true,
						kind: 'menu',
					},
				],
			},
			{
				id: 'insert.book-parts',
				items: [
					{
						id: 'cover',
						large: true,
					},
					{
						id: 'chapter',
					},
					{
						id: 'toc',
					},
					{
						id: 'footnote',
					},
					{
						id: 'barcode',
					},
					{
						id: 'glossary',
						expert: true,
					},
					{
						id: 'indexEntry',
						expert: true,
					},
				],
			},
			{
				id: 'insert.templates',
				items: [
					{
						id: 'templates',
						large: true,
					},
				],
			},
			{
				id: 'insert.bring-in',
				items: [
					{
						id: 'importText',
						large: true,
						icon: 'importDoc',
					},
					{
						id: 'importLayout',
						large: true,
						expert: true,
						icon: 'open',
					},
				],
			},
		],
	},
	{
		id: 'pages',
		groups: [
			{
				id: 'pages.pages',
				items: [
					{
						id: 'addPage',
						large: true,
					},
					{
						id: 'dupPage',
					},
					{
						id: 'delPage',
					},
				],
			},
			{
				id: 'pages.templates',
				items: [
					{
						id: 'templates',
						large: true,
					},
				],
			},
			{
				id: 'pages.background',
				items: [
					{
						id: 'pageBg',
						large: true,
						icon: 'background',
					},
				],
			},
			{
				id: 'pages.book',
				items: [
					{
						id: 'cover',
						large: true,
					},
					{
						id: 'pageSize',
						large: true,
						icon: 'book',
					},
					{
						id: 'marginsT',
						icon: 'margins',
					},
				],
			},
			{
				id: 'pages.masters',
				expert: true,
				items: [
					{
						id: 'master',
						kind: 'toggle',
						large: true,
					},
					{
						id: 'thread',
					},
				],
			},
		],
	},
	{
		id: 'layout',
		groups: [
			{
				id: 'layout.rulers',
				items: [
					{
						id: 'rulers',
						large: true,
						kind: 'toggle',
						icon: 'ruler',
					},
					{
						id: 'rguides',
						kind: 'toggle',
					},
					{
						id: 'rulerUnits',
						kind: 'menu',
					},
					{
						id: 'rgClear',
					},
				],
			},
			{
				id: 'layout.helpers',
				items: [
					{
						id: 'margins',
						kind: 'toggle',
					},
					{
						id: 'grid',
						kind: 'toggle',
					},
					{
						id: 'bleed',
						kind: 'toggle',
					},
					{
						id: 'guides',
						kind: 'toggle',
					},
				],
			},
			{
				id: 'layout.snapping',
				items: [
					{
						id: 'snap',
						large: true,
						kind: 'toggle',
					},
				],
			},
			{
				id: 'layout.arrange',
				items: [
					{
						id: 'front',
					},
					{
						id: 'back',
					},
					{
						id: 'group',
					},
					{
						id: 'ungroup',
					},
					{
						id: 'lock',
					},
				],
			},
			{
				id: 'layout.align',
				items: [
					{
						id: 'alignLeftE',
					},
					{
						id: 'alignCenterE',
					},
					{
						id: 'alignRightE',
					},
					{
						id: 'alignTopE',
					},
					{
						id: 'alignMidE',
					},
					{
						id: 'alignBotE',
					},
				],
			},
			{
				id: 'layout.distribute',
				items: [
					{
						id: 'distH',
					},
					{
						id: 'distV',
					},
				],
			},
			{
				id: 'layout.type-grid',
				expert: true,
				items: [
					{
						id: 'baseline',
						kind: 'toggle',
					},
					{
						id: 'columns',
						kind: 'toggle',
					},
				],
			},
		],
	},
	{
		id: 'view',
		groups: [
			{
				id: 'view.preview',
				items: [
					{
						id: 'print',
						large: true,
						kind: 'toggle',
						radio: 'preview',
					},
					{
						id: 'kindle',
						large: true,
						kind: 'toggle',
						radio: 'preview',
						icon: 'ereader',
					},
					{
						id: 'kobo',
						large: true,
						kind: 'toggle',
						radio: 'preview',
						icon: 'ereader',
					},
				],
			},
			{
				id: 'view.zoom',
				items: [
					{
						id: 'zoomIn',
					},
					{
						id: 'zoomOut',
					},
					{
						id: 'fit',
					},
				],
			},
			{
				id: 'view.interface',
				items: [
					{
						id: 'toolbarLayout',
						large: true,
						kind: 'menu',
						icon: 'toolbarI',
					},
					{
						id: 'sidebarLayout',
						large: true,
						kind: 'menu',
						icon: 'sidebar',
					},
					{
						id: 'menubar',
						kind: 'toggle',
					},
				],
			},
		],
	},
	{
		id: 'export',
		groups: [
			{
				id: 'export.check',
				items: [
					{
						id: 'preflight',
						large: true,
						icon: 'check',
					},
				],
			},
			{
				id: 'export.export',
				items: [
					{
						id: 'export',
						large: true,
					},
					{
						id: 'quickPdf',
						icon: 'export',
					},
				],
			},
		],
	},
	{
		id: 'picture',
		contextual: true,
		groups: [
			{
				id: 'picture.picture',
				items: [
					{
						id: 'replace',
						large: true,
					},
					{
						id: 'fitFill',
					},
					{
						id: 'crop',
					},
					{
						id: 'toBleed',
					},
				],
			},
			{
				id: 'picture.rotate',
				items: [
					{
						id: 'rotL',
					},
					{
						id: 'rotR',
					},
					{
						id: 'flipH',
					},
					{
						id: 'flipV',
					},
				],
			},
			{
				id: 'picture.look',
				items: [
					{
						id: 'border',
						large: true,
						kind: 'menu',
					},
					{
						id: 'corners',
						large: true,
						kind: 'menu',
					},
					{
						id: 'filter',
						large: true,
						kind: 'menu',
					},
				],
			},
			{
				id: 'picture.text',
				items: [
					{
						id: 'wrap',
						large: true,
						kind: 'menu',
					},
				],
			},
		],
	},
	{
		id: 'shape',
		contextual: true,
		groups: [
			{
				id: 'shape.shape',
				items: [
					{
						id: 'fill',
						large: true,
						kind: 'menu',
					},
					{
						id: 'stroke',
						large: true,
						kind: 'menu',
					},
					{
						id: 'shapeKind',
						large: true,
						kind: 'menu',
						icon: 'round',
					},
				],
			},
			{
				id: 'shape.rotate',
				items: [
					{
						id: 'rotL',
					},
					{
						id: 'rotR',
					},
				],
			},
			{
				id: 'shape.text',
				items: [
					{
						id: 'wrap',
						large: true,
						kind: 'menu',
					},
				],
			},
		],
	},
	{
		id: 'table',
		contextual: true,
		groups: [
			{
				id: 'table.rows',
				items: [
					{
						id: 'rowAdd',
					},
					{
						id: 'rowDel',
					},
				],
			},
			{
				id: 'table.columns',
				items: [
					{
						id: 'colAdd',
					},
					{
						id: 'colDel',
					},
				],
			},
			{
				id: 'table.look',
				items: [
					{
						id: 'headToggle',
						large: true,
						kind: 'toggle',
						icon: 'headRow',
					},
					{
						id: 'tableStyle',
						large: true,
						kind: 'menu',
						icon: 'table',
					},
				],
			},
		],
	},
];
