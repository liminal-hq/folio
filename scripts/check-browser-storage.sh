#!/usr/bin/env bash
# Check the app keeps no state in browser storage
#
# (c) Copyright 2026 Liminal HQ, Scott Morris
# SPDX-License-Identifier: Apache-2.0 OR MIT

# App state (preferences, window geometry, recent books) belongs to the settings plugin, which Rust
# owns. `localStorage`, `sessionStorage` and IndexedDB are per-webview and invisible to the other
# windows, so they are not used in the app or the chrome. Exits 1 and lists offenders; 0 if clean.
set -euo pipefail

cd "$(git rev-parse --show-toplevel 2>/dev/null || dirname "$(dirname "${BASH_SOURCE[0]}")")"

SCAN_DIRS=()
for d in apps packages; do
  [[ -d "$d" ]] && SCAN_DIRS+=("$d")
done

if [ "${#SCAN_DIRS[@]}" -eq 0 ]; then
  echo "No app or package sources to check."
  exit 0
fi

offenders="$(grep -rnE '\b(localStorage|sessionStorage|indexedDB)\b' "${SCAN_DIRS[@]}" \
  --include='*.ts' --include='*.tsx' --include='*.js' --include='*.mjs' \
  --exclude-dir=node_modules --exclude-dir=dist --exclude-dir=generated --exclude-dir=gen \
  --exclude='*.test.ts' --exclude='*.test.tsx' || true)"

if [ -n "$offenders" ]; then
  echo "Browser storage found (see AGENTS.md 'Frontend Code Conventions'). Use the settings plugin:"
  echo "$offenders"
  exit 1
fi

echo "No browser storage in the app or the chrome."
