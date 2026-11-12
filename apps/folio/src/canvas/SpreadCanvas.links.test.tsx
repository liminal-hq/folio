// Tests for the links the canvas draws and follows
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { Book } from '@liminal-hq/folio-protocol/generated/Book';
import type { Spread } from '@liminal-hq/folio-protocol/generated/Spread';
import type { SpreadLayout } from '@liminal-hq/folio-protocol/generated/SpreadLayout';
import { fireEvent, render } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { setLinkActions } from '../text/linkActions';
import { SpreadCanvas } from './SpreadCanvas';

const book = {
	trim: { w: 400, h: 400 },
	bleed: 0,
	margins: { top: 0, bottom: 0, inside: 0, outside: 0 },
	grid: { columns: 1, gutter: 0, baseline: null },
} as unknown as Book;

const spread = {
	id: 's',
	kind: 'single',
	pages: [{ id: 'p', master: null }],
	frames: [
		{
			type: 'text',
			id: 'a',
			box: { x: 20, y: 20, w: 300, h: 100 },
			rotation: 0,
			locked: false,
			style: 'body',
			columns: 1,
		},
	],
	guides: [],
	masterOverrides: [],
} as unknown as Spread;

const run = (text: string, x: number, extra = {}) => ({
	text,
	x,
	width: text.length * 6,
	family: 'Inter',
	size: 12,
	weight: 400,
	italic: false,
	tracking: 0,
	hyphen: false,
	start: 0,
	...extra,
});

const layout = {
	spread: 's',
	preflight: [],
	frames: [
		{
			frame: 'a',
			overset: false,
			lines: [
				{
					x: 0,
					top: 0,
					baseline: 12,
					width: 100,
					height: 14,
					runs: [
						run('Read ', 0),
						run('here', 30, {
							href: 'https://example.org',
							link: { offset: 1.5, thickness: 0.6, width: 24 },
						}),
						run('page', 60, { href: '#spread-2', colour: '#a23b2a' }),
					],
				},
			],
		},
	],
} as unknown as SpreadLayout;

afterEach(() =>
	setLinkActions({ follow: () => undefined, edit: () => undefined, remove: () => undefined }),
);

describe('links on the canvas', () => {
	it('draws a link in the link colour with its rule, and says where it goes', () => {
		render(<SpreadCanvas book={book} spread={spread} layout={layout} zoom={1} label="Page" />);
		const link = document.querySelector('[data-href="https://example.org"]')!;
		expect(link.querySelector('text')).toHaveAttribute('fill', '#2b3a67');
		const rule = link.querySelector('[data-rule="link"]')!;
		expect(rule).toHaveAttribute('width', '24');
		expect(rule).toHaveAttribute('fill', '#2b3a67');
		expect(link.querySelector('title')).toHaveTextContent(
			'https://example.org (Ctrl+click to follow)',
		);
		// Words that are not a link are plain ink, and a coloured link keeps its colour.
		expect(document.querySelectorAll('[data-href]:not([data-href=""])')).toHaveLength(2);
		const plain = [...document.querySelectorAll('text')].find((t) => t.textContent === 'Read ')!;
		expect(plain).toHaveAttribute('fill', '#1c1b20');
		const other = document.querySelector('[data-href="#spread-2"] text')!;
		expect(other).toHaveAttribute('fill', '#a23b2a');
	});

	it('follows a link on a Ctrl click, and only then', () => {
		const follow = vi.fn();
		setLinkActions({ follow, edit: () => undefined, remove: () => undefined });
		render(<SpreadCanvas book={book} spread={spread} layout={layout} zoom={1} label="Page" />);
		const text = document.querySelector('[data-href="https://example.org"] text')!;
		fireEvent.click(text);
		expect(follow).not.toHaveBeenCalled();
		fireEvent.click(text, { ctrlKey: true });
		expect(follow).toHaveBeenCalledWith('https://example.org');
		fireEvent.click(document.querySelector('[data-href="#spread-2"] text')!, { metaKey: true });
		expect(follow).toHaveBeenLastCalledWith('#spread-2');
	});
});
