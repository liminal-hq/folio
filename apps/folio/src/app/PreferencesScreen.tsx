// The Preferences window: the shared settings shell over the settings plugin
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { ButtonRow } from '@liminal-hq/folio-chrome/SettingsShell/ButtonRow';
import { SegmentedRow } from '@liminal-hq/folio-chrome/SettingsShell/SegmentedRow';
import { SettingsGroup } from '@liminal-hq/folio-chrome/SettingsShell/SettingsGroup';
import { SettingsSection } from '@liminal-hq/folio-chrome/SettingsShell/SettingsSection';
import { SettingsShell } from '@liminal-hq/folio-chrome/SettingsShell/SettingsShell';
import type { SettingsSectionDef } from '@liminal-hq/folio-chrome/SettingsShell/types';
import { ToggleRow } from '@liminal-hq/folio-chrome/SettingsShell/ToggleRow';
import { WindowFrame } from '@liminal-hq/folio-chrome/WindowFrame';
import { useState } from 'react';
import { t } from '../i18n/messages';
import { useSettings, useSettingsError, useUpdateSettings } from '../settings/useSettings';
import { AppTitleBar } from './AppTitleBar';
import styles from './PreferencesScreen.module.css';

type SectionId = 'interface' | 'appearance' | 'units' | 'help';

function InterfacePage() {
	const settings = useSettings();
	const update = useUpdateSettings();
	const error = useSettingsError();
	const refusal = (field: string) =>
		error?.field === field ? { error: t('preferences.error', { message: error.message }) } : {};
	return (
		<SettingsSection>
			<SettingsGroup>
				<SegmentedRow
					label={t('preferences.mode')}
					description={t('preferences.mode.description')}
					value={settings.mode}
					options={[
						{ value: 'easy', label: t('mode.easy') },
						{ value: 'expert', label: t('mode.expert') },
					]}
					onChange={(mode) => void update({ mode })}
				/>
				<ToggleRow
					label={t('preferences.menuBar')}
					description={t('preferences.menuBar.description')}
					checked={settings.interface.menuBar}
					onChange={(menuBar) => void update({ interface: { menuBar } })}
				/>
				<SegmentedRow
					label={t('preferences.toolbar')}
					value={settings.interface.toolbar}
					options={[
						{ value: 'ribbon', label: t('preferences.toolbar.ribbon') },
						{ value: 'classic', label: t('preferences.toolbar.classic') },
					]}
					onChange={(toolbar) => void update({ interface: { toolbar } })}
				/>
				<ToggleRow
					label={t('preferences.largerControls')}
					description={t('preferences.largerControls.description')}
					checked={settings.interface.largerControls}
					onChange={(largerControls) => void update({ interface: { largerControls } })}
					{...refusal('interface.largerControls')}
				/>
			</SettingsGroup>
		</SettingsSection>
	);
}

function AppearancePage() {
	const settings = useSettings();
	const update = useUpdateSettings();
	return (
		<SettingsSection>
			<SettingsGroup>
				<SegmentedRow
					label={t('preferences.colourMode')}
					value={settings.colourMode}
					options={[
						{ value: 'system', label: t('preferences.colourMode.system') },
						{ value: 'light', label: t('preferences.colourMode.light') },
						{ value: 'dark', label: t('preferences.colourMode.dark') },
					]}
					onChange={(colourMode) => void update({ colourMode })}
				/>
			</SettingsGroup>
		</SettingsSection>
	);
}

function UnitsPage() {
	const settings = useSettings();
	const update = useUpdateSettings();
	return (
		<SettingsSection>
			<SettingsGroup>
				<SegmentedRow
					label={t('preferences.units')}
					description={t('preferences.units.description')}
					value={settings.units}
					options={[
						{ value: 'inches', label: t('preferences.units.inches') },
						{ value: 'millimetres', label: t('preferences.units.millimetres') },
					]}
					onChange={(units) => void update({ units })}
				/>
			</SettingsGroup>
		</SettingsSection>
	);
}

function SpellingPage() {
	const settings = useSettings();
	const update = useUpdateSettings();
	return (
		<SettingsSection>
			<SettingsGroup>
				<ToggleRow
					label={t('preferences.spelling')}
					description={t('preferences.spelling.description')}
					checked={settings.spelling.enabled}
					onChange={(enabled) => void update({ spelling: { enabled } })}
				/>
			</SettingsGroup>
			<SettingsGroup>
				{settings.spelling.personal.length === 0 ? (
					<p className={styles.empty}>{t('preferences.personal.none')}</p>
				) : (
					settings.spelling.personal.map((word) => (
						<ButtonRow
							key={word}
							label={word}
							actionLabel={t('preferences.personal.remove')}
							onAction={() =>
								void update({
									spelling: { personal: settings.spelling.personal.filter((w) => w !== word) },
								})
							}
						/>
					))
				)}
			</SettingsGroup>
		</SettingsSection>
	);
}

function HelpPage() {
	const settings = useSettings();
	const update = useUpdateSettings();
	return (
		<SettingsSection>
			<SettingsGroup>
				<SegmentedRow
					label={t('preferences.jax')}
					description={t('preferences.jax.description')}
					value={settings.jax}
					options={[
						{ value: 'guide', label: t('preferences.jax.guide') },
						{ value: 'tips', label: t('preferences.jax.tips') },
						{ value: 'off', label: t('preferences.jax.off') },
					]}
					onChange={(jax) => void update({ jax })}
				/>
			</SettingsGroup>
		</SettingsSection>
	);
}

function sections(): SettingsSectionDef[] {
	return [
		{ id: 'interface', label: t('preferences.section.interface'), render: () => <InterfacePage /> },
		{
			id: 'appearance',
			label: t('preferences.section.appearance'),
			render: () => <AppearancePage />,
		},
		{ id: 'units', label: t('preferences.section.units'), render: () => <UnitsPage /> },
		{
			id: 'spelling',
			label: t('preferences.section.spelling'),
			render: () => <SpellingPage />,
		},
		{ id: 'help', label: t('preferences.section.help'), render: () => <HelpPage /> },
	];
}

/**
 * Edits the settings. Changes apply at once (there is no Save button): each one is asked of Rust,
 * and a row shows what Rust says is in force.
 */
export function PreferencesScreen() {
	const [active, setActive] = useState<SectionId>('interface');
	return (
		<WindowFrame className={styles.screen}>
			<AppTitleBar title={t('window.preferences.title')} />
			<main className={styles.content}>
				<SettingsShell
					sections={sections()}
					activeId={active}
					onSelect={(id) => setActive(id as SectionId)}
				/>
			</main>
		</WindowFrame>
	);
}
