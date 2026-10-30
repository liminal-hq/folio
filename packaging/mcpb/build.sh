#!/usr/bin/env bash
# Lays out the Claude Desktop bundle (.mcpb) from release binaries
#
# (c) Copyright 2026 Liminal HQ, Scott Morris
# SPDX-License-Identifier: Apache-2.0 OR MIT

# Usage: packaging/mcpb/build.sh <binaries-dir> <out-dir>
# <binaries-dir> holds `folio-mcp-linux-x64`, `folio-mcp-linux-arm64`, `folio-mcp-win32-x64.exe` and
# `folio-mcp-win32-arm64.exe`. A bundle picks its program by operating system and no further, so the
# bundle's server is a small Node launcher that runs the binary for the computer's processor.
# Leaves the bundle's files in <out-dir>; pack them with `mcpb pack <out-dir>` (the `mcpb` tool
# from https://github.com/anthropics/mcpb ⚠ check its current usage).
set -euo pipefail

if [ "$#" -ne 2 ]; then
  echo "Usage: $0 <binaries-dir> <out-dir>" >&2
  exit 2
fi

here="$(cd "$(dirname "$0")" && pwd)"
mkdir -p "$2/server"
cp "$here/manifest.json" "$2/manifest.json"
cp "$here/launcher.js" "$2/server/launcher.js"
for name in folio-mcp-linux-x64 folio-mcp-linux-arm64 folio-mcp-win32-x64.exe folio-mcp-win32-arm64.exe; do
  cp "$1/$name" "$2/server/$name"
  case "$name" in *.exe) ;; *) chmod +x "$2/server/$name" ;; esac
done
echo "Laid out the bundle in $2"
