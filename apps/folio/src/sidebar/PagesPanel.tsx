// The Pages group: the book's pages, and the background of the page being shown
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { Background } from '@liminal-hq/folio-protocol/generated/Background';
import type { Book } from '@liminal-hq/folio-protocol/generated/Book';
import type { Intent } from '@liminal-hq/folio-protocol/generated/Intent';
import { t } from '../i18n/messages';
import { BackgroundSwatch } from '../canvas/BackgroundSwatch';
import { resolveBackgrounds } from '../canvas/backgrounds';
import { backgroundTargets, currentBackground } from '../canvas/backgroundScopes';
import styles from './PagesPanel.module.css';

export interface PagesPanelProps {
	book: Book;
	/** A name for each spread, as the shelf has them. */
	labels: string[];
	/** The spread being shown. */
	shown: number;
	onShow(index: number): void;
	/** Opens the background dialog at the place `scope` names. */
	onChange(scope: string): void;
	onApply(intent: Intent): void;
	/** An address to draw for each picture a background uses, once it has been read. */
	images?: Record<string, string>;
}

/** In words, so the swatch is not the only way to tell what a page has. */
export function describe(bg: Background | null): string {
	if (!bg) return t('pages.none');
	return t(`pages.is.${bg.type}`);
}

/**
 * The spreads in order (the cover first), each with a swatch of what each of its pages shows, and
 * under them the places a background can go on the spread being shown, with what each has now.
 */
export function PagesPanel({
	book,
	labels,
	shown,
	onShow,
	onChange,
	onApply,
	images,
}: PagesPanelProps) {
	const spread = book.spreads[shown];
	const scopes = spread ? backgroundTargets(spread) : [];
	return (
		<>
			<section className={styles.section} aria-labelledby="pages-list">
				<h2 id="pages-list" className={styles.heading}>
					{t('pages.list')}
				</h2>
				<ul className={styles.list}>
					{book.spreads.map((s, index) => (
						<li key={s.id}>
							<button
								type="button"
								className={styles.row}
								aria-current={index === shown ? 'true' : undefined}
								onClick={() => onShow(index)}
							>
								<span className={styles.swatches}>
									{resolveBackgrounds(book.masterPages ?? [], s).map((shownBg, i) => (
										<BackgroundSwatch key={i} bg={shownBg?.bg ?? null} images={images} />
									))}
								</span>
								<span className={styles.label}>{labels[index] ?? ''}</span>
							</button>
						</li>
					))}
				</ul>
			</section>
			{spread && (
				<section className={styles.section} aria-labelledby="pages-background">
					<h2 id="pages-background" className={styles.heading}>
						{t('pages.background')}
					</h2>
					<ul className={styles.list}>
						{scopes.map((scope) => {
							const bg = currentBackground(book, spread, scope);
							const place = t(scope.label);
							return (
								<li key={scope.id} className={styles.place}>
									<BackgroundSwatch bg={bg} images={images} />
									<span className={styles.label}>
										<span>{place}</span>
										<span className={styles.is}>{describe(bg)}</span>
									</span>
									<button
										type="button"
										className={styles.action}
										aria-label={t('pages.change', { place })}
										onClick={() => onChange(scope.id)}
									>
										{t('pages.change.short')}
									</button>
									<button
										type="button"
										className={styles.action}
										aria-label={t('pages.remove', { place })}
										disabled={bg === null}
										onClick={() =>
											onApply({ type: 'SetBackground', target: scope.target, bg: null })
										}
									>
										{t('pages.remove.short')}
									</button>
								</li>
							);
						})}
					</ul>
				</section>
			)}
		</>
	);
}
