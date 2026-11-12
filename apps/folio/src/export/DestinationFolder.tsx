// Where the files go: the folder in use, a way to choose another and a way back to the usual one
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { t } from '../i18n/messages';
import styles from './DestinationFolder.module.css';

interface DestinationFolderProps {
	/** The folder in use as the book keeps it (relative or full), or `undefined` for the usual one. */
	folder: string | undefined;
	/** The full path of `folder`, to show and to start the picker at. */
	shown: string | undefined;
	onChoose(): void;
	onUsual(): void;
	disabled?: boolean;
}

export function DestinationFolder({
	folder,
	shown,
	onChoose,
	onUsual,
	disabled,
}: DestinationFolderProps) {
	return (
		<section className={styles.folder} aria-label={t('export.folder')}>
			<span className={styles.label}>{t('export.folder')}</span>
			<span className={styles.path} title={shown}>
				{folder === undefined ? t('export.folder.usual') : (shown ?? folder)}
			</span>
			<span className={styles.actions}>
				<button type="button" className={styles.button} disabled={disabled} onClick={onChoose}>
					{t('export.folder.choose')}
				</button>
				{folder !== undefined && (
					<button type="button" className={styles.button} disabled={disabled} onClick={onUsual}>
						{t('export.folder.back')}
					</button>
				)}
			</span>
		</section>
	);
}
