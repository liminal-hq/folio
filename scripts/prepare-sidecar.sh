#!/usr/bin/env bash
# Builds folio-mcp and puts it where the bundle expects the app's companion program
#
# (c) Copyright 2026 Liminal HQ, Scott Morris
# SPDX-License-Identifier: Apache-2.0 OR MIT

# Tauri bundles an `externalBin` from `binaries/<name>-<target triple>[.exe]`. The bundle installs
# it beside the app as `folio-mcp`, which is where the app looks for it (`agent.rs`).
# Usage: scripts/prepare-sidecar.sh [target-triple]   (the host's, when left out)
set -euo pipefail

cd "$(dirname "$0")/.."
triple="${1:-$(rustc -vV | sed -n 's/^host: //p')}"
cargo build --release -p folio-mcp --target "$triple"
ext=""
case "$triple" in *windows*) ext=".exe" ;; esac
out="apps/folio/src-tauri/binaries"
mkdir -p "$out"
cp "target/$triple/release/folio-mcp$ext" "$out/folio-mcp-$triple$ext"
echo "Put folio-mcp for $triple in $out"
