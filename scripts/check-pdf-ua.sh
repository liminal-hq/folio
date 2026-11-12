#!/usr/bin/env bash
# Check that a tagged PDF is tagged, well formed and, where veraPDF is there, PDF/UA-1
#
# (c) Copyright 2026 Liminal HQ, Scott Morris
# SPDX-License-Identifier: Apache-2.0 OR MIT

# The Sharing PDF is tagged for screen readers (D-82) and says it is PDF/UA-1 when the book qualifies.
# For each PDF given (or, with none, the tagged fixture the tests make) this runs:
#
#   qpdf --check                 the file is well formed
#   pdfinfo                      it says `Tagged: yes`
#   pdfinfo -struct-text         its structure tree reads out, in order
#   veraPDF --flavour ua1        it passes PDF/UA-1, when `verapdf` or Docker is there to run it
#
#   scripts/check-pdf-ua.sh [--require-verapdf] [BOOK.pdf ...]
#
# veraPDF is the `verapdf` command if installed, else the `verapdf/cli` Docker image. Without either
# the PDF/UA step is skipped with a note, unless `--require-verapdf` is given. The other steps need
# `qpdf` and poppler's `pdfinfo`; a missing one is reported and skipped the same way.
set -euo pipefail

REQUIRE=0
FILES=()
for arg in "$@"; do
  case "$arg" in
    --require-verapdf) REQUIRE=1 ;;
    *) FILES+=("$arg") ;;
  esac
done

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WORK="$(mktemp -d)"
chmod 755 "$WORK"
trap 'rm -rf "$WORK"' EXIT

if [ ${#FILES[@]} -eq 0 ]; then
  echo "Making the tagged fixture with the tests"
  FOLIO_TAGGED_PDF_OUT="$WORK/fixture.pdf" cargo test --manifest-path "$ROOT/Cargo.toml" \
    -p folio-export --test tagging -- writes_the_fixture_for_the_validators --exact --nocapture >/dev/null
  FILES=("$WORK/fixture.pdf")
fi

run_verapdf() {
  # $1 is a directory and $2 a file in it.
  if command -v verapdf >/dev/null 2>&1; then
    verapdf --flavour ua1 --format text "$1/$2"
  elif command -v docker >/dev/null 2>&1; then
    docker run --rm -v "$1":/data:ro verapdf/cli:latest --flavour ua1 --format text "/data/$2"
  else
    return 127
  fi
}

failed=0
for pdf in "${FILES[@]}"; do
  echo "== $pdf"
  if command -v qpdf >/dev/null 2>&1; then
    qpdf --check "$pdf" >/dev/null && echo "qpdf: well formed" || { echo "qpdf: FAILED"; failed=1; }
  else
    echo "qpdf: skipped (not installed)"
  fi
  if command -v pdfinfo >/dev/null 2>&1; then
    if pdfinfo "$pdf" | grep -q '^Tagged:[[:space:]]*yes'; then
      echo "pdfinfo: Tagged: yes"
    else
      echo "pdfinfo: not tagged"; failed=1
    fi
    if [ -n "$(pdfinfo -struct-text "$pdf" 2>/dev/null | head -c 1)" ]; then
      echo "structure (first lines):"
      pdfinfo -struct-text "$pdf" | head -n 12 | sed 's/^/  /'
    else
      echo "pdfinfo -struct-text: no structure"; failed=1
    fi
  else
    echo "pdfinfo: skipped (poppler is not installed)"
  fi
  dir="$(cd "$(dirname "$pdf")" && pwd)"
  set +e
  out="$(run_verapdf "$dir" "$(basename "$pdf")" 2>&1)"
  status=$?
  set -e
  if [ $status -eq 127 ]; then
    echo "veraPDF: skipped (no verapdf or docker)"
    [ $REQUIRE -eq 1 ] && failed=1
  elif echo "$out" | grep -q '^PASS'; then
    echo "veraPDF: PDF/UA-1 PASS"
  else
    echo "veraPDF: FAILED"; echo "$out" | head -n 20; failed=1
  fi
done
exit $failed
