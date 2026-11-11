#!/usr/bin/env bash
# Generate the Software Bills of Materials: sbom.cdx.json and sbom-web.cdx.json.
#
#   scripts/sbom.sh            # regenerate
#   scripts/sbom.sh --check    # fail if either committed file is out of date
#
# Two files, both CycloneDX 1.6 JSON, including build dependencies:
# sbom.cdx.json covers every crate for every target (desktop, iOS, wasm), and
# sbom-web.cdx.json covers what Cargo resolves for the browser build, which
# is what a release ships. Both are derived entirely from Cargo.lock and crate metadata, so
# regenerating them is deterministic: the same lockfile always yields the
# same bytes. CI runs the --check mode so neither can drift from the lock.
#
# Requires the generator at the version pinned below:
#   cargo install cargo-cyclonedx --version 0.5.10 --locked
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$root"

generator_version="0.5.10"

installed="$(cargo cyclonedx --version 2>/dev/null | awk '{print $2}' || true)"
if [[ "$installed" != "$generator_version" ]]; then
  echo "cargo-cyclonedx $generator_version required (found '${installed:-none}')." >&2
  echo "  cargo install cargo-cyclonedx --version $generator_version --locked" >&2
  exit 1
fi

# cargo-cyclonedx writes one file per workspace member, next to each manifest.
# Only the application crate's file is kept: its dependency tree includes
# ames-core and everything else that ships.
manifest="crates/ames-app/Cargo.toml"
tmp="crates/ames-app/sbom.tmp"
cleanup() { find crates -name 'sbom.tmp.json' -delete; }
trap cleanup EXIT

# Two inventories from the one lockfile:
#   sbom.cdx.json      every target at once: browser, iOS, macOS, Windows,
#                      Linux and Android. The audit view; no single build
#                      contains all of it.
#   sbom-web.cdx.json  what Cargo resolves for the browser build, which is
#                      what a release ships. A little more than is
#                      delivered (it includes build-time tools), and not the
#                      toolchain or the generated JavaScript glue.
#   sbom-windows.cdx.json  the same for the Windows program, which a release
#                      also ships.
stale=0
for pair in "all:sbom.cdx.json" "wasm32-unknown-unknown:sbom-web.cdx.json" "x86_64-pc-windows-msvc:sbom-windows.cdx.json"; do
  target="${pair%%:*}"; out="${pair#*:}"

  # A fixed timestamp keeps the output byte-for-byte reproducible. Freshness
  # is tracked by git, not by this field. The serial number is omitted by
  # default.
  SOURCE_DATE_EPOCH=0 cargo cyclonedx \
    --manifest-path "$manifest" \
    --format json \
    --spec-version 1.6 \
    --target "$target" \
    --all \
    --override-filename sbom.tmp \
    --quiet

  # cargo-cyclonedx identifies the workspace's own crates by the absolute
  # path of their manifests (`path+file:///home/you/Ames-Window/crates/...`),
  # which differs between machines and would make the file depend on where
  # the repository was cloned. Rewrite that prefix to a fixed one so the
  # output is the same everywhere.
  # (perl, not `sed -i`: GNU and BSD sed disagree on that flag's argument.)
  ROOT="$root" perl -pi -e 's#path\+file://\Q$ENV{ROOT}\E/#path+file:///src/#g' "$tmp.json"

  if [[ "${1:-}" == "--check" ]]; then
    if cmp -s "$tmp.json" "$out"; then
      echo "$out is up to date."
    else
      diff -u "$out" "$tmp.json" | head -40 || true
      echo "$out is stale. Run scripts/sbom.sh and commit the result." >&2
      stale=1
    fi
  else
    mv "$tmp.json" "$out"
    echo "Wrote $out ($(jq '.components | length' "$out") components)."
  fi
done
exit "$stale"
