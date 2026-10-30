#!/usr/bin/env bash
# Make a sample book of every kind as the EPUBs Folio exports for Kindle and run them through Amazon's converter
#
# (c) Copyright 2026 Liminal HQ, Scott Morris
# SPDX-License-Identifier: Apache-2.0 OR MIT

# Needs `scripts/kindle-previewer.sh setup` done first (see `docs/kindle-previewer.md`). It is not part
# of `bun run validate`: the converter is a 900 MB download and runs under Wine on Linux.
set -euo pipefail

cd "$(git rev-parse --show-toplevel)"
out="$(mktemp -d)"
trap 'rm -rf "$out"' EXIT

cargo build -q -p folio-export --example sample_epub
sample=target/debug/examples/sample_epub
books=()
for kind in novel picture textbook illustrated poetry memoir; do
  "$sample" "$kind" "$out/$kind-kindle.epub" >/dev/null
  books+=("$out/$kind-kindle.epub")
done
for kind in picture textbook illustrated; do
  "$sample" "$kind" "$out/$kind-fixed.epub" fixed >/dev/null
  books+=("$out/$kind-fixed.epub")
done
scripts/kindle-previewer.sh check "${books[@]}"
