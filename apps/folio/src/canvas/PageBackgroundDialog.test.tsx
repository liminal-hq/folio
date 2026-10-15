// Tests for the page background dialog: where it goes and what is applied
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { Book } from '@liminal-hq/folio-protocol/generated/Book';
import type { Spread } from '@liminal-hq/folio-protocol/generated/Spread';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import { FAKE_PAINTS } from '../books/fakeBooksClient';
import { backgroundTargets, currentBackground } from './backgroundScopes';
import { PageBackgroundDialog } from './PageBackgroundDialog';

const book = { masterPages: [] } as unknown as Book;
const pair = {
	id: 's1',
	kind: 'spread',
	pages: [
		{ id: 'p1', master: null },
		{ id: 'p2', master: null, bg: { type: 'colour', colour: '#f6efdc' } },
	],
	frames: [],
} as unknown as Spread;
const single = { ...pair, id: 's0', kind: 'opener', pages: [pair.pages[0]!] } as Spread;

const setup = (spread: Spread, page: number | null = null) => {
	const onApply = vi.fn();
	render(
		<PageBackgroundDialog
			open
			onClose={() => undefined}
			book={book}
			spread={spread}
			page={page}
			presets={FAKE_PAINTS}
			onApply={onApply}
		/>,
	);
	return onApply;
};

describe('backgroundTargets', () => {
	it('offers both pages, each page and every page for a pair, and the page and every page for one', () => {
		expect(backgroundTargets(pair).map((s) => s.label)).toEqual([
			'background.scope.span',
			'background.scope.left',
			'background.scope.right',
			'background.scope.every',
		]);
		expect(backgroundTargets(single).map((s) => s.label)).toEqual([
			'background.scope.page',
			'background.scope.every',
		]);
	});

	it("reads what the scope has now, and the first master's for every page", () => {
		const [span, left, right, every] = backgroundTargets(pair);
		expect(currentBackground(book, pair, span!)).toBeNull();
		expect(currentBackground(book, pair, left!)).toBeNull();
		expect(currentBackground(book, pair, right!)?.type).toBe('colour');
		expect(currentBackground(book, pair, every!)).toBeNull();
	});
});

describe('PageBackgroundDialog', () => {
	it('starts on both pages for the spread and applies a colour there at once', async () => {
		const onApply = setup(pair);
		expect(screen.getByRole('radio', { name: 'Both pages' })).toHaveAttribute(
			'aria-checked',
			'true',
		);
		await userEvent.click(screen.getByRole('button', { name: 'Slate' }));
		expect(onApply).toHaveBeenCalledWith({
			type: 'SetBackground',
			target: { type: 'span', spread: 's1' },
			bg: { type: 'colour', colour: '#1f3a5f' },
		});
	});

	it('starts on the page it was asked about, with that page’s colour shown', () => {
		setup(pair, 1);
		expect(screen.getByRole('radio', { name: 'Right page' })).toHaveAttribute(
			'aria-checked',
			'true',
		);
		expect(screen.getByRole('button', { name: 'Cream' })).toHaveAttribute('aria-pressed', 'true');
	});

	it('applies a gradient to the scope chosen, expanded to its stops', async () => {
		const onApply = setup(pair);
		await userEvent.click(screen.getByRole('radio', { name: 'Every page' }));
		await userEvent.click(screen.getByRole('tab', { name: 'Gradient' }));
		await userEvent.click(screen.getByRole('button', { name: 'Dawn' }));
		expect(onApply).toHaveBeenCalledWith({
			type: 'SetBackground',
			target: { type: 'everyPage' },
			bg: { type: 'gradient', angle: 180, stops: FAKE_PAINTS.gradients[0]!.stops },
		});
	});

	it('applies a texture, keeps its tint when another is chosen, and tints it', async () => {
		const onApply = setup(pair);
		await userEvent.click(screen.getByRole('tab', { name: 'Texture' }));
		await userEvent.click(screen.getByRole('button', { name: 'Dot grid' }));
		expect(onApply).toHaveBeenCalledWith({
			type: 'SetBackground',
			target: { type: 'span', spread: 's1' },
			bg: { type: 'texture', id: 'dot-grid', tint: null },
		});
	});

	it('removes the background only when there is one to remove', async () => {
		const onApply = setup(pair, 1);
		await userEvent.click(screen.getByRole('button', { name: 'Remove background' }));
		expect(onApply).toHaveBeenCalledWith({
			type: 'SetBackground',
			target: { type: 'page', spread: 's1', page: 'p2' },
			bg: null,
		});
	});

	it('has nothing to remove on a bare page', () => {
		setup(single);
		expect(screen.getByRole('button', { name: 'Remove background' })).toBeDisabled();
	});
});
