#!/usr/bin/env bash
# Deliver a release to a recipient site, and record it.
#
#   scripts/deliver.sh --to example-lab --version v0.2.0 [--assets DIR]
#   scripts/deliver.sh --installed example-lab v0.2.0
#
# A release is not a delivery. A release is a tagged version with its assets.
# A delivery hands one release to one recipient, with a Software Version
# Description cut from THAT recipient's installed version. This script:
#
#   1. reads the site's installed version from deployments.toml (the baseline);
#   2. checks out the release tag in a temporary worktree;
#   3. generates the spanning SVD (baseline -> version) there, or uses the
#      release's own SVD for a first delivery;
#   4. collects the per-release SVDs of any releases the site skipped;
#   5. takes the release assets from --assets DIR, or downloads them with
#      `gh release download` when gh is installed;
#   6. writes target/deliveries/<site>-<version>/ with SHA256SUMS and a
#      DELIVERY.md record;
#   7. marks the site as delivered in deployments.toml (not installed).
#
# When the recipient confirms installation, run --installed to update the
# baseline. Both updates to deployments.toml are Standard changes: commit them.
#
# Requires: git, jq, sha256sum; gh optional.
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$root"

site=""; version=""; assets=""; mode="deliver"
while [[ $# -gt 0 ]]; do
  case "$1" in
    --to) site="$2"; shift 2 ;;
    --version) version="$2"; shift 2 ;;
    --assets) assets="$2"; shift 2 ;;
    --installed) mode="installed"; site="$2"; version="$3"; shift 3 ;;
    -h|--help) sed -n '2,26p' "$0"; exit 0 ;;
    *) echo "unknown argument $1" >&2; exit 1 ;;
  esac
done
[[ -n "$site" && -n "$version" ]] || { echo "usage: scripts/deliver.sh --to SITE --version vX.Y.Z [--assets DIR]" >&2; exit 1; }
today="$(date -u +%Y-%m-%d)"

site_block() { sed -n "/^\[sites\.$site\]/,/^\[/p" deployments.toml; }
site_field() { site_block | sed -n "s/^$1 = \"\(.*\)\"/\1/p" | head -1; }
set_field() {  # set_field key value  (within the site's block only)
  python3 - "$site" "$1" "$2" <<'PY'
import re, sys
site, key, val = sys.argv[1:4]
p = "deployments.toml"; s = open(p).read()
m = re.search(rf"(\[sites\.{re.escape(site)}\]\n)(.*?)(?=\n\[|\Z)", s, re.S)
head, body = m.group(1), m.group(2)
if re.search(rf"^{key} = ", body, re.M):
    body = re.sub(rf"^{key} = .*$", f'{key} = "{val}"', body, flags=re.M)
else:
    body = body.rstrip("\n") + f'\n{key} = "{val}"\n'
s = s[:m.start()] + head + body + s[m.end():]
open(p, "w").write(s)
PY
}

[[ -n "$(site_block)" ]] || { echo "site '$site' is not in deployments.toml" >&2; exit 1; }
git rev-parse -q --verify "refs/tags/$version" >/dev/null || { echo "tag $version does not exist; release first" >&2; exit 1; }

if [[ "$mode" == "installed" ]]; then
  delivered="$(site_field delivered)"
  if [[ "$delivered" != "$version" ]]; then
    echo "deployments.toml says '$site' was delivered '$delivered', not $version. Deliver first." >&2; exit 1
  fi
  set_field installed "$version"
  set_field installed_on "$today"
  echo "Marked $version installed at $site on $today. Commit deployments.toml (Standard change)."
  exit 0
fi

installed="$(site_field installed)"
pkg="target/deliveries/$site-$version"
rm -rf "$pkg"; mkdir -p "$pkg"

# 2. The release tag in a worktree, so the SVD is generated from exactly that tree.
wt="$(mktemp -d)"
trap 'git worktree remove --force "$wt" 2>/dev/null || true' EXIT
git worktree add -q --detach "$wt" "$version"

# 3. Spanning SVD, or the release's own for a first delivery.
if [[ "$installed" == "none" || -z "$installed" ]]; then
  # The filed copy at the tag; else the one filed after the release (a tag
  # cannot contain its own release's record); else generate it from the tag.
  if [[ -f "$wt/docs/releases/SVD-$version.md" ]]; then
    cp "$wt/docs/releases/SVD-$version.md" "$pkg/"
  elif [[ -f "docs/releases/SVD-$version.md" ]]; then
    cp "docs/releases/SVD-$version.md" "$pkg/"
  else
    (cd "$wt" && scripts/svd.sh "$root/$pkg/SVD-$version.md" >/dev/null)
  fi
  baseline="none (first delivery)"
  skipped=""
else
  git rev-parse -q --verify "refs/tags/$installed" >/dev/null || { echo "installed version '$installed' is not a tag" >&2; exit 1; }
  (cd "$wt" && scripts/svd.sh --from "$installed" "$root/$pkg/SVD-$installed-to-$version.md" >/dev/null)
  baseline="$installed"
  # 4. Per-release SVDs the site skipped, as supporting history.
  skipped="$(git tag --merged "$version" --no-merged "$installed" --sort=version:refname | grep -vx "$version" || true)"
  for t in $skipped $version; do
    [[ -f "$wt/docs/releases/SVD-$t.md" ]] && cp "$wt/docs/releases/SVD-$t.md" "$pkg/"
  done
fi

# 5. Release assets.
mkdir -p "$pkg/release"
if [[ -n "$assets" ]]; then
  cp -r "$assets"/. "$pkg/release/"
elif command -v gh >/dev/null; then
  gh release download "$version" -D "$pkg/release" --clobber
else
  echo "No --assets DIR and gh is not installed. Download the $version assets from GitHub Releases into $pkg/release and re-run with --assets." >&2
  exit 1
fi

# 6. Record and checksums.
{
cat <<EOF
# Delivery record

| | |
|---|---|
| Recipient | $site |
| Version delivered | $version |
| Recipient's installed version (baseline) | $baseline |
| Releases skipped by the recipient | ${skipped:-none} |
| Delivery date | $today |
| Prepared by | \`scripts/deliver.sh\` from the repository at tag $version |

A release is not a delivery and a delivery is not an installation. This
package records the delivery. Installation is recorded in \`deployments.toml\`
when the recipient confirms it (\`scripts/deliver.sh --installed $site $version\`).

## Contents

| File | SHA-256 |
|---|---|
EOF
(cd "$pkg" && find . -type f ! -name DELIVERY.md ! -name SHA256SUMS | sort | while read -r f; do
  printf '| `%s` | `%s` |\n' "${f#./}" "$(sha256sum "$f" | cut -d' ' -f1)"
done)
} > "$pkg/DELIVERY.md"
(cd "$pkg" && find . -type f ! -name SHA256SUMS | sort | sed 's|^\./||' | xargs sha256sum > SHA256SUMS)

# 7. Record the delivery.
set_field delivered "$version"
set_field delivered_on "$today"

echo "Delivery package: $pkg"
sed -n '/## Contents/,$p' "$pkg/DELIVERY.md"
echo
echo "deployments.toml updated: $site delivered $version on $today. Commit it (Standard change)."
