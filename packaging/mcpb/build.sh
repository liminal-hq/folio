#!/usr/bin/env bash
# Lays out the Claude Desktop bundle (.mcpb) from release binaries
#
# (c) Copyright 2026 Liminal HQ, Scott Morris
# SPDX-License-Identifier: Apache-2.0 OR MIT

# Usage: packaging/mcpb/build.sh <folio-mcp (Linux)> <folio-mcp.exe (Windows)> <out-dir>
# Leaves the bundle's files in <out-dir>; pack them with `mcpb pack <out-dir>` (the `mcpb` tool
# from https://github.com/anthropics/mcpb ⚠ check its current usage).
set -euo pipefail

if [ "$#" -ne 3 ]; then
  echo "Usage: $0 <folio-mcp> <folio-mcp.exe> <out-dir>" >&2
  exit 2
fi

here="$(cd "$(dirname "$0")" && pwd)"
mkdir -p "$3/server"
cp "$here/manifest.json" "$3/manifest.json"
cp "$1" "$3/server/folio-mcp"
cp "$2" "$3/server/folio-mcp.exe"
chmod +x "$3/server/folio-mcp"
echo "Laid out the bundle in $3"
