#!/usr/bin/env bash
# Make the npm packages and the Claude Desktop bundle layout for `folio-mcp` from release binaries
#
# (c) Copyright 2026 Liminal HQ, Scott Morris
# SPDX-License-Identifier: Apache-2.0 OR MIT

# Usage: scripts/package-mcp.sh <version> <binaries-dir> <out-dir>
#
# <binaries-dir> holds `folio-mcp-linux-x64`, `folio-mcp-linux-arm64`, `folio-mcp-win32-x64.exe` and
# `folio-mcp-win32-arm64.exe`.
# <out-dir> gets `npm/*.tgz`, one for each platform package and the launcher, in the order they must
# be published (platforms first), and `mcpb/`, the bundle's files, ready for `mcpb pack`. Sources
# under `packaging/` keep their development version; this sets the release's in copies.
set -euo pipefail

if [ "$#" -ne 3 ]; then
  echo "Usage: $0 <version> <binaries-dir> <out-dir>" >&2
  exit 2
fi
version="$1" bins="$(cd "$2" && pwd)"
mkdir -p "$3"
out="$(cd "$3" && pwd)"
root="$(cd "$(dirname "$0")/.." && pwd)"
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT

mkdir -p "$out/npm"
for platform in linux-x64 linux-arm64 win32-x64 win32-arm64; do
  binary="$bins/folio-mcp-$platform"
  case "$platform" in win32-*) binary="$binary.exe" ;; esac
  node "$root/packaging/npm/make-platform-package.js" "$platform" "$binary" "$work/$platform" "$version"
  (cd "$work/$platform" && npm pack --silent --pack-destination "$out/npm" >/dev/null)
done

# The launcher asks for exactly the version of each platform package.
cp -r "$root/packaging/npm/folio-mcp" "$work/launcher"
jq --arg v "$version" '.version = $v | .optionalDependencies |= with_entries(.value = $v)' \
  "$root/packaging/npm/folio-mcp/package.json" >"$work/launcher/package.json"
(cd "$work/launcher" && npm pack --silent --pack-destination "$out/npm" >/dev/null)

bash "$root/packaging/mcpb/build.sh" "$bins" "$out/mcpb"
jq --arg v "$version" '.version = $v' "$root/packaging/mcpb/manifest.json" >"$out/mcpb/manifest.json"
echo "Packaged folio-mcp $version in $out"
