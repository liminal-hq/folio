#!/usr/bin/env bash
# Fails when a dependency has a licence deny.toml does not allow (D-25).
#
# (c) Copyright 2026 Liminal HQ, Scott Morris
# SPDX-License-Identifier: Apache-2.0 OR MIT
set -euo pipefail

cd "$(dirname "$0")/.."
if ! command -v cargo-deny >/dev/null 2>&1; then
  if [ "${CI:-}" = "true" ]; then
    echo "cargo-deny is not installed" >&2
    exit 1
  fi
  echo "cargo-deny is not installed: skipping the licence check (cargo install cargo-deny --locked)"
  exit 0
fi
cargo deny check licenses
