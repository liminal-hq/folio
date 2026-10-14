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
import { DEFAULT_MARKS } from '../canvas/SpreadCanvas';
import { Inspector, type InspectorProps } from './Inspector';

const book = {
	trim: { w: 612, h: 612 },
	bleed: 9,
	layout: 'fixed',
	margins: { top: 36, bottom: 36, inside: 36, outside: 36 },
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

	it('says why words with emphasis or a flow are not edited here', () => {
		setup([text({ content: { inline: [{ text: 'Bold', bold: true }] } })]);
		expect(screen.getByRole('textbox', { name: 'Words' })).toBeDisabled();
		expect(screen.getByText(/bold or italic/)).toBeInTheDocument();
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
			spread: 'sp',
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
