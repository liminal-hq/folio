// Reads a colour the way the Rust model does: `#rgb` or `#rrggbb`, stored as lower-case `#rrggbb`
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

/** The stored form of a typed colour, or `null` if it is not one. A `#` is optional when typing. */
export function parseColour(text: string): string | null {
	const typed = text.trim();
	const digits = typed.startsWith('#') ? typed.slice(1) : typed;
	if (!/^[0-9a-fA-F]+$/.test(digits)) return null;
	if (digits.length === 6) return `#${digits.toLowerCase()}`;
	if (digits.length === 3) {
		return `#${[...digits].map((c) => c + c).join('')}`.toLowerCase();
	}
	return null;
}
