// What the wizard has been told so far, and the defaults a step starts from
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { Settings } from '@liminal-hq/folio-plugin-settings';
import type { NewBookSpec } from '@liminal-hq/folio-protocol/generated/NewBookSpec';

/** The book the wizard will make, and how the person likes to work: every question has a field. */
export type WizardDraft = NewBookSpec & {
	mode: Settings['mode'];
	jax: Settings['jax'];
};

/** The answers a person gets by pressing Next all the way, matching the quick start. */
export function initialDraft(settings: Pick<Settings, 'mode' | 'jax'>): WizardDraft {
	return {
		mode: settings.mode,
		jax: settings.jax,
		kind: 'picture',
		trimIndex: 0,
		paper: 'white',
		title: '',
		subtitle: '',
		author: '',
		isbn: '',
		words: 'samples',
		coverColour: '#5b2a4a',
		look: 'friendly',
	};
}
