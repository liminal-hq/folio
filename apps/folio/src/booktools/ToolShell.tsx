// The Book tools window's body: an area switcher, that area's tools, and the tool being used
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { useState } from 'react';
import { t } from '../i18n/messages';
import styles from './ToolShell.module.css';
import { toolsIn } from './registry';
import { AREAS, LATER_AREAS, type ToolArea, type ToolProps } from './tools';

interface ToolShellProps {
	area: ToolArea;
	onArea(area: ToolArea): void;
	/** What the tools work on; `null` when no book is open. */
	work: Omit<ToolProps, 'say'> | null;
	/** The title and kind of the book, for the line under the switcher. */
	about: string | null;
}

export function ToolShell({ area, onArea, work, about }: ToolShellProps) {
	const tools = toolsIn(area);
	const [chosen, setChosen] = useState<Record<string, string>>({});
	const [message, setMessage] = useState('');
	const current = tools.find((tool) => tool.id === chosen[area]) ?? tools[0] ?? null;

	return (
		<div className={styles.shell}>
			<nav className={styles.side} aria-label={t('tools.nav')}>
				<div className={styles.areas} role="tablist" aria-label={t('tools.areas')}>
					{AREAS.map((name) => (
						<button
							key={name}
							type="button"
							role="tab"
							aria-selected={name === area}
							className={styles.area}
							onClick={() => onArea(name)}
						>
							{t(`tools.area.${name}`)}
						</button>
					))}
					{LATER_AREAS.map((name) => (
						<button
							key={name}
							type="button"
							role="tab"
							aria-selected={false}
							disabled
							title={t('tools.later')}
							className={styles.area}
						>
							{t(`tools.area.${name}`)}
						</button>
					))}
				</div>
				{about && <p className={styles.about}>{about}</p>}
				<ul className={styles.tools}>
					{tools.map((tool) => (
						<li key={tool.id}>
							<button
								type="button"
								className={styles.tool}
								aria-current={tool.id === current?.id ? 'true' : undefined}
								onClick={() => setChosen({ ...chosen, [area]: tool.id })}
							>
								{t(tool.titleKey)}
							</button>
						</li>
					))}
				</ul>
			</nav>
			<main className={styles.main}>
				{work === null ? (
					<p className={styles.empty}>{t('tools.noBook')}</p>
				) : current === null ? (
					<p className={styles.empty}>{t('tools.noTools')}</p>
				) : (
					<>
						<header className={styles.header}>
							<h1 className={styles.title}>{t(current.titleKey)}</h1>
							<p className={styles.desc}>{t(current.descKey)}</p>
						</header>
						<div className={styles.body}>
							<current.Component {...work} say={setMessage} />
						</div>
					</>
				)}
				<p className={styles.status} role="status">
					{message}
				</p>
			</main>
		</div>
	);
}
