#!/usr/bin/env bash
# Write the `latest.json` the app's updater reads, from the signed installers in a folder
#
# (c) Copyright 2026 Liminal HQ, Scott Morris
# SPDX-License-Identifier: Apache-2.0 OR MIT

# Usage: scripts/make-update-manifest.sh <assets-dir> <tag> <repository> [notes]
#
# Tauri builds a `.sig` beside each installer it makes for the updater. The updater looks a release up
# as `<os>-<arch>-<bundle type>` (the kind of package the running app was installed from), then
# `<os>-<arch>`, so each installer is listed under its own bundle type. A Windows NSIS installer is
# also the plain `windows-<arch>` entry. The AppImage is listed as `appimage-<arch>`, not under
# `linux`: it is repackaged from the `.deb`, so the app inside reports itself as a `.deb` install and
# would find the `linux-<arch>-deb` entry first. An app that is running as an AppImage asks for the
# target `appimage` instead and installs the download itself (see `updates.rs`).
# Fails if an installer has no `.sig` or if nothing at all can be listed.
set -euo pipefail

if [ "$#" -lt 3 ]; then
  echo "Usage: $0 <assets-dir> <tag> <repository> [notes]" >&2
  exit 2
fi
dir="$1" tag="$2" repo="$3" notes="${4:-}"
version="${tag#v}"
base="https://github.com/${repo}/releases/download/${tag}"

arch_of() {
  case "$1" in
    *x86_64* | *amd64* | *x64*) echo x86_64 ;;
    *aarch64* | *arm64*) echo aarch64 ;;
    *) return 1 ;;
  esac
}

platforms='{}'
add() {
  local key="$1" file="$2"
  platforms="$(jq --arg key "$key" --arg url "$base/$file" --rawfile sig "$dir/$file.sig" \
    '. + {($key): {signature: ($sig | rtrimstr("\n")), url: $url}}' <<<"$platforms")"
}

for path in "$dir"/*; do
  file="$(basename "$path")"
  case "$file" in
    *.deb) os=linux kind=deb ;;
    *.rpm) os=linux kind=rpm ;;
    *.AppImage) os=appimage kind=appimage ;;
    *-setup.exe) os=windows kind=nsis ;;
    *.msi) os=windows kind=msi ;;
    *) continue ;;
  esac
  [ -f "$path.sig" ] || { echo "make-update-manifest: $file has no .sig" >&2; exit 1; }
  arch="$(arch_of "$file")" || { echo "make-update-manifest: cannot tell the architecture of $file" >&2; exit 1; }
  if [ "$kind" = appimage ]; then add "$os-$arch" "$file"; continue; fi
  add "$os-$arch-$kind" "$file"
  if [ "$kind" = nsis ]; then add "$os-$arch" "$file"; fi
done

if [ "$platforms" = '{}' ]; then
  echo "make-update-manifest: no installers to list in $dir" >&2
  exit 1
fi

jq -n --arg version "$version" --arg notes "$notes" --arg date "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
  --argjson platforms "$platforms" \
  '{version: $version, notes: $notes, pub_date: $date, platforms: $platforms}' >"$dir/latest.json"
echo "Listed $(jq '.platforms | length' "$dir/latest.json") updater targets in $dir/latest.json"
