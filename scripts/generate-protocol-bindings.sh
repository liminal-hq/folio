#!/usr/bin/env bash
# Regenerates every TypeScript binding that the Rust crates and plugins export
#
# (c) Copyright 2026 Liminal HQ, Scott Morris
# SPDX-License-Identifier: Apache-2.0 OR MIT

# Run after changing any `#[ts(export)]`-annotated type. Each type names its own output directory
# with `#[ts(export, export_to = ...)]`, so nothing here needs configuring beyond the crate list.
# Do not hand-edit generated files: the domain bindings live in `packages/protocol/src/generated/`.
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."

CRATES=()
source scripts/binding-crates.sh

if [ "${#CRATES[@]}" -eq 0 ]; then
  echo "No crates export bindings yet."
  exit 0
fi

cargo_args=()
for crate in "${CRATES[@]}"; do cargo_args+=(-p "${crate}"); done
cargo test "${cargo_args[@]}"
echo "Regenerated the protocol bindings."
