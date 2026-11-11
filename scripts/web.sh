#!/usr/bin/env bash
# Build the app for the browser into web/dist.
#
#   scripts/web.sh            # build
#   scripts/web.sh serve      # build, then serve on http://localhost:8000
#
# Requires the wasm target and the wasm-bindgen CLI at the version pinned in
# Cargo.lock:
#   rustup target add wasm32-unknown-unknown
#   cargo install wasm-bindgen-cli --version 0.2.130 --locked
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$root"

cargo build --release -p ames-app --target wasm32-unknown-unknown

dist="web/dist"
rm -rf "$dist"
wasm-bindgen --target web --no-typescript \
  --out-dir "$dist" --out-name ames-window \
  target/wasm32-unknown-unknown/release/ames-window.wasm
# Stamp the page with the commit it was built from, so a feedback report says
# which build the viewer was looking at.
build="$(git rev-parse --short HEAD 2>/dev/null || echo unknown)"
git diff --quiet HEAD 2>/dev/null || build="$build+local"
sed "s/<meta name=\"build\" content=\"dev\">/<meta name=\"build\" content=\"$build\">/" \
  web/index.html > "$dist/index.html"
# GitHub Pages must not run Jekyll over this folder.
touch "$dist/.nojekyll"

echo "Built $dist"
if [[ "${1:-}" == "serve" ]]; then
  python3 -m http.server --directory "$dist" 8000
fi
