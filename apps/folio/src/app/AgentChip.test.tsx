// Tests for the status bar line that says an agent changed the book
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { act, render, screen } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { AgentChip } from './AgentChip';

let editedAt: number | null = null;

vi.mock('../books/useBooks', () => ({
	useBooksSelector: (pick: (s: { agentEditedAt: number | null }) => unknown) =>
		pick({ agentEditedAt: editedAt }),
}));

describe('the agent line', () => {
	beforeEach(() => {
		vi.useFakeTimers();
	});
	afterEach(() => {
		vi.useRealTimers();
	});

	it('says nothing until an agent has changed a book', () => {
		editedAt = null;
		render(<AgentChip />);
		expect(screen.queryByRole('status')).toBeNull();
	});

	it('says so after a change, and goes away a few seconds later', () => {
		editedAt = Date.now();
		render(<AgentChip />);
		expect(screen.getByRole('status')).toHaveTextContent('An agent just changed this book');
		act(() => {
			vi.advanceTimersByTime(6500);
		});
		expect(screen.queryByRole('status')).toBeNull();
	});
});
