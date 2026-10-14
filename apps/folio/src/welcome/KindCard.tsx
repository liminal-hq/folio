// One kind of book on the welcome screen, drawn as two small pages
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { KindInfo } from '@liminal-hq/folio-plugin-books';
import styles from './KindCard.module.css';
import { KIND_ART } from './kindArt';

interface KindCardProps {
	info: KindInfo;
	selected: boolean;
	onSelect: () => void;
}

/** A choice in a radio group: the group around it handles the keys, this is one of its buttons. */
export function KindCard({ info, selected, onSelect }: KindCardProps) {
	const [left, right] = KIND_ART[info.kind];
	return (
		<button
			type="button"
			role="radio"
			aria-checked={selected}
			tabIndex={selected ? 0 : -1}
			data-kind={info.kind}
			className={styles.card}
			onClick={onSelect}
		>
			<span className={styles.art} aria-hidden="true">
				<span className={styles.page} data-art={left} />
				<span className={styles.page} data-art={right} />
			</span>
			<span className={styles.title}>{info.label}</span>
			<span className={styles.sub}>{info.sub}</span>
		</button>
	);
}
