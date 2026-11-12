// Tests that the Paragraph section gives the focus back to the words being edited
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { Frame } from '@liminal-hq/folio-protocol/generated/Frame';
import { fireEvent, render, screen } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { setOpenEditor, type OpenEditor } from '../text/openEditor';
import { ParagraphSection } from './ParagraphSection';

const frame = {
	type: 'text',
	id: 't',
	box: {},
	locked: false,
	style: 'body',
	content: { inline: [{ text: 'One\nTwo', style: null, bold: false, italic: false }] },
} as unknown as Frame;

function setup() {
	const apply = vi.fn();
	const focus = vi.fn();
	const editor: OpenEditor = {
		copy: async () => false,
		cut: async () => false,
		paste: async () => false,
		focus,
	};
	setOpenEditor(editor);
	render(
		<ParagraphSection
			selected={[frame]}
			layout={null}
			choices={[]}
			paragraphs={{ target: { type: 'frame', frame: 't' }, frame: 't', from: 1, to: 1, looks: [] }}
			units="inches"
			apply={apply}
		/>,
	);
	return { apply, focus, editor };
}

afterEach(() => setOpenEditor(null));

describe('the Paragraph section with the editor open', () => {
	it('keeps the editor open while a field has the focus', () => {
		setup();
		expect(screen.getByRole('group', { name: 'Paragraph' })).toHaveAttribute('data-keeps-editor');
	});

	it('sets the paragraph under the caret and gives the words the focus back on Enter', () => {
		const { apply, focus } = setup();
		const field = screen.getByRole('textbox', { name: /^Space before/ });
		fireEvent.change(field, { target: { value: '0.5' } });
		fireEvent.keyDown(field, { key: 'Enter' });
		expect(apply).toHaveBeenCalledWith({
			type: 'SetParagraphLook',
			target: { type: 'frame', frame: 't' },
			from: 1,
			to: 1,
			change: { set: { spaceBefore: 36 }, clear: [] },
		});
		expect(focus).toHaveBeenCalledTimes(1);
	});

	it('gives the focus back once a line spacing is chosen', () => {
		const { focus } = setup();
		fireEvent.change(screen.getByRole('combobox'), { target: { value: '1.5' } });
		expect(focus).toHaveBeenCalledTimes(1);
	});
});
