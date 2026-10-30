#!/usr/bin/env node
// Makes the npm package that holds the folio-mcp binary for one platform
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

// Usage: node make-platform-package.js <linux-x64|linux-arm64|win32-x64> <binary> <out-dir> [version]
// The package goes in <out-dir>, ready for `npm publish`.

'use strict';

const fs = require('node:fs');
const path = require('node:path');

const [platform, binary, out, version = '0.1.0'] = process.argv.slice(2);
const known = ['linux-x64', 'linux-arm64', 'win32-x64', 'win32-arm64'];
if (!known.includes(platform) || !binary || !out) {
	process.stderr.write(
		`Usage: make-platform-package.js <${known.join('|')}> <binary> <out-dir> [version]\n`,
	);
	process.exit(2);
}
const [os, cpu] = platform.split('-');
const file = os === 'win32' ? 'folio-mcp.exe' : 'folio-mcp';
fs.mkdirSync(path.join(out, 'bin'), { recursive: true });
fs.copyFileSync(binary, path.join(out, 'bin', file));
if (os !== 'win32') fs.chmodSync(path.join(out, 'bin', file), 0o755);
fs.writeFileSync(
	path.join(out, 'package.json'),
	JSON.stringify(
		{
			name: `@liminal-hq/folio-mcp-${platform}`,
			version,
			description: `The folio-mcp binary for ${platform}`,
			license: 'Apache-2.0 OR MIT',
			repository: { type: 'git', url: 'git+https://github.com/liminal-hq/folio.git' },
			os: [os],
			cpu: [cpu],
			files: ['bin'],
		},
		null,
		'\t',
	) + '\n',
);
