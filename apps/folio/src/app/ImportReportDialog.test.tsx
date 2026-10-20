// Tests for the import report dialog
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
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
});
