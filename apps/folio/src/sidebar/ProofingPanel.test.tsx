// Tests for the Proofing group: the list of misspelt words and what can be done with each
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { BookSpelling } from '@liminal-hq/folio-protocol/generated/BookSpelling';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import { ProofingPanel, type ProofingPanelProps } from './ProofingPanel';

const word = (extra: object = {}) => ({
	spread: 's1',
	frame: 'f1',
	row: null,
	col: null,
	word: 'ottr',
	start: 8,
	end: 12,
	context: 'A small ottr named Pip',
	replaceable: true,
	...extra,
});

const found = (words: object[], extra: Partial<BookSpelling> = {}): BookSpelling => ({
	supported: true,
	dictionary: 'en-CA',
	offer: null,
	truncated: false,
	words: words as BookSpelling['words'],
	...extra,
});

const spreads = [
	{ id: 's0', label: 'Cover' },
	{ id: 's1', label: 'Pages 1–2' },
	{ id: 's2', label: 'Pages 3–4' },
];

function setup(result: BookSpelling | null, extra: Partial<ProofingPanelProps> = {}) {
	const handlers = {
		onShow: vi.fn(),
		onIgnore: vi.fn(),
		onAdd: vi.fn(),
		onSuggest: vi.fn(async () => ['otter', 'utter']),
		onChange: vi.fn(),
		onDownload: vi.fn(),
	};
	render(<ProofingPanel found={result} spreads={spreads} {...handlers} {...extra} />);
	return handlers;
}

describe('ProofingPanel', () => {
	it('says it is checking until it has an answer', () => {
		setup(null);
		expect(screen.getByText('Checking the book…')).toBeInTheDocument();
	});

	it('says there is nothing to fix, and counts what there is', () => {
		const { unmount } = render(
			<ProofingPanel
				found={found([])}
				spreads={spreads}
				onShow={vi.fn()}
				onIgnore={vi.fn()}
				onAdd={vi.fn()}
				onSuggest={vi.fn()}
				onChange={vi.fn()}
			/>,
		);
		expect(screen.getByRole('status')).toHaveTextContent('No misspelt words.');
		unmount();
		setup(found([word(), word({ word: 'zorb', start: 20, end: 24 })]));
		expect(screen.getByRole('status')).toHaveTextContent('2 misspelt');
	});

	it('groups the words by the spread they are on, in the shelf’s order', () => {
		setup(found([word({ spread: 's2', word: 'zorb' }), word()]));
		const headings = screen.getAllByRole('heading').map((h) => h.textContent);
		expect(headings).toEqual(['Pages 1–2', 'Pages 3–4']);
	});

	it('shows a word where it is, ignores it, and adds it to the dictionary', async () => {
		const h = setup(found([word()]));
		await userEvent.click(screen.getByRole('button', { name: 'Show ottr on Pages 1–2' }));
		expect(h.onShow).toHaveBeenCalledWith(expect.objectContaining({ frame: 'f1', word: 'ottr' }));
		await userEvent.click(screen.getByRole('button', { name: 'Ignore ottr in this book' }));
		expect(h.onIgnore).toHaveBeenCalledWith(expect.objectContaining({ word: 'ottr' }));
		await userEvent.click(screen.getByRole('button', { name: 'Add ottr to the dictionary' }));
		expect(h.onAdd).toHaveBeenCalledWith(expect.objectContaining({ word: 'ottr' }));
	});

	it('offers suggestions and puts the chosen one in', async () => {
		const h = setup(found([word()]));
		const button = screen.getByRole('button', { name: 'Suggestions for ottr' });
		await userEvent.click(button);
		expect(h.onSuggest).toHaveBeenCalledWith('ottr');
		expect(button).toHaveAttribute('aria-expanded', 'true');
		await userEvent.click(await screen.findByRole('button', { name: 'otter' }));
		expect(h.onChange).toHaveBeenCalledWith(expect.objectContaining({ word: 'ottr' }), 'otter');
		expect(screen.queryByRole('button', { name: 'otter' })).toBeNull();
	});

	it('sends a word that cannot be changed from here to the page', async () => {
		const h = setup(found([word({ replaceable: false })]));
		await userEvent.click(screen.getByRole('button', { name: 'Suggestions for ottr' }));
		expect(await screen.findByText('Change this one on the page.')).toBeInTheDocument();
		expect(screen.queryByRole('button', { name: 'otter' })).toBeNull();
		expect(h.onChange).not.toHaveBeenCalled();
	});

	it('says when a language has no dictionary and offers the one that could be downloaded', async () => {
		const h = setup(
			found([], {
				supported: false,
				dictionary: null,
				offer: { id: 'fr', name: 'French', bytes: 1 },
			}),
		);
		expect(screen.getByText(/no dictionary for this book’s language/)).toBeInTheDocument();
		await userEvent.click(screen.getByRole('button', { name: 'Download the French dictionary' }));
		expect(h.onDownload).toHaveBeenCalledWith('fr');
	});

	it('offers a better dictionary alongside the list', async () => {
		const h = setup(
			found([word()], { offer: { id: 'en-GB', name: 'English (United Kingdom)', bytes: 1 } }),
		);
		await userEvent.click(
			screen.getByRole('button', {
				name: 'A better dictionary for this book: download English (United Kingdom)',
			}),
		);
		expect(h.onDownload).toHaveBeenCalledWith('en-GB');
	});

	it('lists the words the book ignores, and checks one again when it is restored', async () => {
		const onRestore = vi.fn();
		setup(found([]), { ignored: ['Pip', 'reed'], onRestore });
		const section = screen.getByRole('region', { name: 'Ignored in this book' });
		expect(section).toHaveTextContent('Pip');
		expect(section).toHaveTextContent('reed');
		await userEvent.click(screen.getByRole('button', { name: 'Stop ignoring reed' }));
		expect(onRestore).toHaveBeenCalledWith('reed');
	});

	it('draws no ignored list when nothing is ignored', () => {
		setup(found([]), { ignored: [], onRestore: vi.fn() });
		expect(screen.queryByRole('region', { name: 'Ignored in this book' })).toBeNull();
	});

	it('still lists the ignored words for a book with no dictionary', () => {
		setup(found([], { supported: false, dictionary: null }), {
			ignored: ['Pip'],
			onRestore: vi.fn(),
		});
		expect(screen.getByRole('region', { name: 'Ignored in this book' })).toBeInTheDocument();
	});

	it('offers the languages a book can be in, and changes it', async () => {
		const onLanguage = vi.fn();
		setup(found([]), {
			languages: {
				choices: [
					{ value: 'en-CA', label: 'English (Canada)' },
					{ value: 'fr', label: 'French' },
				],
				selected: 'en-CA',
			},
			onLanguage,
		});
		const select = screen.getByRole('combobox', { name: 'Language of the book' });
		expect(select).toHaveValue('en-CA');
		await userEvent.selectOptions(select, 'French');
		expect(onLanguage).toHaveBeenCalledWith('fr');
	});
});
