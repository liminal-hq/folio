// What the ribbon's document controls do: which are live for the selection, and what each one asks for
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { Frame } from '@liminal-hq/folio-protocol/generated/Frame';
import type { Intent } from '@liminal-hq/folio-protocol/generated/Intent';
import type { Spread } from '@liminal-hq/folio-protocol/generated/Spread';
import type { SpreadLayout } from '@liminal-hq/folio-protocol/generated/SpreadLayout';
import type { Marks } from '../canvas/marksModel';
import { surfaceOf } from '../canvas/masterView';
import type { NewFrameKind } from '../canvas/newFrames';
import type { StyleChoice } from '../inspector/styleChoices';
import { effectiveSize, withOverrides } from '../inspector/textFacts';

export interface EditorInput {
	spread: Spread;
	/** Where the spread is on the shelf, and how many there are. */
	spreadIndex: number;
	spreadCount: number;
	selected: Frame[];
	layout: SpreadLayout | null;
	choices: StyleChoice[];
	marks: Marks;
	snap: boolean;
	/** Whether the rulers show. */
	rulers: boolean;
	/** Whether `spread` is a master page being edited, not one of the book's spreads. */
	master?: boolean;
}

/** What the controls show: which can be used now, which are pressed, the numbers, the extra tabs. */
export interface EditorState {
	available: ReadonlySet<string>;
	active: Readonly<Record<string, boolean>>;
	values: Readonly<Record<string, number>>;
	contextual: readonly string[];
}

/** What a control asks the book view to do. */
export type EditorAction =
	| { kind: 'intents'; intents: Intent[] }
	| { kind: 'add'; shape: NewFrameKind }
	| { kind: 'marks'; marks: Partial<Marks> }
	| { kind: 'snap' }
	| { kind: 'rulers' }
	| { kind: 'zoom'; zoom: 'in' | 'out' | 'fit' }
	| { kind: 'page'; op: 'add' | 'duplicate' | 'remove' }
	| { kind: 'choosePicture'; frame: string }
	| { kind: 'stretch'; frame: string }
	| { kind: 'fit'; frame: string; fit: 'fill' | 'fit' }
	| { kind: 'editMaster' };

const ALIGNS: Record<string, 'left' | 'center' | 'right' | 'justify'> = {
	alignL: 'left',
	alignC: 'center',
	alignR: 'right',
	justify: 'justify',
};

const texts = (frames: Frame[]) =>
	frames.flatMap((f) => (f.type === 'text' && !f.locked && !f.fromMaster ? [f] : []));
const free = (frames: Frame[]) => frames.filter((f) => !f.locked && !f.fromMaster);

export function editorState(input: EditorInput): EditorState {
	const { spread, selected, marks, snap, layout, choices } = input;
	const available = new Set<string>([
		'text',
		'image',
		'bubble',
		'rect',
		'round',
		'ellipse',
		'line',
		'speech',
		'thought',
		'shout',
		'addPage',
		'margins',
		'grid',
		'bleed',
		'guides',
		'rulers',
		'rguides',
		'snap',
		'zoomIn',
		'zoomOut',
		'fit',
	]);
	const active: Record<string, boolean> = {
		margins: marks.margins,
		grid: marks.columns || marks.baseline,
		bleed: marks.bleed,
		guides: marks.frameEdges,
		rulers: input.rulers,
		rguides: marks.guides,
		snap,
	};
	const values: Record<string, number> = {};
	const contextual: string[] = [];

	if (input.master) {
		available.delete('addPage');
	} else {
		if (spread.kind !== 'cover') {
			available.add('dupPage');
			if (input.spreadCount > 1) available.add('delPage');
		}
		if (spread.guides.some((g) => !g.locked)) available.add('rgClear');
	}
	if (input.master) active.master = true;
	available.add('master');
	if (selected.length > 0) available.add('lock');
	if (selected.length > 0 && selected.every((f) => f.locked)) active.lock = true;
	if (free(selected).length > 0) {
		available.add('front');
		available.add('back');
	}
	const words = texts(selected);
	if (words.length > 0) {
		for (const id of ['bold', 'italic', ...Object.keys(ALIGNS), 'size']) available.add(id);
		const first = words[0]!;
		active.bold = first.overrides?.bold === true;
		active.italic = first.overrides?.italic === true;
		const align = first.overrides?.align ?? 'left';
		for (const [id, value] of Object.entries(ALIGNS)) active[id] = align === value;
		values.size = effectiveSize(first, layout, choices);
	}
	const only = selected.length === 1 ? selected[0]! : null;
	if (only?.type === 'image') {
		contextual.push('picture');
		for (const id of ['replace', 'toBleed', 'fitFill']) available.add(id);
		active.fitFill = only.image.fit === 'fit';
	}
	if (only?.type === 'shape') {
		contextual.push('shape');
		if (!only.locked && !only.fromMaster) {
			available.add('rotL');
			available.add('rotR');
		}
	}
	return { available, active, values, contextual };
}

/** The action a control asks for, or `null` when it is not one of the document's controls. */
export function editorAction(
	id: string,
	value: string | number | boolean | undefined,
	input: EditorInput,
): EditorAction | null {
	const { spread, selected } = input;
	const on = { on: surfaceOf(spread) };
	switch (id) {
		case 'text':
			return { kind: 'add', shape: 'text' };
		case 'image':
			return { kind: 'add', shape: 'picture' };
		case 'rect':
		case 'round':
		case 'ellipse':
		case 'line':
		case 'speech':
		case 'thought':
		case 'shout':
			return { kind: 'add', shape: id };
		case 'bubble':
			return { kind: 'add', shape: 'speech' };
		case 'addPage':
			return { kind: 'page', op: 'add' };
		case 'dupPage':
			return { kind: 'page', op: 'duplicate' };
		case 'delPage':
			return { kind: 'page', op: 'remove' };
		case 'margins':
			return { kind: 'marks', marks: { margins: !input.marks.margins } };
		case 'bleed':
			return { kind: 'marks', marks: { bleed: !input.marks.bleed } };
		case 'guides':
			return { kind: 'marks', marks: { frameEdges: !input.marks.frameEdges } };
		case 'grid': {
			const next = !(input.marks.columns || input.marks.baseline);
			return { kind: 'marks', marks: { columns: next, baseline: next } };
		}
		case 'snap':
			return { kind: 'snap' };
		case 'rulers':
			return { kind: 'rulers' };
		case 'rguides':
			return { kind: 'marks', marks: { guides: !input.marks.guides } };
		case 'master':
			return { kind: 'editMaster' };
		case 'rgClear':
			return { kind: 'intents', intents: [{ type: 'ClearGuides', spread: spread.id }] };
		case 'zoomIn':
			return { kind: 'zoom', zoom: 'in' };
		case 'zoomOut':
			return { kind: 'zoom', zoom: 'out' };
		case 'fit':
			return { kind: 'zoom', zoom: 'fit' };
		case 'front':
		case 'back':
			return {
				kind: 'intents',
				intents: free(selected).map((f) => ({
					type: 'Reorder',
					...on,
					frame: f.id,
					to: id === 'front' ? 'front' : 'back',
				})),
			};
		case 'lock': {
			const lock = !selected.every((f) => f.locked);
			return {
				kind: 'intents',
				intents: selected
					.filter((f) => f.locked !== lock)
					.map((f) => ({ type: 'LockFrame', ...on, frame: f.id, locked: lock })),
			};
		}
		case 'bold':
		case 'italic': {
			const words = texts(selected);
			const turnOn = words[0]?.overrides?.[id] !== true;
			return {
				kind: 'intents',
				intents: words.map((f) => ({
					type: 'OverrideStyle',
					frame: f.id,
					overrides: withOverrides(f, { [id]: turnOn }),
				})),
			};
		}
		case 'alignL':
		case 'alignC':
		case 'alignR':
		case 'justify':
			return {
				kind: 'intents',
				intents: texts(selected).map((f) => ({
					type: 'OverrideStyle',
					frame: f.id,
					overrides: withOverrides(f, { align: ALIGNS[id]! }),
				})),
			};
		case 'size': {
			const size = typeof value === 'number' ? value : Number(value);
			if (!Number.isFinite(size)) return null;
			return {
				kind: 'intents',
				intents: texts(selected).map((f) => ({
					type: 'OverrideStyle',
					frame: f.id,
					overrides: withOverrides(f, { size: Math.min(200, Math.max(4, size)) }),
				})),
			};
		}
		case 'rotL':
		case 'rotR': {
			const only = selected.length === 1 ? selected[0]! : null;
			if (only?.type !== 'shape' || only.locked) return null;
			const turn = id === 'rotR' ? 90 : -90;
			return {
				kind: 'intents',
				intents: [{ type: 'RotateFrame', ...on, frame: only.id, degrees: only.rotation + turn }],
			};
		}
		case 'replace':
		case 'toBleed':
		case 'fitFill': {
			const only = selected.length === 1 ? selected[0]! : null;
			if (only?.type !== 'image' || only.locked) return null;
			if (id === 'replace') return { kind: 'choosePicture', frame: only.id };
			if (id === 'toBleed') return { kind: 'stretch', frame: only.id };
			return { kind: 'fit', frame: only.id, fit: only.image.fit === 'fit' ? 'fill' : 'fit' };
		}
		default:
			return null;
	}
}
