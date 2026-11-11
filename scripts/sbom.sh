#!/usr/bin/env bash
# Generate the Software Bill of Materials (SBOM) at sbom.cdx.json.
#
#   scripts/sbom.sh            # regenerate
#   scripts/sbom.sh --check    # fail if the committed file is out of date
#
# The file is CycloneDX 1.6 JSON covering every crate the ames-window binary
# depends on, for every target (desktop, iOS and wasm), including build
# dependencies. It is derived entirely from Cargo.lock and crate metadata, so
# regenerating it is deterministic: the same lockfile always yields the same
# bytes. CI runs the --check mode so the SBOM can never drift from the lock.
#
# Requires the generator at the version pinned below:
#   cargo install cargo-cyclonedx --version 0.5.10 --locked
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$root"

generator_version="0.5.10"
out="sbom.cdx.json"

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

# A fixed timestamp keeps the output byte-for-byte reproducible. Freshness is
# tracked by git, not by this field. The serial number is omitted by default.
SOURCE_DATE_EPOCH=0 cargo cyclonedx \
  --manifest-path "$manifest" \
  --format json \
  --spec-version 1.6 \
  --target all \
  --all \
  --override-filename sbom.tmp \
  --quiet

if [[ "${1:-}" == "--check" ]]; then
  if cmp -s "$tmp.json" "$out"; then
    echo "$out is up to date."
  else
    diff -u "$out" "$tmp.json" | head -40 || true
    echo "$out is stale. Run scripts/sbom.sh and commit the result." >&2
    exit 1
  fi
else
  mv "$tmp.json" "$out"
  echo "Wrote $out ($(jq '.components | length' "$out") components)."
fi
