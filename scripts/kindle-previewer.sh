#!/usr/bin/env bash
# Set up Amazon's Kindle Previewer command line and run EPUB files through it, as KDP will
#
# (c) Copyright 2026 Liminal HQ, Scott Morris
# SPDX-License-Identifier: Apache-2.0 OR MIT

# KDP converts an uploaded EPUB with the same engine as Kindle Previewer, so a book that converts
# there without errors is one KDP will take. The Previewer app is a Windows and macOS program, but
# its package carries `KindlePreviewerCLI.exe`, which converts and validates without a window, and
# which runs under Wine on Linux.
#
#   scripts/kindle-previewer.sh setup           download Previewer and unpack its command line
#   scripts/kindle-previewer.sh check BOOK...   convert each EPUB, print Amazon's log, exit 1 on errors
#
# Options for `check`: `--keep DIR` also keeps the converted Kindle file and Amazon's logs in DIR.
#
# `KINDLE_PREVIEWER_CLI` names an existing `KindlePreviewerCLI` (or `.exe`) to use instead of the one
# `setup` unpacks, which is how a Windows or macOS machine uses its installed Previewer. Linux needs
# `wine`, `7z` and `curl`; `setup` and `check` say so when one is missing. The unpacked copy is under
# `${XDG_CACHE_HOME:-~/.cache}/folio/kindle-previewer` (about 900 MB) with its Wine prefix.
set -euo pipefail

HOME_DIR="${XDG_CACHE_HOME:-$HOME/.cache}/folio/kindle-previewer"
INSTALLER_URL="https://d3mb67z9ki82yk.cloudfront.net/KindlePreviewerSetup.exe"
CLI_DIR="$HOME_DIR/app/KindlePreviewerApp"
export WINEPREFIX="${KINDLE_PREVIEWER_PREFIX:-$HOME_DIR/prefix}"
export WINEDEBUG="${WINEDEBUG:--all}"

need() {
  command -v "$1" >/dev/null 2>&1 || {
    echo "kindle-previewer: '$1' is needed ($2)" >&2
    exit 2
  }
}

# The command to run the CLI, as words in `RUN`.
find_cli() {
  RUN=()
  if [ -n "${KINDLE_PREVIEWER_CLI:-}" ]; then
    case "$KINDLE_PREVIEWER_CLI" in
      *.exe) need wine "to run a Windows program"; RUN=(wine "$KINDLE_PREVIEWER_CLI") ;;
      *) RUN=("$KINDLE_PREVIEWER_CLI") ;;
    esac
  elif [ -f "$CLI_DIR/KindlePreviewerCLI.exe" ]; then
    need wine "to run Kindle Previewer's command line on Linux"
    RUN=(wine "$CLI_DIR/KindlePreviewerCLI.exe")
  else
    echo "kindle-previewer: not set up; run 'scripts/kindle-previewer.sh setup'" >&2
    exit 2
  fi
}

# `path` as the CLI reads it: a Windows path under Wine, as it is otherwise.
for_cli() {
  if [ "${RUN[0]}" = wine ]; then winepath -w "$1"; else printf '%s\n' "$1"; fi
}

setup() {
  need curl "to download Kindle Previewer"
  need 7z "to unpack its installer (p7zip)"
  need wine "to run Kindle Previewer's command line on Linux"
  if [ -f "$CLI_DIR/KindlePreviewerCLI.exe" ]; then
    echo "kindle-previewer: already set up in $HOME_DIR"
    return
  fi
  local work
  work="$(mktemp -d)"
  trap 'rm -rf "$work"' EXIT
  mkdir -p "$HOME_DIR/app"
  echo "kindle-previewer: downloading the installer (about 500 MB)"
  curl -fL --progress-bar -o "$work/setup.exe" "$INSTALLER_URL"
  # The installer is an NSIS bootstrapper around an MSIX package, which Wine cannot install but
  # which is a zip of the program.
  7z x -y -o"$work/nsis" "$work/setup.exe" >/dev/null
  7z x -y -o"$HOME_DIR/app" "$work/nsis/\$PLUGINSDIR/KindlePreviewerApp.msix" >/dev/null
  echo "kindle-previewer: ready in $HOME_DIR"
  find_cli
  "${RUN[@]}" --help 2>/dev/null | grep -m1 'Kindle Previewer' || true
}

# One book through the CLI. Prints what Amazon logged; returns 1 if the conversion failed.
check_one() {
  local book="$1" keep="${2:-}" out status stem
  stem="$(basename "$book")"
  mkdir -p "$HOME_DIR/work"
  out="$(mktemp -d "$HOME_DIR/work/out.XXXXXX")"
  # A run leaves its working folder behind in the Wine profile, and the next run of a book of the
  # same name fails on it. The converter also writes nothing unless its output is a pipe.
  rm -rf "$WINEPREFIX"/drive_c/users/*/AppData/Local/Temp/kpr_cli_* 2>/dev/null || true
  "${RUN[@]}" "$(for_cli "$(realpath "$book")")" --convert --output "$(for_cli "$out")" 2>&1 | cat >"$out/cli.txt" || true
  status="$(tr -d '\r' <"$out/Summary_Log.csv" 2>/dev/null | sed -n '2p' || true)"
  echo "== $stem"
  if [ -z "$status" ]; then
    echo "   no result; the converter said:"
    grep -v 'fixme:' "$out/cli.txt" | sed 's/^/   /'
    rm -rf "$out"
    return 1
  fi
  # Each row of the book's log is `"Type","Description"`: Error, Warning, Status.
  find "$out/Logs" -name '*.csv' -exec cat {} + 2>/dev/null | tr -d '\r' |
    sed '1d;s/^"\([A-Za-z]*\)","\(.*\)$/   \1: \2/;s/[",]*$//'
  if [ -n "$keep" ]; then
    mkdir -p "$keep"
    find "$out" -type f \( -name '*.mobi' -o -name '*.kpf' -o -name '*.azw3' -o -name '*_log.csv' \) -exec cp {} "$keep"/ \;
  fi
  rm -rf "$out"
  case "$status" in
    *Success*) echo "   converts" ;;
    *) echo "   does not convert"; return 1 ;;
  esac
}

check() {
  local keep="" books=() failed=0
  while [ $# -gt 0 ]; do
    case "$1" in
      --keep) keep="${2:?--keep needs a folder}"; shift 2 ;;
      *) books+=("$1"); shift ;;
    esac
  done
  [ ${#books[@]} -gt 0 ] || { echo "usage: $0 check [--keep DIR] BOOK.epub..." >&2; exit 2; }
  find_cli
  for b in "${books[@]}"; do
    check_one "$b" "$keep" || failed=1
  done
  exit "$failed"
}

case "${1:-}" in
  setup) setup ;;
  check) shift; check "$@" ;;
  *) echo "usage: $0 setup | check [--keep DIR] BOOK.epub..." >&2; exit 2 ;;
esac
