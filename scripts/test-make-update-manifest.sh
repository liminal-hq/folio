#!/usr/bin/env bash
# Check make-update-manifest.sh lists every kind of installer under the key the updater looks for
#
# (c) Copyright 2026 Liminal HQ, Scott Morris
# SPDX-License-Identifier: Apache-2.0 OR MIT

set -euo pipefail

cd "$(dirname "$0")/.."
dir="$(mktemp -d)"
trap 'rm -rf "$dir"' EXIT

signed=(
  Folio_0.1.0_amd64.deb Folio_0.1.0_arm64.deb Folio-0.1.0-1.x86_64.rpm
  Folio-0.1.0-x86_64.AppImage Folio-0.1.0-aarch64.AppImage
  Folio_0.1.0_x64-setup.exe Folio_0.1.0_arm64-setup.exe Folio_0.1.0_x64_en-US.msi
)
for f in "${signed[@]}"; do
  echo bytes >"$dir/$f"
  printf 'SIG %s\n' "$f" >"$dir/$f.sig"
done
echo ignored >"$dir/SHA256SUMS"

bash scripts/make-update-manifest.sh "$dir" v0.1.0 liminal-hq/folio "Notes here" >/dev/null

expect() {
  local got
  got="$(jq -r --arg key "$1" '.platforms[$key].url // empty' "$dir/latest.json")"
  [ "$got" = "https://github.com/liminal-hq/folio/releases/download/v0.1.0/$2" ] || {
    echo "FAIL: $1 is '$got', expected $2" >&2
    exit 1
  }
}
expect linux-x86_64-deb Folio_0.1.0_amd64.deb
expect linux-aarch64-deb Folio_0.1.0_arm64.deb
expect linux-x86_64-rpm Folio-0.1.0-1.x86_64.rpm
expect appimage-x86_64 Folio-0.1.0-x86_64.AppImage
expect appimage-aarch64 Folio-0.1.0-aarch64.AppImage
expect windows-x86_64-nsis Folio_0.1.0_x64-setup.exe
expect windows-x86_64 Folio_0.1.0_x64-setup.exe
expect windows-aarch64 Folio_0.1.0_arm64-setup.exe
expect windows-x86_64-msi Folio_0.1.0_x64_en-US.msi
[ "$(jq -r .version "$dir/latest.json")" = 0.1.0 ] || { echo "FAIL: version" >&2; exit 1; }
[ "$(jq -r .notes "$dir/latest.json")" = "Notes here" ] || { echo "FAIL: notes" >&2; exit 1; }
[ "$(jq -r '.platforms["appimage-x86_64"].signature' "$dir/latest.json")" = "SIG Folio-0.1.0-x86_64.AppImage" ] || { echo "FAIL: signature" >&2; exit 1; }

# An installer without a signature stops the release rather than leaving a hole in the manifest.
rm "$dir/Folio_0.1.0_amd64.deb.sig"
if bash scripts/make-update-manifest.sh "$dir" v0.1.0 liminal-hq/folio >/dev/null 2>&1; then
  echo "FAIL: an unsigned installer was listed" >&2
  exit 1
fi
echo "make-update-manifest.sh: ok"
