// Tests for the Publishing tools: the checklist, the keywords and the royalty calculator
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { Intent } from '@liminal-hq/folio-protocol/generated/Intent';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import type { ComponentType } from 'react';
import { describe, expect, it, vi } from 'vitest';
import { createFakeBooksClient } from '../../books/fakeBooksClient';
import type { ToolProps } from '../tools';
import { ChecklistTool } from './ChecklistTool';
import { KeywordsTool } from './KeywordsTool';
import { RoyaltyTool } from './RoyaltyTool';

async function mount(Tool: ComponentType<ToolProps>, expert = false) {
	const fake = createFakeBooksClient();
	const made = await fake.client.newBook({ title: 'Pip' });
	const apply = vi.fn(async (_intent: Intent) => {});
	const say = vi.fn();
	render(
		<Tool
			client={fake.client}
			id={made.summary.id}
			book={made.book}
			revision={0}
			expert={expert}
			apply={apply}
			say={say}
		/>,
	);
	return { fake, apply, say };
}

describe('ChecklistTool', () => {
	it('says what is done, what a store will still ask for, and copies what the book has', async () => {
		const write = vi.fn(async () => {});
		Object.defineProperty(navigator, 'clipboard', {
			value: { writeText: write },
			configurable: true,
		});
		const { say } = await mount(ChecklistTool);
		expect(await screen.findByText(/2 things a store will still ask for/)).toBeInTheDocument();
		expect(
			screen.getByText('Write the description in Book details; it is the blurb too.'),
		).toBeInTheDocument();
		await userEvent.click(screen.getAllByRole('button', { name: 'Copy' })[0]!);
		expect(write).toHaveBeenCalledWith('Pip');
		expect(say).toHaveBeenCalledWith('Copied the Title.');
	});
});

describe('KeywordsTool', () => {
	it('has seven slots and keeps what is typed as keywords, leaving the empty ones out', async () => {
		const { apply } = await mount(KeywordsTool);
		const boxes = screen.getAllByRole('textbox');
		expect(boxes).toHaveLength(7);
		await userEvent.type(boxes[0]!, 'otter');
		const last = apply.mock.calls.at(-1)![0];
		expect(last).toEqual({ type: 'SetKeywords', keywords: ['otter'] });
		await userEvent.type(boxes[2]!, 'river');
		expect(apply.mock.calls.at(-1)![0]).toEqual({
			type: 'SetKeywords',
			keywords: ['otter', 'river'],
		});
		expect(screen.getAllByText(/\/50/)[0]).toHaveTextContent('5/50');
	});
});

describe('RoyaltyTool', () => {
	it('works out what is earned from the prices and says it is an estimate', async () => {
		const { fake } = await mount(RoyaltyTool);
		expect(
			await screen.findByText(/A copy costs \$1\.29 to print \(24 pages\)/),
		).toBeInTheDocument();
		expect(screen.getByText(/An estimate made 2026-10-21/)).toBeInTheDocument();
		expect(screen.queryByLabelText(/Printing cost on every copy/)).toBeNull();
		const price = screen.getByLabelText(/Price of the printed book/);
		await userEvent.clear(price);
		await userEvent.type(price, '20');
		expect(fake.calls.some((c) => c.startsWith('bookTool(') && c.includes('"listPrice":20'))).toBe(
			true,
		);
	});

	it('lets Expert put in the printing cost KDP’s own calculator gives', async () => {
		await mount(RoyaltyTool, true);
		expect(await screen.findByLabelText(/Printing cost on every copy/)).toBeInTheDocument();
	});
});
