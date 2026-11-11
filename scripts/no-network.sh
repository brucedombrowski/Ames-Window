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
# Content-Security-Policy in web/index.html, which allows loading from and
# connecting to the page's own site only. Two things are outside both checks:
# navigation, whether the viewer's (the Report feedback link) or scripted,
# and a WebRTC peer connection built by looking its constructor up at run
# time, which the policy's `connect-src` does not cover.
#
# Requires: python3.
set -euo pipefail

target="${1:?usage: scripts/no-network.sh <bundle directory | app.wasm>}"
if [[ -d "$target" ]]; then
  bundle="$target"
  wasm="$(find "$bundle" -name '*.wasm')"
  [[ -n "$wasm" && "$(wc -l <<<"$wasm")" -eq 1 ]] || { echo "expected exactly one .wasm file in $bundle" >&2; exit 1; }
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
  # The policy must be where a browser will apply it, not merely present as
  # text. A copy inside a comment, a <template> or <body> is text; so is one
  # that follows anything which makes a browser leave <head> early, down to a
  # single stray character. So the page is read tag by tag, and it must run:
  # doctype, <html>, <head>, then nothing but <meta> and <title> before the
  # one expected policy. This refuses the ordinary ways a policy ends up
  # inert. It is not a browser and does not prove the policy is applied;
  # loading the built page in one would. Changing the policy means changing
  # it here too, in review.
  python3 - "$bundle/index.html" <<'PY'
import sys
from html.parser import HTMLParser

EXPECTED = "default-src 'none'; script-src 'self' 'wasm-unsafe-eval'; style-src 'unsafe-inline'; connect-src 'self'; worker-src blob:; base-uri 'none'; form-action 'none'"
VOID = {"meta", "link", "br", "img", "input", "hr", "base", "area", "col", "embed", "source", "track", "wbr"}

class Page(HTMLParser):
    """Records what comes before the first policy <meta>, and where it sits."""
    def __init__(self):
        super().__init__(convert_charrefs=True)
        self.stack, self.policies = [], []
        self.seen = []      # ("decl"|"start"|"end"|"selfclosed"|"text", what) up to the policy
        self.done = False   # set once the first policy has been seen
    def note(self, kind, what):
        if not self.done:
            self.seen.append((kind, what))
    def handle_decl(self, decl):
        self.note("decl", decl.lower())
    def handle_data(self, data):
        # HTML's own whitespace only: a non-breaking space is text to a browser.
        if data.strip(" \t\n\r\f"):
            self.note("text", data.strip(" \t\n\r\f")[:20])
    def handle_starttag(self, tag, attrs):
        a = dict(attrs)
        if tag == "meta" and any((v or "").lower() == "content-security-policy" for k, v in attrs if k == "http-equiv"):
            names = [k for k, _ in attrs]
            self.policies.append((list(self.stack), list(self.seen), [v for k, v in attrs if k == "content"], len(names) != len(set(names))))
            self.done = True
        self.note("start", tag)
        if tag not in VOID:
            self.stack.append(tag)
    def handle_startendtag(self, tag, attrs):
        self.note("selfclosed", tag)
        self.handle_starttag(tag, attrs)
        if tag not in VOID and self.stack and self.stack[-1] == tag:
            self.stack.pop()
    def handle_endtag(self, tag):
        self.note("end", tag)
        if tag in self.stack:
            while self.stack.pop() != tag:
                pass

page = Page()
page.feed(open(sys.argv[1]).read())
page.close()
problem = None
if len(page.policies) != 1:
    problem = f"expected one Content-Security-Policy <meta>, found {len(page.policies)}"
else:
    stack, seen, contents, repeated = page.policies[0]
    opening = [(k, w) for k, w in seen[:3]]
    rest = seen[3:]
    title_open = False
    stray = None
    for kind, what in rest:
        if kind == "start" and what == "meta":
            continue
        if kind == "start" and what == "title" and not title_open:
            title_open = True
        elif kind == "text" and title_open:
            continue
        elif kind == "end" and what == "title" and title_open:
            title_open = False
        else:
            stray = f"{kind} {what!r}"
            break
    if repeated or len(contents) != 1:
        problem = "the policy tag repeats an attribute; a browser keeps the first"
    elif contents[0] != EXPECTED:
        problem = "the policy is not the expected one"
    elif opening != [("decl", "doctype html"), ("start", "html"), ("start", "head")]:
        problem = "the page does not open with <!doctype html>, <html>, <head> and nothing else"
    elif stack != ["html", "head"] or title_open:
        problem = "the policy is not directly inside <head> (it is in " + ("<" + "><".join(stack) + ">" if stack else "no element") + ")"
    elif stray:
        problem = f"something other than <meta> or <title> comes before the policy in <head>: {stray}"
if problem:
    sys.exit(f"{sys.argv[1]}: {problem}.\nExpected, as the only policy and first in <head>:\n  {EXPECTED}")
PY
  echo "${#pages[@]} JavaScript and HTML files: one fetch, the module loader; the page carries the expected Content-Security-Policy, first in <head>."
fi
