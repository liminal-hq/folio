// Tests for the import report dialog
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import { ImportReportDialog } from './ImportReportDialog';

describe('ImportReportDialog', () => {
	it('says what came across and what was left out', () => {
		render(
			<ImportReportDialog
				outcome={{
					report: {
						paragraphs: 12,
						headings: 2,
						sceneBreaks: 1,
						notes: ['Pictures were left out.'],
						styles: [],
					},
				}}
				onClose={() => undefined}
			/>,
		);
		expect(
			screen.getByText('Brought in 12 paragraphs, including 2 headings and 1 scene breaks.'),
		).toBeInTheDocument();
		expect(screen.getByText('Pictures were left out.')).toBeInTheDocument();
	});

	it('says why a file could not be brought in', () => {
		render(
			<ImportReportDialog
				outcome={{ problem: 'That file has no words.' }}
				onClose={() => undefined}
			/>,
		);
		expect(screen.getByRole('alert')).toHaveTextContent('That file has no words.');
	});

	it('is not shown without an outcome', () => {
		render(<ImportReportDialog outcome={null} onClose={() => undefined} />);
		expect(screen.queryByRole('dialog')).toBeNull();
	});

	it('lists the document styles and brings the file in again with the changes', async () => {
		const onRemap = vi.fn();
		render(
			<ImportReportDialog
				outcome={{
					report: {
						paragraphs: 3,
						headings: 1,
						sceneBreaks: 0,
						notes: [],
						styles: [
							{ name: 'Heading 1', count: 1, sample: 'The kettle', folio: 'h1' },
							{ name: 'Caption', count: 2, sample: 'A picture', folio: null },
						],
					},
				}}
				onClose={() => undefined}
				onRemap={onRemap}
			/>,
		);
		expect(screen.queryByRole('button', { name: 'Bring it in again' })).toBeNull();
		const caption = screen.getByRole('combobox', { name: 'What Caption becomes' });
		expect(caption).toHaveValue('');
		await userEvent.selectOptions(caption, 'epigraph');
		await userEvent.click(screen.getByRole('button', { name: 'Bring it in again' }));
		expect(onRemap).toHaveBeenCalledWith({ 'Heading 1': 'h1', Caption: 'epigraph' });
	});
});
