// Tests following and editing a link from the on-page editor
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { TextPos } from '@liminal-hq/folio-protocol/generated/TextPos';
import { fireEvent, render, screen } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { layoutOf } from './fixtureLayout';
import { setLinkActions } from './linkActions';
import { linkAt } from './caretGeometry';
import { TextSurface } from './TextSurface';

const plain = layoutOf([['hello world']]);
// "hello" (0 to 50 points) is a link; "world" is not.
const layout = {
	...plain,
	lines: plain.lines.map((line) => ({
		...line,
		runs: line.runs.map((r) => (r.text === 'hello' ? { ...r, href: 'https://example.org' } : r)),
	})),
};

function mount() {
	const onEdit = vi.fn(async () => ({ caret: { para: 0, offset: 0 } as TextPos }));
	render(
		<TextSurface
			target={{ type: 'frame', frame: 'f' }}
			frameId="f"
			texts={['hello world']}
			layout={layout}
			box={{ w: 200, h: 100 }}
			at={{ x: 0, y: 0 }}
			scale={1}
			onEdit={onEdit}
			onClose={() => undefined}
		/>,
	);
	return document.querySelector('[data-text-surface]') as HTMLElement;
}

const follow = vi.fn();
const edit = vi.fn();
const remove = vi.fn();

beforeEach(() => {
	follow.mockClear();
	edit.mockClear();
	remove.mockClear();
	setLinkActions({ follow, edit, remove });
	vi.spyOn(HTMLElement.prototype, 'getBoundingClientRect').mockReturnValue({
		left: 0,
		top: 0,
		width: 200,
		height: 100,
		right: 200,
		bottom: 100,
		x: 0,
		y: 0,
		toJSON: () => ({}),
	});
});

afterEach(() =>
	setLinkActions({ follow: () => undefined, edit: () => undefined, remove: () => undefined }),
);

describe('links in the on-page editor', () => {
	it('finds the link under a point, and none beside it', () => {
		expect(linkAt(layout, 10, 5)).toBe('https://example.org');
		expect(linkAt(layout, 80, 5)).toBeUndefined();
	});

	it('follows a link on a Ctrl click, and a plain click only places the caret', () => {
		const surface = mount();
		fireEvent.mouseDown(surface, { clientX: 10, clientY: 5 });
		expect(follow).not.toHaveBeenCalled();
		fireEvent.mouseDown(surface, { clientX: 10, clientY: 5, ctrlKey: true });
		expect(follow).toHaveBeenCalledWith('https://example.org');
		follow.mockClear();
		fireEvent.mouseDown(surface, { clientX: 80, clientY: 5, ctrlKey: true });
		expect(follow).not.toHaveBeenCalled();
	});

	it('says where the link goes while the pointer is over it', () => {
		const surface = mount();
		fireEvent.mouseMove(surface, { clientX: 10, clientY: 5 });
		expect(surface).toHaveAttribute('title', 'https://example.org (Ctrl+click to follow)');
		fireEvent.mouseMove(surface, { clientX: 80, clientY: 5 });
		expect(surface).not.toHaveAttribute('title');
	});

	it('offers to follow, edit or remove the link on a right click', () => {
		const surface = mount();
		fireEvent.contextMenu(surface, { clientX: 10, clientY: 5 });
		fireEvent.click(screen.getByRole('menuitem', { name: 'Follow link' }));
		expect(follow).toHaveBeenCalledWith('https://example.org');
		fireEvent.contextMenu(surface, { clientX: 10, clientY: 5 });
		fireEvent.click(screen.getByRole('menuitem', { name: 'Edit link…' }));
		expect(edit).toHaveBeenCalled();
		fireEvent.contextMenu(surface, { clientX: 10, clientY: 5 });
		fireEvent.click(screen.getByRole('menuitem', { name: 'Remove link' }));
		expect(remove).toHaveBeenCalled();
		// Beside the link there is no menu.
		fireEvent.contextMenu(surface, { clientX: 80, clientY: 5 });
		expect(screen.queryByRole('menuitem')).toBeNull();
	});
});
