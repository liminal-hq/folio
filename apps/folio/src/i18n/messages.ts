// The message catalogue: every user-visible string, looked up by key
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

/** English (Canadian) messages. Add a key here, never a literal in a component. */
export const messages = {
	'app.name': 'Folio',
	'window.main.title': 'Folio',
	'main.loading': 'Starting Folio…',
	'main.running': 'Folio {version} is running on {platform}.',
	'main.identifier': 'Application identifier: {identifier}',
	'main.unavailable': 'The application details are not available here.',
} as const;

export type MessageKey = keyof typeof messages;

/** Looks up a message and fills its `{name}` placeholders. */
export function t(key: MessageKey, values: Record<string, string | number> = {}): string {
	return messages[key].replace(/\{(\w+)\}/g, (whole, name: string) =>
		name in values ? String(values[name]) : whole,
	);
}
