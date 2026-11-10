// What an address in a link may be, and what a link opens: the checks the link dialog makes before it sets one
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

/** Why an address was refused, or the address in the form a link keeps it. */
export type HrefCheck =
	{ ok: true; href: string } | { ok: false; reason: 'empty' | 'unsafe' | 'spread' | 'invalid' };

/**
 * Reads what was typed as the address of a link. A web address, a `mailto:` or `tel:` address and
 * `#spread-3` for a spread of the book are kept; typing `example.org` or `www.example.org` makes a
 * web address and `me@example.org` an email address, so nobody has to know about schemes. Anything
 * else with a scheme (`javascript:`, `file:`, `data:`) is refused. `spreads` is how many spreads the
 * book has, which a `#spread-n` link must fall within.
 *
 * The same rules are in `clean_href` in `folio-model`, which has the last word.
 */
export function checkHref(typed: string, spreads: number): HrefCheck {
	const text = typed.trim();
	if (text === '') return { ok: false, reason: 'empty' };
	if (text.length > 2048 || /[\s\u0000-\u001f\u007f]/.test(text))
		return { ok: false, reason: 'invalid' };
	const anchor = /^#spread-(\d+)$/i.exec(text);
	if (anchor) {
		const n = Number(anchor[1]);
		return n >= 1 && n <= spreads
			? { ok: true, href: `#spread-${n}` }
			: { ok: false, reason: 'spread' };
	}
	if (text.startsWith('#')) return { ok: false, reason: 'invalid' };
	const scheme = /^([a-z][a-z0-9+.-]*):/i.exec(text);
	if (scheme) {
		const name = scheme[1]!.toLowerCase();
		const rest = text.slice(scheme[0].length);
		if (name === 'http' || name === 'https') {
			const host = (rest.startsWith('//') ? rest.slice(2) : '')
				.split(/[/?#]/)[0]!
				.split('@')
				.pop()!;
			return host !== '' && !host.startsWith(':')
				? { ok: true, href: `${name}:${rest}` }
				: { ok: false, reason: 'invalid' };
		}
		if (name === 'mailto' || name === 'tel')
			return rest !== '' && !rest.startsWith('?')
				? { ok: true, href: `${name}:${rest}` }
				: { ok: false, reason: 'invalid' };
		// `localhost:8080` and `example.org:8080` are addresses with a port, not schemes.
		if (!/^(localhost|[^/@\s]+\.[a-z]{2,})$/i.test(scheme[1]!))
			return { ok: false, reason: 'unsafe' };
	}
	if (/^[^\s@/]+@[^\s@/]+\.[a-z]{2,}$/i.test(text)) return { ok: true, href: `mailto:${text}` };
	if (
		/^(www\.)?[^\s@/:]+\.[a-z]{2,}([:/?#].*)?$/i.test(text) ||
		/^localhost([:/?#].*)?$/i.test(text)
	)
		return { ok: true, href: `https://${text}` };
	return { ok: false, reason: 'invalid' };
}

/** The spread a `#spread-n` link names, counted from 1. */
export function spreadOfHref(href: string): number | null {
	const match = /^#spread-(\d+)$/.exec(href);
	const n = match ? Number(match[1]) : 0;
	return n >= 1 ? n : null;
}

/** Whether a link goes to a place in the book rather than out of it. */
export const isInternalHref = (href: string): boolean => spreadOfHref(href) !== null;

/** Whether an address is one the system opens for us: web, email or phone. */
export const isExternalHref = (href: string): boolean => /^(https?|mailto|tel):/i.test(href);
