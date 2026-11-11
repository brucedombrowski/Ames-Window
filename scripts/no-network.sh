#!/usr/bin/env bash
# Prove the compiled app cannot reach the network (the charter, REQ-2.1).
#
#   scripts/no-network.sh path/to/app.wasm [path/to/glue.js]
#
# A WebAssembly module can call nothing but the functions it imports, so its
# import list is the complete set of things it is able to ask the browser to
# do. This reads that list out of the binary and fails if any entry is a
# network API. Given the JavaScript glue as well, it also checks that the
# glue's only `fetch` is the one that loads the module itself.
#
# What it cannot see: a link the viewer chooses to follow (web/index.html has
# one, Report feedback), which is navigation and not a call the app makes.
#
# Requires: python3.
set -euo pipefail

wasm="${1:?usage: scripts/no-network.sh app.wasm [glue.js]}"
glue="${2:-}"

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
print(f"{len(calls)} browser functions imported by {sys.argv[1]}; none is a network API.")
PY

if [[ -n "$glue" ]]; then
  # The loader fetches the module's own .wasm file and nothing else.
  fetches="$(grep -oE '(^|[^.A-Za-z_])fetch\(' "$glue" | wc -l | tr -d ' ')"
  if [[ "$fetches" -ne 1 ]] || ! grep -qE 'module_or_path = fetch\(module_or_path\)' "$glue"; then
    echo "$glue: expected exactly one fetch, the module loader; found $fetches." >&2
    exit 1
  fi
  if grep -nE 'XMLHttpRequest|WebSocket|sendBeacon|EventSource|RTCPeerConnection' "$glue"; then
    echo "$glue: network API in the JavaScript glue." >&2
    exit 1
  fi
  echo "$glue: one fetch, the module loader."
fi
