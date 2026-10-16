// How the start screen says when a recovery copy was kept
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

/** "Oct 15, 2026, 12:57 p.m." for a time in seconds since 1970, in the person's own language. */
export function keptWhen(savedAt: number): string {
	return new Date(savedAt * 1000).toLocaleString(undefined, {
		dateStyle: 'medium',
		timeStyle: 'short',
	});
}
