// Maps a webview label to the kind of window it hosts
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { WindowKind } from '@liminal-hq/folio-protocol/generated/WindowKind';

/** Mirrors `WindowKind::from_label` in `crates/folio-protocol`. */
export function windowKindFromLabel(label: string): WindowKind | null {
	switch (label) {
		case 'main':
			return 'Main';
		case 'about':
			return 'About';
		case 'preferences':
			return 'Preferences';
		case 'book-tools':
			return 'BookTools';
		case 'wizard-new':
			return 'WizardNew';
	}
	return null;
}
