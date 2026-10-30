#!/usr/bin/env node
// Runs the folio-mcp binary in this bundle that matches the computer, passing streams through
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

'use strict';

const { spawn } = require('node:child_process');
const path = require('node:path');

const key = `${process.platform}-${process.arch}`;
const supported = ['linux-x64', 'linux-arm64', 'win32-x64', 'win32-arm64'];
if (!supported.includes(key)) {
	process.stderr.write(
		`folio-mcp: there is no build for ${key}. Folio runs on Linux and Windows 11.\n`,
	);
	process.exit(1);
}
const program = path.join(
	__dirname,
	`folio-mcp-${key}${process.platform === 'win32' ? '.exe' : ''}`,
);
const child = spawn(program, process.argv.slice(2), { stdio: 'inherit' });
for (const signal of ['SIGINT', 'SIGTERM']) {
	process.on(signal, () => child.kill(signal));
}
child.on('error', (error) => {
	process.stderr.write(`folio-mcp: could not run ${program}: ${error.message}\n`);
	process.exit(1);
});
child.on('exit', (code, signal) => {
	if (signal) process.kill(process.pid, signal);
	else process.exit(code ?? 1);
});
