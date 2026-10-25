#!/usr/bin/env node
// Runs the folio-mcp binary for this computer, passing its arguments and streams through
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

'use strict';

const { spawn } = require('node:child_process');
const path = require('node:path');

/** The package that holds the binary for each platform. */
const PLATFORMS = {
	'linux-x64': '@liminal-hq/folio-mcp-linux-x64',
	'linux-arm64': '@liminal-hq/folio-mcp-linux-arm64',
	'win32-x64': '@liminal-hq/folio-mcp-win32-x64',
};

/** The binary to run: the one `FOLIO_MCP_BINARY` names, or the platform package's. */
function binary() {
	if (process.env.FOLIO_MCP_BINARY) return process.env.FOLIO_MCP_BINARY;
	const key = `${process.platform}-${process.arch}`;
	const name = PLATFORMS[key];
	if (!name) {
		throw new Error(
			`There is no folio-mcp for ${key}. Folio runs on Linux and Windows 11; set FOLIO_MCP_BINARY to a build of your own.`,
		);
	}
	const file = process.platform === 'win32' ? 'folio-mcp.exe' : 'folio-mcp';
	try {
		return require.resolve(path.posix.join(name, 'bin', file));
	} catch {
		throw new Error(
			`${name} is not installed. Install @liminal-hq/folio-mcp again without --no-optional.`,
		);
	}
}

function main() {
	let program;
	try {
		program = binary();
	} catch (error) {
		process.stderr.write(`folio-mcp: ${error.message}\n`);
		process.exit(1);
	}
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
}

main();
