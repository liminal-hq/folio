// Tests for the message catalogue
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { describe, expect, it } from 'vitest';
import { t } from './messages';

describe('t', () => {
	it('returns a plain message', () => {
		expect(t('app.name')).toBe('Folio');
	});

	it('fills placeholders and leaves unknown ones in place', () => {
		expect(t('main.running', { version: '1.0.0', platform: 'linux' })).toBe(
			'Folio 1.0.0 is running on linux.',
		);
		expect(t('main.running', { version: '1.0.0' })).toBe('Folio 1.0.0 is running on {platform}.');
	});
});
