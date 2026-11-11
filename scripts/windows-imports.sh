#!/usr/bin/env bash
# Check that a Windows executable imports no networking library (the charter,
# REQ-2.1).
#
#   scripts/windows-imports.sh path/to/ames-window.exe
#
# A Windows program reaches the network through system libraries it names in
# its import table: Winsock, WinHTTP, WinINet and their kin. This lists the
# libraries the executable imports and fails if any of those is among them.
#
# Like scripts/no-network.sh for the browser build, this is a tripwire and
# not a proof: a program can load a library by name while it runs, which no
# import table shows. The source grep and the banned crates in deny.toml are
# the other checks; nothing here watches the running program.
#
# Requires: llvm-readobj, from `rustup component add llvm-tools`.
set -euo pipefail

exe="${1:?usage: scripts/windows-imports.sh ames-window.exe}"
readobj="$(find "$(rustc --print sysroot)" -name 'llvm-readobj*' -type f | head -1)"
[[ -n "$readobj" ]] || { echo "llvm-readobj not found; run: rustup component add llvm-tools" >&2; exit 2; }

libraries="$("$readobj" --coff-imports "$exe" | sed -n 's/^ *Name: *//p' | tr 'A-Z' 'a-z' | tr -d '\r' | sort -u)"
[[ -n "$libraries" ]] || { echo "$exe: no import table found; is this a Windows executable?" >&2; exit 2; }

echo "$exe imports:"
echo "$libraries" | sed 's/^/  /'

network="$(echo "$libraries" | grep -E '^(ws2_32|wsock32|mswsock|winhttp|wininet|urlmon|dnsapi|iphlpapi|webio|httpapi|rasapi32|netapi32)\.dll$' || true)"
if [[ -n "$network" ]]; then
  echo "Networking library imported:" >&2
  echo "$network" | sed 's/^/  /' >&2
  echo "The charter forbids it; see issue #8." >&2
  exit 1
fi
echo "None is a networking library."
