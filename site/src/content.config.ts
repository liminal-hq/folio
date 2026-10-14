// Declares the documentation content collection Starlight reads
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import { docsLoader } from '@astrojs/starlight/loaders';
import { docsSchema } from '@astrojs/starlight/schema';
import { defineCollection } from 'astro:content';

export const collections = {
	docs: defineCollection({ loader: docsLoader(), schema: docsSchema() }),
};
