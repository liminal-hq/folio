// The welcome screen's greeting for the time of day
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { MessageKey } from '../i18n/messages';

/** The message to greet with: morning until noon, afternoon until six, evening after. */
export function greetingKey(now: Date): MessageKey {
	const hour = now.getHours();
	if (hour < 12) return 'welcome.greeting.morning';
	if (hour < 18) return 'welcome.greeting.afternoon';
	return 'welcome.greeting.evening';
}
