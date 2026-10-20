// Tests that the textures are the size and base colour the Rust twin draws them with
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { render } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import facts from './fixtures/textures.json';
import { hasTexture, TexturePattern } from './textures';

describe('the textures', () => {
	for (const [id, fact] of Object.entries(facts)) {
		it(`${id} is ${fact.size} points square on ${fact.base}`, () => {
			expect(hasTexture(id)).toBe(true);
			const { container } = render(
				<svg>
					<defs>
						<TexturePattern id="t" texture={id} />
					</defs>
				</svg>,
			);
			const pattern = container.querySelector('pattern')!;
			expect(Number(pattern.getAttribute('width'))).toBe(fact.size);
			expect(Number(pattern.getAttribute('height'))).toBe(fact.size);
			const base = pattern.querySelector('rect');
			if (fact.base === 'none') expect(base).toBeNull();
			else expect(base?.getAttribute('fill')).toBe(fact.base);
		});
	}
});
