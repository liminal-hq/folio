// Tests for the inspector card: what it shows for each selection and the changes it makes
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { Book } from '@liminal-hq/folio-protocol/generated/Book';
import type { Frame } from '@liminal-hq/folio-protocol/generated/Frame';
import type { Spread } from '@liminal-hq/folio-protocol/generated/Spread';
import { fireEvent, render, screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import { DEFAULT_MARKS } from '../canvas/marksModel';
import { Inspector, type InspectorProps } from './Inspector';

const book = {
	trim: { w: 612, h: 612 },
	bleed: 9,
	layout: 'fixed',
	margins: { top: 36, bottom: 36, inside: 36, outside: 36 },
	grid: { columns: 6, gutter: 12, baseline: null },
	spreads: [
		{ kind: 'cover', pages: [{}] },
		{ kind: 'spread', pages: [{}, {}] },
	],
} as unknown as Book;

const spread = {
	id: 'sp',
	kind: 'spread',
	pages: [{ id: 'p1' }, { id: 'p2' }],
	frames: [],
} as unknown as Spread;

const styleFile = {
	styles: {
		body: { name: 'Story text', sizePt: 13 },
		title: { name: 'Title', sizePt: 28 },
	},
};

const text = (extra: object = {}) =>
	({
		type: 'text',
		id: 't',
		box: { x: 0, y: 0, w: 100, h: 40 },
		locked: false,
		style: 'body',
		content: { inline: [{ text: 'Once upon a time' }] },
		...extra,
	}) as unknown as Frame;

function setup(selected: Frame[], extra: Partial<InspectorProps> = {}) {
	const apply = vi.fn();
	const onMarks = vi.fn();
	render(
		<Inspector
			book={book}
			styleFile={styleFile}
			kindLabel="Picture book"
			units="inches"
			mode="easy"
			spread={spread}
			layout={null}
			selected={selected}
			marks={DEFAULT_MARKS}
			onMarks={onMarks}
			apply={apply}
			{...extra}
		/>,
	);
	return { apply, onMarks };
}

describe('with nothing selected', () => {
	it('describes the book and offers the page helpers', async () => {
		const { onMarks } = setup([]);
		expect(screen.getByRole('heading', { name: 'This book' })).toBeInTheDocument();
		expect(screen.getByText('Picture book')).toBeInTheDocument();
		expect(screen.getByText('8.5 × 8.5 in')).toBeInTheDocument();
		await userEvent.click(screen.getByRole('checkbox', { name: 'Columns' }));
		expect(onMarks).toHaveBeenCalledWith({ ...DEFAULT_MARKS, columns: true });
		expect(screen.getByRole('checkbox', { name: 'Margins' })).toBeChecked();
	});
});

describe('page numbers', () => {
	it('shows the choice and sends a new one', async () => {
		const { apply } = setup([]);
		expect(screen.getByRole('radio', { name: 'None' })).toBeChecked();
		await userEvent.click(screen.getByRole('radio', { name: 'Outside' }));
		expect(apply).toHaveBeenCalledWith({ type: 'SetPageNumbers', position: 'outside' });
	});
});

describe('the language of the book', () => {
	const languages = {
		choices: [
			{ value: 'en-CA', label: 'English (Canada)' },
			{ value: 'fr', label: 'French' },
		],
		selected: 'en-CA',
	};

	it('is offered with the dictionaries there are, and sets the language when changed', async () => {
		const { apply } = setup([], { languages });
		const select = screen.getByRole('combobox', { name: 'Language' });
		expect(select).toHaveValue('en-CA');
		await userEvent.selectOptions(select, 'French');
		expect(apply).toHaveBeenCalledWith({ type: 'SetLanguage', language: 'fr' });
	});

	it('is not drawn when the dictionaries have not been listed', () => {
		setup([]);
		expect(screen.queryByRole('combobox', { name: 'Language' })).toBeNull();
	});
});

describe('with a text frame selected', () => {
	it('changes the words when the box loses focus, and only if they changed', () => {
		const { apply } = setup([text()]);
		const words = screen.getByRole('textbox', { name: 'Words' });
		fireEvent.blur(words);
		expect(apply).not.toHaveBeenCalled();
		fireEvent.change(words, { target: { value: 'The otter slept' } });
		fireEvent.blur(words);
		expect(apply).toHaveBeenCalledWith({ type: 'SetText', frame: 't', text: 'The otter slept' });
	});

	it('picks a style from the chips', async () => {
		const { apply } = setup([text()]);
		const chips = screen.getByRole('group', { name: 'Style' });
		expect(within(chips).getByRole('button', { name: 'Story text' })).toHaveAttribute(
			'aria-pressed',
			'true',
		);
		await userEvent.click(within(chips).getByRole('button', { name: 'Title' }));
		expect(apply).toHaveBeenCalledWith({ type: 'SetStyle', frame: 't', style: 'title' });
	});

	it('makes the words bigger or smaller by a point, keeping the other overrides', async () => {
		const { apply } = setup([
			text({ overrides: { size: null, align: 'center', bold: null, italic: null } }),
		]);
		expect(screen.getByText('13 pt')).toBeInTheDocument();
		await userEvent.click(screen.getByRole('button', { name: 'Bigger' }));
		expect(apply).toHaveBeenLastCalledWith({
			type: 'OverrideStyle',
			frame: 't',
			overrides: { size: 14, align: 'center', bold: null, italic: null },
		});
		await userEvent.click(screen.getByRole('button', { name: 'Smaller' }));
		expect(apply).toHaveBeenLastCalledWith(
			expect.objectContaining({ overrides: expect.objectContaining({ size: 12 }) }),
		);
	});

	it('sets the alignment', async () => {
		const { apply } = setup([text()]);
		await userEvent.click(screen.getByRole('radio', { name: 'Right' }));
		expect(apply).toHaveBeenCalledWith({
			type: 'OverrideStyle',
			frame: 't',
			overrides: { size: null, align: 'right', bold: null, italic: null },
		});
	});

	it('edits words with emphasis, since the look of what is left alone is kept', () => {
		setup([text({ content: { inline: [{ text: 'Bold', bold: true }] } })]);
		expect(screen.getByRole('textbox', { name: 'Words' })).toBeEnabled();
		expect(screen.getByRole('textbox', { name: 'Words' })).toHaveValue('Bold');
	});

	it('changes nothing about a locked frame', () => {
		setup([text({ locked: true })]);
		expect(screen.getByRole('textbox', { name: 'Words' })).toBeDisabled();
		expect(screen.getByRole('button', { name: 'Bigger' })).toBeDisabled();
		expect(screen.getByText(/is locked/)).toBeInTheDocument();
	});
});

describe('with a picture selected', () => {
	const picture = {
		type: 'image',
		id: 'i',
		box: { x: 20, y: 20, w: 200, h: 200 },
		locked: false,
		image: { asset: null },
	} as unknown as Frame;

	it('stretches it to the bleed line of its page', async () => {
		const { apply } = setup([picture]);
		expect(screen.getByText('Placeholder')).toBeInTheDocument();
		expect(screen.getByRole('button', { name: 'Choose picture…' })).toBeDisabled();
		await userEvent.click(screen.getByRole('button', { name: 'Stretch to the bleed line' }));
		expect(apply).toHaveBeenCalledWith({
			type: 'MoveFrame',
			on: { spread: 'sp' },
			frame: 'i',
			// The left page: bleed on the outside edge only, and above and below.
			box: { x: -9, y: -9, w: 621, h: 630 },
		});
	});
});

describe('with several frames selected', () => {
	it('counts them', () => {
		setup([text(), text({ id: 'u' })]);
		expect(screen.getByRole('heading', { name: '2 frames' })).toBeInTheDocument();
	});
});

describe('in Expert mode', () => {
	it('shows no number boxes in Easy mode', () => {
		setup([]);
		expect(screen.queryByRole('textbox', { name: /Top margin/ })).toBeNull();
	});

	it('edits the margins in the units the person reads, one change at a time', async () => {
		const { apply } = setup([], { mode: 'expert' });
		const top = screen.getByRole('textbox', { name: 'Top margin (in)' });
		expect(top).toHaveValue('0.5');
		await userEvent.clear(top);
		await userEvent.type(top, '0.75{Enter}');
		expect(apply).toHaveBeenCalledWith({
			type: 'SetBookGeometry',
			trim: null,
			margins: { top: 54, bottom: 36, inside: 36, outside: 36 },
			bleed: null,
			grid: null,
		});
	});

	it('puts a typo back instead of sending it, and Escape too', async () => {
		const { apply } = setup([], { mode: 'expert' });
		const top = screen.getByRole('textbox', { name: 'Top margin (in)' });
		await userEvent.clear(top);
		await userEvent.type(top, 'wide{Enter}');
		expect(top).toHaveValue('0.5');
		await userEvent.type(top, '9{Escape}');
		expect(top).toHaveValue('0.5');
		expect(apply).not.toHaveBeenCalled();
	});

	it('changes the bleed, the gutter and the columns', async () => {
		const { apply } = setup([], { mode: 'expert', units: 'millimetres' });
		const bleed = screen.getByRole('textbox', { name: 'Bleed (mm)' });
		await userEvent.clear(bleed);
		await userEvent.type(bleed, '5{Enter}');
		expect(apply).toHaveBeenLastCalledWith(
			expect.objectContaining({ type: 'SetBookGeometry', bleed: expect.closeTo(14.17, 1) }),
		);
		fireEvent.change(screen.getByRole('spinbutton', { name: 'Columns' }), {
			target: { value: '3' },
		});
		expect(apply).toHaveBeenLastCalledWith(
			expect.objectContaining({ grid: { columns: 3, gutter: 12, baseline: null } }),
		);
	});

	it('moves and resizes the selected frame by number, and not a locked one', async () => {
		const { apply } = setup([text()], { mode: 'expert' });
		const x = screen.getByRole('textbox', { name: 'X (in)' });
		await userEvent.clear(x);
		await userEvent.type(x, '1{Enter}');
		expect(apply).toHaveBeenLastCalledWith({
			type: 'MoveFrame',
			on: { spread: 'sp' },
			frame: 't',
			box: { x: 72, y: 0, w: 100, h: 40 },
		});
		const w = screen.getByRole('textbox', { name: 'Width (in)' });
		await userEvent.clear(w);
		await userEvent.type(w, '0{Enter}');
		expect(apply).toHaveBeenCalledTimes(1);
	});

	it('turns the frame by a typed angle, and puts back what is not a number', async () => {
		const { apply } = setup([text({ rotation: 30 })], { mode: 'expert' });
		const angle = screen.getByRole('textbox', { name: 'Rotation (°)' });
		expect(angle).toHaveValue('30');
		await userEvent.clear(angle);
		await userEvent.type(angle, '-12,5{Enter}');
		expect(apply).toHaveBeenLastCalledWith({
			type: 'RotateFrame',
			on: { spread: 'sp' },
			frame: 't',
			degrees: -12.5,
		});
		await userEvent.clear(angle);
		await userEvent.type(angle, 'sideways{Enter}');
		expect(apply).toHaveBeenCalledTimes(1);
		expect(angle).toHaveValue('30');
	});

	it('has no Rotation box in Easy mode', () => {
		setup([text()]);
		expect(screen.queryByRole('textbox', { name: 'Rotation (°)' })).toBeNull();
	});

	it('disables the position boxes of a locked frame', () => {
		setup([text({ locked: true })], { mode: 'expert' });
		expect(screen.getByRole('textbox', { name: 'X (in)' })).toBeDisabled();
	});
});

describe('choosing a picture', () => {
	const frame = (asset: string | null, fit = 'fill') =>
		({
			type: 'image',
			id: 'i',
			box: { x: 0, y: 0, w: 100, h: 100 },
			locked: false,
			image: { asset, fit, focus: [0.5, 0.5] },
		}) as unknown as Frame;

	it('asks for a picture for the frame', async () => {
		const choosePicture = vi.fn();
		setup([frame(null)], { choosePicture });
		await userEvent.click(screen.getByRole('button', { name: 'Choose picture…' }));
		expect(choosePicture).toHaveBeenCalledWith('i');
		expect(screen.queryByRole('radio', { name: 'Fit inside' })).toBeNull();
	});

	it('offers how the picture sits once there is one', async () => {
		const { apply } = setup([frame('assets/images/a.png')], { choosePicture: vi.fn() });
		expect(screen.getByRole('radio', { name: 'Fill frame' })).toBeChecked();
		await userEvent.click(screen.getByRole('radio', { name: 'Fit inside' }));
		expect(apply).toHaveBeenCalledWith({ type: 'SetImage', frame: 'i', asset: null, fit: 'fit' });
	});
});

describe('the part of a picture to keep in view', () => {
	const picture = (fit: string) =>
		({
			type: 'image',
			id: 'i',
			box: { x: 0, y: 0, w: 100, h: 100 },
			locked: false,
			image: { asset: 'assets/images/a.png', fit, focus: [0.5, 0.5] },
		}) as unknown as Frame;
	const images = { 'assets/images/a.png': 'blob:a' };

	it('is moved by the arrow keys, keeping the picture and fit as they are', async () => {
		const { apply } = setup([picture('fill')], { images });
		screen.getByRole('slider').focus();
		await userEvent.keyboard('{ArrowDown}');
		expect(apply).toHaveBeenCalledWith({
			type: 'SetImage',
			frame: 'i',
			asset: null,
			fit: null,
			focus: [0.5, expect.closeTo(0.55)],
		});
	});

	it('is left out when the whole picture shows or it has not been read yet', () => {
		setup([picture('fit')], { images });
		expect(screen.queryByRole('slider')).toBeNull();
	});
});

describe('with a table selected', () => {
	const table = (extra = {}) =>
		({
			type: 'table',
			id: 'tb',
			box: { x: 0, y: 0, w: 100, h: 50 },
			locked: false,
			rotation: 0,
			cols: [0.5, 0.5],
			rows: [
				[{}, {}],
				[{}, {}],
				[{}, {}],
			],
			head: 1,
			look: 'lined',
			...extra,
		}) as unknown as Frame;

	it('names it and its size, and changes the look and the header row', async () => {
		const { apply } = setup([table()]);
		expect(screen.getByRole('heading', { name: 'Table' })).toBeInTheDocument();
		expect(screen.getByText('3 × 2')).toBeInTheDocument();
		expect(screen.getByRole('radio', { name: 'Lined' })).toBeChecked();
		await userEvent.click(screen.getByRole('radio', { name: 'Grid' }));
		expect(apply).toHaveBeenLastCalledWith({
			type: 'EditTable',
			frame: 'tb',
			edit: { type: 'setLook', look: 'grid' },
		});
		await userEvent.click(screen.getByRole('checkbox', { name: 'Header row' }));
		expect(apply).toHaveBeenLastCalledWith({
			type: 'EditTable',
			frame: 'tb',
			edit: { type: 'setHead', head: 0 },
		});
	});

	it('leaves a locked table alone', async () => {
		const { apply } = setup([table({ locked: true })]);
		expect(screen.getByRole('checkbox', { name: 'Header row' })).toBeDisabled();
		await userEvent.click(screen.getByRole('radio', { name: 'Grid' }));
		expect(apply).not.toHaveBeenCalled();
	});
});

describe('the words box', () => {
	it('keeps what was typed when another frame is chosen before it loses focus', () => {
		const apply = vi.fn();
		const view = render(
			<Inspector
				book={book}
				styleFile={styleFile}
				kindLabel="Picture book"
				units="inches"
				mode="easy"
				spread={spread}
				layout={null}
				selected={[text()]}
				marks={DEFAULT_MARKS}
				onMarks={() => {}}
				apply={apply}
			/>,
		);
		fireEvent.change(screen.getByRole('textbox', { name: 'Words' }), {
			target: { value: 'Half-typed' },
		});
		view.rerender(
			<Inspector
				book={book}
				styleFile={styleFile}
				kindLabel="Picture book"
				units="inches"
				mode="easy"
				spread={spread}
				layout={null}
				selected={[]}
				marks={DEFAULT_MARKS}
				onMarks={() => {}}
				apply={apply}
			/>,
		);
		expect(apply).toHaveBeenCalledWith({ type: 'SetText', frame: 't', text: 'Half-typed' });
		expect(apply).toHaveBeenCalledTimes(1);
	});
});

describe('with a shape selected', () => {
	const shape = (extra: object = {}) =>
		({
			type: 'shape',
			id: 's1',
			box: { x: 0, y: 0, w: 100, h: 50 },
			locked: false,
			shape: 'round',
			fill: '#ffffff',
			style: null,
			...extra,
		}) as unknown as Frame;
	const paints = {
		swatches: [
			{ name: 'Cream', colour: '#f6efdc' },
			{ name: 'Slate', colour: '#1f3a5f' },
		],
		gradients: [],
		textures: [],
	};

	it('changes the fill with a swatch, and clears it', async () => {
		const { apply } = setup([shape()], { paints });
		const fill = within(screen.getByRole('group', { name: 'Fill' }));
		await userEvent.click(fill.getByRole('button', { name: 'Slate' }));
		expect(apply).toHaveBeenLastCalledWith({
			type: 'SetShape',
			frame: 's1',
			fill: { type: 'colour', colour: '#1f3a5f' },
		});
		await userEvent.click(fill.getByRole('button', { name: 'None' }));
		expect(apply).toHaveBeenLastCalledWith({
			type: 'SetShape',
			frame: 's1',
			fill: { type: 'none' },
		});
	});

	it('changes the kind, the outline width and the corner radius of a rounded shape', async () => {
		const { apply } = setup([shape()], { paints });
		await userEvent.selectOptions(screen.getByRole('combobox', { name: 'Shape' }), 'speech');
		expect(apply).toHaveBeenLastCalledWith({ type: 'SetShape', frame: 's1', kind: 'speech' });
		// The boxes read and write in the book's units; the book keeps points.
		const width = screen.getByRole('textbox', { name: 'Outline width (in)' });
		await userEvent.clear(width);
		await userEvent.type(width, '0.05{Enter}');
		expect(apply).toHaveBeenLastCalledWith({ type: 'SetShape', frame: 's1', strokeWidth: 3.6 });
		const radius = screen.getByRole('textbox', { name: 'Corner roundness (in)' });
		await userEvent.clear(radius);
		await userEvent.type(radius, '0.25{Enter}');
		expect(apply).toHaveBeenLastCalledWith({ type: 'SetShape', frame: 's1', radius: 18 });
	});

	it('offers a line no fill, no way to clear its outline and a way to flip it', async () => {
		const { apply } = setup([shape({ shape: 'line', fill: null, stroke: '#14110f' })], { paints });
		expect(screen.queryByRole('group', { name: 'Fill' })).toBeNull();
		const outline = within(screen.getByRole('group', { name: 'Outline' }));
		expect(outline.queryByRole('button', { name: 'None' })).toBeNull();
		expect(screen.queryByRole('textbox', { name: 'Corner roundness (in)' })).toBeNull();
		await userEvent.click(screen.getByRole('checkbox', { name: 'Run the other way' }));
		expect(apply).toHaveBeenLastCalledWith({ type: 'SetShape', frame: 's1', flip: true });
	});

	it('leaves a locked shape as it is', async () => {
		const { apply } = setup([shape({ locked: true })], { paints });
		expect(screen.getByRole('combobox', { name: 'Shape' })).toBeDisabled();
		const fill = within(screen.getByRole('group', { name: 'Fill' }));
		expect(fill.getByRole('button', { name: 'Slate' })).toBeDisabled();
		expect(fill.getByRole('textbox')).toBeDisabled();
		expect(screen.getByRole('textbox', { name: 'Outline width (in)' })).toBeDisabled();
		expect(apply).not.toHaveBeenCalled();
	});
});
