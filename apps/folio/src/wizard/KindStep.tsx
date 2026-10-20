// The wizard's first step: what kind of book this is
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { KindInfo } from '@liminal-hq/folio-plugin-books';
import type { KeyboardEvent } from 'react';
import { t } from '../i18n/messages';
import { KindCard } from '../welcome/KindCard';
import styles from './steps.module.css';
import type { WizardDraft } from './wizardDraft';

interface KindStepProps {
	kinds: KindInfo[];
	draft: WizardDraft;
	onChange: (patch: Partial<WizardDraft>) => void;
}

/** Moves the choice with the arrow keys, as in a native radio group. */
function moved(count: number, current: number, event: KeyboardEvent): number | null {
	const forward = event.key === 'ArrowRight' || event.key === 'ArrowDown';
	const back = event.key === 'ArrowLeft' || event.key === 'ArrowUp';
	if (!forward && !back) return null;
	return (current + (forward ? 1 : -1) + count) % count;
}

export function KindStep({ kinds, draft, onChange }: KindStepProps) {
	const chosen = Math.max(
		0,
		kinds.findIndex((info) => info.kind === draft.kind),
	);
	// A different kind has different sizes, so the size starts again from its first.
	const choose = (info: KindInfo) => onChange({ kind: info.kind, trimIndex: 0 });
	return (
		<div
			className={styles.kinds}
			role="radiogroup"
			aria-label={t('welcome.kinds')}
			onKeyDown={(event) => {
				const next = moved(kinds.length, chosen, event);
				const info = next === null ? undefined : kinds[next];
				if (next === null || !info) return;
				event.preventDefault();
				choose(info);
				event.currentTarget.querySelectorAll<HTMLElement>('[role="radio"]')[next]?.focus();
			}}
		>
			{kinds.map((info) => (
				<KindCard
					key={info.kind}
					info={info}
					selected={info.kind === draft.kind}
					onSelect={() => choose(info)}
				/>
			))}
		</div>
	);
}
