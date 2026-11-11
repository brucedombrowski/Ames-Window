#!/usr/bin/env bash
# Check that a Windows executable imports no networking library (the charter,
# REQ-2.1).
#
#   scripts/windows-imports.sh path/to/ames-window.exe
#
# A Windows program reaches the network through system libraries it names in
# its import table: Winsock, WinHTTP, WinINet and their kin. This lists the
# libraries the executable imports and fails if any is not on the short list
# below, which is what the program imports today and contains no networking
# library. A new library, harmless or not, is a change to make here in review.
#
# Like scripts/no-network.sh for the browser build, this is a tripwire and
# not a proof. The program imports LoadLibrary and GetProcAddress, so it
# could load any library by name while it runs, and it imports COM
# (ole32), through which Windows components that use the network can be
# created with no networking library named anywhere. No import table shows
# either. The source grep and the banned crates in deny.toml are the other
# checks; nothing here watches the running program.
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

# What the program imports today: windowing, drawing, input, theming, the
# shell, COM, the kernel, and a random-number source for hash tables.
allowed='^(api-ms-win-core-synch-l1-2-0|bcryptprimitives|dwmapi|gdi32|imm32|kernel32|ntdll|ole32|shell32|user32|uxtheme)\.dll$'
unexpected="$(echo "$libraries" | grep -vE "$allowed" || true)"
if [[ -n "$unexpected" ]]; then
  echo "Imported, and not on the list in $0:" >&2
  echo "$unexpected" | sed 's/^/  /' >&2
  if echo "$unexpected" | grep -qE '^(ws2_32|wsock32|mswsock|winhttp|wininet|urlmon|dnsapi|iphlpapi|webio|httpapi|rasapi32|netapi32|mpr|wldap32|websocket|msquic|rpcrt4|secur32|crypt32|wlanapi)\.dll$'; then
    echo "At least one is a networking library. The charter forbids it; see issue #8." >&2
  else
    echo "If it is not a networking library, add it to the list, in review." >&2
  fi
  exit 1
fi
echo "All are on the expected list; none is a networking library."
