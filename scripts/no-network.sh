#!/usr/bin/env bash
# Check that the built app names no network API (the charter, REQ-2.1).
#
#   scripts/no-network.sh web/dist            # a built bundle: module, glue, page
#   scripts/no-network.sh path/to/app.wasm    # a module alone
#
# Two checks.
#
# The module. A WebAssembly module can call nothing but the functions it
# imports. This reads the import list out of the binary and fails if any
# entry is a network API by name.
#
# The JavaScript and HTML beside it. wasm-bindgen names every constructor
# `new`, so `new WebSocket(...)` is invisible in the import list and shows
# only in the generated glue. Every .js and .html file in the bundle is
# searched for network APIs, and the bundle may contain exactly one `fetch`:
# the loader fetching the app's own .wasm.
#
# What this is: a tripwire for the ordinary ways of adding network code. What
# it is not: a proof. A module can reach the network through functions that
# are not network APIs by name, for example by creating an <img> and setting
# its `src`, by assigning `location`, or by calling a function it looked up
# at run time. Those are refused by the browser instead, under the
# Content-Security-Policy in web/index.html, which allows connections to the
# page's own site only. Navigation the viewer chooses, the Report feedback
# link, is outside both.
#
# Requires: python3.
set -euo pipefail

target="${1:?usage: scripts/no-network.sh <bundle directory | app.wasm>}"
if [[ -d "$target" ]]; then
  bundle="$target"
  wasm="$(find "$bundle" -name '*.wasm' | head -1)"
  [[ -n "$wasm" ]] || { echo "no .wasm file in $bundle" >&2; exit 1; }
else
  bundle=""
  wasm="$target"
fi

python3 - "$wasm" <<'PY'
import re, sys

data = open(sys.argv[1], "rb").read()
assert data[:8] == b"\0asm\x01\0\0\0", "not a WebAssembly module"

def leb(i):
    value = shift = 0
    while True:
        byte = data[i]; i += 1
        value |= (byte & 0x7F) << shift; shift += 7
        if byte < 0x80:
            return value, i

def name(i):
    n, i = leb(i)
    return data[i:i + n].decode(), i + n

def limits(i):  # flags, minimum, and a maximum when flags say so
    flags, i = leb(i); _, i = leb(i)
    if flags & 1: _, i = leb(i)
    return i

imports, i = [], 8
while i < len(data):
    section = data[i]; size, i = leb(i + 1); end = i + size
    if section == 2:  # the import section
        count, i = leb(i)
        for _ in range(count):
            _, i = name(i); field, i = name(i); kind = data[i]; i += 1
            if kind == 0: _, i = leb(i)            # function: type index
            elif kind == 1: i = limits(i + 1)      # table: element type, limits
            elif kind == 2: i = limits(i)          # memory: limits
            elif kind in (3, 4): i += 2            # global, tag
            else: sys.exit(f"unknown import kind {kind}")
            imports.append(field)
    i = end

# wasm-bindgen names an import __wbg_<what it calls>_<hash>.
calls = sorted({re.sub(r"^__wbg_|_[0-9a-f]{16}$", "", f) for f in imports})
network = re.compile(
    r"fetch|xmlhttp|websocket|sendbeacon|eventsource|rtcpeer|rtcdata"
    r"|webtransport|importscripts|serviceworker|^open$", re.I)
# The fetch API's own types, matched exactly: `request` alone would also
# catch requestAnimationFrame.
types = re.compile(r"(^|_)(Request|Response|Headers)($|_)")
found = [c for c in calls if network.search(c) or types.search(c)]
if found:
    sys.exit("Network API imported by the module: " + ", ".join(found)
             + "\nThe charter forbids it; see issue #8.")
if not calls:
    sys.exit("The module imports nothing, so this is not the app.")
print(f"{sys.argv[1]} imports {len(calls)} distinct browser functions; none is a network API by name.")
PY

if [[ -n "$bundle" ]]; then
  pages=()
  while IFS= read -r f; do pages+=("$f"); done < <(find "$bundle" -type f \( -name '*.js' -o -name '*.html' \) | sort)
  if grep -nE 'XMLHttpRequest|WebSocket|sendBeacon|EventSource|RTCPeerConnection|WebTransport|importScripts' "${pages[@]}"; then
    echo "Network API in the bundle's JavaScript or HTML." >&2
    exit 1
  fi
  # Every fetch, however it is called, on whatever object.
  fetches="$(cat "${pages[@]}" | grep -oE 'fetch[[:space:]]*\(' | wc -l | tr -d ' ')"
  if [[ "$fetches" -ne 1 ]] || ! grep -qE 'module_or_path = fetch\(module_or_path\)' "${pages[@]}"; then
    echo "Expected exactly one fetch in the bundle, the module loader; found $fetches." >&2
    exit 1
  fi
  if ! grep -qE "http-equiv=\"Content-Security-Policy\"[^>]*default-src 'none'[^>]*connect-src 'self'" "$bundle/index.html"; then
    echo "$bundle/index.html carries no Content-Security-Policy limiting connections to this site." >&2
    exit 1
  fi
  echo "${#pages[@]} JavaScript and HTML files: one fetch, the module loader; the page's policy allows connections to its own site only."
fi
