#!/usr/bin/env bash
# Check that Folio's release-facing manifest versions stay synchronised
#
# (c) Copyright 2026 Liminal HQ, Scott Morris
# SPDX-License-Identifier: Apache-2.0 OR MIT

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
mode="report"

if [[ "${1:-}" == "--current-version" ]]; then
	mode="current-version"
elif [[ -n "${1:-}" ]]; then
	echo "Usage: $0 [--current-version]" >&2
	exit 2
fi

read_json_version() {
	sed -n 's/^[[:space:]]*"version"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' "$1" | head -n 1
}

read_toml_version() {
	sed -n 's/^version = "\([^"]*\)"/\1/p' "$1" | head -n 1
}

files=(
	"package.json:json"
	"apps/folio/package.json:json"
	"apps/folio/src-tauri/tauri.conf.json:json"
	"apps/folio/src-tauri/Cargo.toml:toml"
	"packaging/npm/folio-mcp/package.json:json"
	"packaging/mcpb/manifest.json:json"
)

reference=""
for spec in "${files[@]}"; do
	path="${spec%%:*}"
	kind="${spec##*:}"
	case "${kind}" in
		json) version="$(read_json_version "${repo_root}/${path}")" ;;
		toml) version="$(read_toml_version "${repo_root}/${path}")" ;;
	esac
	[[ -n "${version}" ]] || { echo "Could not read ${path}" >&2; exit 1; }
	if [[ -z "${reference}" ]]; then reference="${version}"; fi
	if [[ "${version}" != "${reference}" ]]; then
		echo "Release version mismatch: ${path} is ${version}; expected ${reference}" >&2
		exit 1
	fi
done

if [[ "${mode}" == "--current-version" ]]; then
	printf '%s\n' "${reference}"
else
	echo "Release versions are synchronised at ${reference}."
fi
