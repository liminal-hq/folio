#!/usr/bin/env bash
# Lists the crates and directories that export ts-rs bindings
#
# (c) Copyright 2026 Liminal HQ, Scott Morris
# SPDX-License-Identifier: Apache-2.0 OR MIT

# Sourced by `generate-protocol-bindings.sh` and `check-bindings-drift.sh`. Add a directory to DIRS
# and its crate to CRATES whenever a crate or plugin starts exporting bindings. A domain plugin's
# wire types go to the protocol's generated folder, so it is in CRATES only.
DIRS=()
CRATES=()
