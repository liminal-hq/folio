// Tests for the find and replace bar
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { FindMatch } from '@liminal-hq/folio-protocol/generated/FindMatch';
import { fireEvent, render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import { FindBar, countText } from './FindBar';
import { PLAIN_SEARCH, type FindApi } from './useFind';

const match = (from: number): FindMatch => ({
	target: { type: 'frame', frame: 'a' },
	para: 0,
	from,
	to: from + 3,
	spread: 's',
	context: 'cat',
	locked: false,
});

function api(over: Partial<FindApi> = {}): FindApi {
	return {
		open: true,
		asked: 1,
		replacing: false,
		query: 'cat',
		replacement: '',
		options: PLAIN_SEARCH,
		matches: [match(0), match(5), match(9)],
		truncated: false,
		index: 1,
		error: null,
		notice: null,
		focus: null,
		show: vi.fn(),
		close: vi.fn(),
		setQuery: vi.fn(),
		setReplacement: vi.fn(),
		setOptions: vi.fn(),
		step: vi.fn(),
		replaceOne: vi.fn(async () => {}),
		replaceAll: vi.fn(async () => {}),
		...over,
	};
}

describe('countText', () => {
	it('says which match of how many, and when there are none or more', () => {
		expect(countText(api())).toBe('2 of 3');
		expect(countText(api({ matches: [], index: null }))).toBe('No matches');
		expect(countText(api({ query: '' }))).toBe('');
		expect(countText(api({ truncated: true }))).toBe('2 of 3 or more');
		expect(countText(api({ error: 'bad' }))).toBe('');
	});
});

describe('the find bar in Easy', () => {
	it('is a labelled search with the query focused, a count that is announced, and no options', () => {
		render(<FindBar find={api()} expert={false} />);
		expect(screen.getByRole('search', { name: 'Find in the book' })).toBeInTheDocument();
		const field = screen.getByRole('textbox', { name: 'Find' });
		expect(field).toHaveFocus();
		expect(field).toHaveValue('cat');
		const status = screen.getByText('2 of 3');
		expect(status).toHaveAttribute('role', 'status');
		expect(status).toHaveAttribute('aria-live', 'polite');
		expect(screen.queryByRole('button', { name: 'Options' })).not.toBeInTheDocument();
		expect(screen.queryByRole('textbox', { name: 'Replace with' })).not.toBeInTheDocument();
	});

	it('goes to the next and previous match with the buttons, Enter and Shift+Enter', async () => {
		const find = api();
		render(<FindBar find={find} expert={false} />);
		await userEvent.click(screen.getByRole('button', { name: 'Next match' }));
		expect(find.step).toHaveBeenLastCalledWith(false);
		await userEvent.click(screen.getByRole('button', { name: 'Previous match' }));
		expect(find.step).toHaveBeenLastCalledWith(true);
		const field = screen.getByRole('textbox', { name: 'Find' });
		fireEvent.keyDown(field, { key: 'Enter' });
		expect(find.step).toHaveBeenLastCalledWith(false);
		fireEvent.keyDown(field, { key: 'Enter', shiftKey: true });
		expect(find.step).toHaveBeenLastCalledWith(true);
	});

	it('has no matches to step through when there are none', () => {
		render(<FindBar find={api({ matches: [], index: null })} expert={false} />);
		expect(screen.getByRole('button', { name: 'Next match' })).toBeDisabled();
		expect(screen.getByRole('button', { name: 'Previous match' })).toBeDisabled();
	});

	it('sends what is typed, and closes with Escape and the close button', async () => {
		const find = api();
		render(<FindBar find={find} expert={false} />);
		await userEvent.type(screen.getByRole('textbox', { name: 'Find' }), 's');
		expect(find.setQuery).toHaveBeenCalledWith('cats');
		fireEvent.keyDown(screen.getByRole('textbox', { name: 'Find' }), { key: 'Escape' });
		expect(find.close).toHaveBeenCalledTimes(1);
		await userEvent.click(screen.getByRole('button', { name: 'Close find' }));
		expect(find.close).toHaveBeenCalledTimes(2);
	});

	it('shows why a pattern could not be searched in place of the count', () => {
		render(<FindBar find={api({ error: 'That is not a pattern Folio can search for.' })} expert />);
		expect(screen.getByRole('textbox', { name: 'Find' })).toHaveAttribute('aria-invalid', 'true');
		expect(screen.getByText('That is not a pattern Folio can search for.')).toHaveAttribute(
			'role',
			'status',
		);
	});

	it('asks for the replace bar with Ctrl+H and the find bar with Ctrl+F inside the field', () => {
		const find = api();
		render(<FindBar find={find} expert={false} />);
		const field = screen.getByRole('textbox', { name: 'Find' });
		fireEvent.keyDown(field, { key: 'h', ctrlKey: true });
		expect(find.show).toHaveBeenLastCalledWith(true);
		fireEvent.keyDown(field, { key: 'f', ctrlKey: true });
		expect(find.show).toHaveBeenLastCalledWith(false);
	});
});

describe('the replace row', () => {
	it('replaces one or all, and says what was done', async () => {
		const find = api({ replacing: true, replacement: 'dog', notice: 'Replaced 3.' });
		render(<FindBar find={find} expert={false} />);
		expect(
			screen.getByRole('search', { name: 'Find and replace in the book' }),
		).toBeInTheDocument();
		await userEvent.click(screen.getByRole('button', { name: 'Replace' }));
		expect(find.replaceOne).toHaveBeenCalledTimes(1);
		await userEvent.click(screen.getByRole('button', { name: 'Replace all' }));
		expect(find.replaceAll).toHaveBeenCalledTimes(1);
		expect(screen.getByText('Replaced 3.')).toHaveAttribute('role', 'status');
		await userEvent.type(screen.getByRole('textbox', { name: 'Replace with' }), 's');
		expect(find.setReplacement).toHaveBeenCalledWith('dogs');
	});

	it('has nothing to replace when there is no match', () => {
		render(<FindBar find={api({ replacing: true, matches: [], index: null })} expert={false} />);
		expect(screen.getByRole('button', { name: 'Replace' })).toBeDisabled();
		expect(screen.getByRole('button', { name: 'Replace all' })).toBeDisabled();
	});
});

describe('the options in Expert', () => {
	it('are in a row that opens and closes, with match case, whole word and regular expression', async () => {
		const find = api();
		render(<FindBar find={find} expert />);
		const more = screen.getByRole('button', { name: 'Options' });
		expect(more).toHaveAttribute('aria-expanded', 'false');
		expect(screen.queryByRole('checkbox', { name: 'Match case' })).not.toBeInTheDocument();
		await userEvent.click(more);
		expect(more).toHaveAttribute('aria-expanded', 'true');
		await userEvent.click(screen.getByRole('checkbox', { name: 'Whole word' }));
		expect(find.setOptions).toHaveBeenLastCalledWith({ ...PLAIN_SEARCH, wholeWord: true });
		await userEvent.click(screen.getByRole('checkbox', { name: 'Regular expression' }));
		expect(find.setOptions).toHaveBeenLastCalledWith({ ...PLAIN_SEARCH, regex: true });
		await userEvent.click(screen.getByRole('checkbox', { name: 'Match case' }));
		expect(find.setOptions).toHaveBeenLastCalledWith({ ...PLAIN_SEARCH, matchCase: true });
	});
});
