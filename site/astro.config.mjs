// Astro configuration for Folio's documentation site, built with Starlight for GitHub Pages
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import starlight from '@astrojs/starlight';
import { defineConfig } from 'astro/config';

// GitHub Pages serves a project site from `/folio`, so `base` defaults to it. A preview or a custom
// domain sets `PAGES_BASE_PATH` (an empty value serves from the root) and `SITE_URL`.
const base = process.env.PAGES_BASE_PATH ?? '/folio';
const site = process.env.SITE_URL ?? 'https://liminal-hq.github.io';

export default defineConfig({
	site,
	base,
	integrations: [
		starlight({
			title: 'Folio',
			description: 'Books, laid out by hand, without the hard parts.',
			logo: { src: './src/assets/folio-mark.svg', alt: '' },
			favicon: '/favicon.svg',
			social: [{ icon: 'github', label: 'GitHub', href: 'https://github.com/liminal-hq/folio' }],
			customCss: ['./src/styles/folio.css'],
			editLink: { baseUrl: 'https://github.com/liminal-hq/folio/edit/main/site/' },
			sidebar: [
				{ label: 'Getting started', autogenerate: { directory: 'manual/getting-started' } },
				{ label: 'Walkthrough', autogenerate: { directory: 'manual/walkthrough' } },
				{ label: 'Using Folio', autogenerate: { directory: 'manual/using-folio' } },
			],
		}),
	],
});
