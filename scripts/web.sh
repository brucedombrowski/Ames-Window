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

# The compiler records the paths of source files in the program, for panic
# messages. Left alone, a bundle built on someone's own machine carries their
# home directory, and so their account name, dozens of times. Rewrite the two
# prefixes that vary by machine to fixed ones; the repository's own comes
# last because rustc applies the last that matches, and a clone may sit under
# the home directory. (--config adds to the flags in .cargo/config.toml; a
# RUSTFLAGS variable would replace them.)
cargo build --release --locked -p ames-app --target wasm32-unknown-unknown \
  --config "target.wasm32-unknown-unknown.rustflags=['--remap-path-prefix=$HOME=/home/builder', '--remap-path-prefix=$root=/src']"

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
cp web/app.js web/icon.svg "$dist/"
# GitHub Pages must not run Jekyll over this folder.
touch "$dist/.nojekyll"

echo "Built $dist"
if [[ "${1:-}" == "serve" ]]; then
  python3 -m http.server --directory "$dist" 8000
fi
