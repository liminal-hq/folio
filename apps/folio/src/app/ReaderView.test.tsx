// Tests for the reader preview: the pages as a Kindle or Kobo shows them, and what it is built from
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { Book } from '@liminal-hq/folio-protocol/generated/Book';
import type { ReaderPreview } from '@liminal-hq/folio-protocol/generated/ReaderPreview';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it } from 'vitest';
import { createFakeBooksClient } from '../books/fakeBooksClient';
import { DEVICES, pageCount, readerDocument, TEXT_STEPS } from './readerDocument';
import { ReaderView } from './ReaderView';

const preview: ReaderPreview = {
	title: 'Pip',
	language: 'en-CA',
	css: '.body { text-indent: 1em }',
	chapters: [
		{
			title: 'One',
			body: '<h1>One</h1><p class="body">Down by the reeds.</p><img src="../images/image-001.png" alt=""/>',
		},
		{ title: 'Two', body: '<h1>Two</h1><p>Later.</p>' },
	],
	images: [{ name: 'image-001.png', dataUri: 'data:image/png;base64,AAAA' }],
};

describe('readerDocument', () => {
	const make = (extra = {}) =>
		readerDocument({
			preview,
			faces: '@font-face { font-family: X; }',
			scale: 1,
			theme: 'white',
			readerFont: false,
			...extra,
		});

	it("is the EPUB's chapters and CSS in columns one screen wide, with the pictures as addresses", () => {
		const html = make();
		expect(html).toContain('<h1>One</h1>');
		expect(html).toContain('<h1>Two</h1>');
		expect(html).toContain('.body { text-indent: 1em }');
		expect(html).toContain('@font-face { font-family: X; }');
		expect(html).toContain('column-width: calc(100vw - 56px)');
		expect(html).toContain('src="data:image/png;base64,AAAA"');
		expect(html).not.toContain('../images/');
	});

	it('sets the text size, the page colour and the reader font', () => {
		expect(make({ scale: 1.25 })).toContain('font-size: 22.50px');
		expect(make({ theme: 'sepia' })).toContain('#f4ecd8');
		expect(make({ theme: 'dark' })).toContain('#16151a');
		expect(make({ readerFont: true })).toContain("font-family: 'Inter', sans-serif !important");
		expect(make()).not.toContain('!important');
	});

	it('steps the text about 12% at a time and counts the pages by their width', () => {
		for (let i = 2; i < 5; i++) {
			const ratio = TEXT_STEPS[i + 1]! / TEXT_STEPS[i]!;
			expect(ratio).toBeGreaterThan(1.09);
			expect(ratio).toBeLessThan(1.15);
		}
		expect(pageCount(618, 618)).toBe(1);
		expect(pageCount(620, 618)).toBe(2);
		expect(pageCount(1854, 618)).toBe(3);
		expect(pageCount(0, 618)).toBe(1);
	});

	it('knows the two devices by their panels', () => {
		expect(DEVICES.kindle.panel).toEqual({ width: 1236, height: 1648 });
		expect(DEVICES.kobo.panel).toEqual({ width: 1072, height: 1448 });
	});
});

describe('the reader view', () => {
	const book = (layout: 'flowing' | 'fixed') =>
		({ layout, trim: { w: 400, h: 600 }, spreads: [], masterPages: [] }) as unknown as Book;

	it('shows a flowing book in a device frame, with page, text size and colour controls', async () => {
		const fake = createFakeBooksClient();
		const made = await fake.client.newBook({});
		render(
			<ReaderView
				client={fake.client}
				id={made.summary.id}
				book={book('flowing')}
				revision={0}
				device="kindle"
				spread={null}
				layout={null}
				images={{}}
			/>,
		);
		expect(await screen.findByTitle('The screen')).toBeInTheDocument();
		expect(
			screen.getByRole('region', { name: 'Preview on a Kindle Paperwhite' }),
		).toBeInTheDocument();
		expect(screen.getByRole('button', { name: 'Previous page' })).toBeDisabled();
		expect(screen.getByRole('status')).toHaveTextContent('Page 1 of 1');
		await userEvent.click(screen.getByRole('button', { name: 'Bigger text' }));
		await userEvent.selectOptions(screen.getByRole('combobox', { name: 'Page colour' }), 'sepia');
		expect(fake.calls.filter((c) => c.startsWith('readerPreview(')).length).toBe(1);
		expect(screen.getByText(/A close look at how a reader sets the book/)).toBeInTheDocument();
	});

	it('says readers can zoom but not reflow a fixed-layout book', async () => {
		const fake = createFakeBooksClient();
		const made = await fake.client.newBook({});
		render(
			<ReaderView
				client={fake.client}
				id={made.summary.id}
				book={book('fixed')}
				revision={0}
				device="kobo"
				spread={null}
				layout={null}
				images={{}}
			/>,
		);
		expect(screen.getByRole('region', { name: 'Preview on a Kobo Clara 2E' })).toBeInTheDocument();
		expect(screen.getByText(/zoom a fixed-layout book but cannot reflow it/)).toBeInTheDocument();
		expect(fake.calls.filter((c) => c.startsWith('readerPreview('))).toHaveLength(0);
	});
});
