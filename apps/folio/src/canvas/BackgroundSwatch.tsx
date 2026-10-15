// A small sample of a background, to show what a page has
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { Background } from '@liminal-hq/folio-protocol/generated/Background';
import { useId } from 'react';
import { hasTexture, TexturePattern } from './textures';
import styles from './BackgroundSwatch.module.css';

export const SWATCH_WIDTH = 40;
export const SWATCH_HEIGHT = 28;

/** A background as a CSS `background`, or `null` for the kinds that are drawn with SVG instead. */
function cssOf(bg: Background, images: Record<string, string>): string | null {
	if (bg.type === 'colour') return bg.colour;
	if (bg.type === 'gradient') {
		return `linear-gradient(${bg.angle}deg, ${bg.stops.map((s) => `${s.colour} ${s.at * 100}%`).join(', ')})`;
	}
	if (bg.type === 'picture') {
		const url = images[bg.asset];
		return url ? `center / ${bg.fit === 'fill' ? 'cover' : 'contain'} no-repeat url(${url})` : null;
	}
	return null;
}

/**
 * What a page's background looks like, small. `bg` of `null` is the bare paper, drawn as an empty
 * outline so "none" can be told from a white page. It is decoration: whatever shows it says what it
 * is in words.
 */
export function BackgroundSwatch({
	bg,
	images = {},
}: {
	bg: Background | null;
	images?: Record<string, string>;
}) {
	const id = `swatch-${useId().replace(/[^A-Za-z0-9_-]/g, '')}`;
	const css = bg ? cssOf(bg, images) : null;
	return (
		<span className={styles.swatch} data-empty={bg === null} aria-hidden="true">
			{css && <span className={styles.fill} style={{ background: css }} />}
			{bg?.type === 'texture' && hasTexture(bg.id) && (
				<svg width={SWATCH_WIDTH} height={SWATCH_HEIGHT} className={styles.fill}>
					<defs>
						<TexturePattern id={id} texture={bg.id} tint={bg.tint} simple />
					</defs>
					<rect width={SWATCH_WIDTH} height={SWATCH_HEIGHT} fill={`url(#${id})`} />
				</svg>
			)}
		</span>
	);
}
