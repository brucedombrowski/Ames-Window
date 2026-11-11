#!/usr/bin/env bash
# Prepare a release on main: bump the version, close the changelog section,
# commit and tag. Pushing the tag starts .github/workflows/release.yml, which
# builds and seals the release (docs/configuration-management.md, "Release
# procedure").
#
#   scripts/release.sh vX.Y.Z            # do it
#   scripts/release.sh vX.Y.Z --dry-run  # show what would change, touch nothing
#
# Cutting a release is a Major change: the release authority approves the
# version before this script is run. Semantic Versioning 2.0.0 applies.
#
# Requires: git, a clean working tree on main, and the checks the workflow
# will repeat: cargo test, scripts/sbom.sh --check, scripts/ver.sh --check.
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$root"

tag="${1:-}"
dry=""
[[ "${2:-}" == "--dry-run" ]] && dry=1
if [[ ! "$tag" =~ ^v[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  echo "usage: $0 vX.Y.Z [--dry-run]" >&2
  exit 2
fi
version="${tag#v}"
today="$(date -u +%Y-%m-%d)"

fail() { echo "error: $*" >&2; exit 1; }

# Preconditions.
if [[ -z "$dry" ]]; then
  [[ "$(git branch --show-current)" == "main" ]] || fail "releases are cut from main"
  git diff --quiet && git diff --cached --quiet || fail "working tree is not clean"
fi
git rev-parse -q --verify "refs/tags/$tag" >/dev/null && fail "tag $tag already exists"
current="$(sed -n 's/^version = "\(.*\)"$/\1/p' Cargo.toml | head -1)"
grep -q '^## \[Unreleased\]' CHANGELOG.md || fail "CHANGELOG.md has no Unreleased section"
if sed -n '/^## \[Unreleased\]/,/^## \[/p' CHANGELOG.md | grep -q '^### '; then :; else
  fail "the Unreleased section of CHANGELOG.md is empty; nothing to release"
fi
prev_tag="$(git describe --tags --abbrev=0 2>/dev/null || true)"

echo "Release $tag (workspace version $current -> $version), previous tag: ${prev_tag:-none}"

if [[ -z "$dry" ]]; then
  echo "Running the checks the workflow will repeat..."
  cargo test --workspace --quiet
  scripts/sbom.sh --check
  scripts/ver.sh --check
fi

# Version in the workspace manifest and the iOS bundle.
bump() {
  # First match only: the workspace version. (perl, not GNU-only `sed -i 0,/re/`.)
  CURRENT="$current" VERSION="$version" perl -0pi -e \
    's/^version = "\Q$ENV{CURRENT}\E"$/version = "$ENV{VERSION}"/m' Cargo.toml
  python3 - "$version" <<'PY'
import re, sys
v = sys.argv[1]
p = "ios/Info.plist"
s = open(p).read()
s, n = re.subn(r"(<key>CFBundleShortVersionString</key>\s*<string>)[^<]*", r"\g<1>" + v, s)
assert n == 1, "CFBundleShortVersionString not found"
open(p, "w").write(s)
PY
  # Cargo.lock records the workspace crates' own versions.
  cargo update -q --workspace
}

# CHANGELOG: Unreleased becomes this version, a fresh Unreleased goes above
# it, and the link references at the bottom follow Keep a Changelog.
changelog() {
  python3 - "$version" "$today" "$prev_tag" <<'PY'
import re, sys
version, today, prev = sys.argv[1:4]
p = "CHANGELOG.md"
s = open(p).read()
s = s.replace("No version has been released yet; everything is under Unreleased.\n", "", 1)
s = s.replace("## [Unreleased]\n", f"## [Unreleased]\n\n## [{version}] - {today}\n", 1)
repo = "https://github.com/brucedombrowski/Ames-Window"
s = re.sub(r"\n\[Unreleased\]: .*\n?$", "\n", s)
s = s.rstrip("\n") + "\n\n"
s += f"[Unreleased]: {repo}/compare/v{version}...HEAD\n"
if prev:
    s += f"[{version}]: {repo}/compare/{prev}...v{version}\n"
else:
    s += f"[{version}]: {repo}/releases/tag/v{version}\n"
open(p, "w").write(s)
PY
}

if [[ -n "$dry" ]]; then
  tmp="$(mktemp -d)"
  trap 'rm -rf "$tmp"' EXIT
  git worktree add -q "$tmp" HEAD
  (cd "$tmp" && bump && changelog && git --no-pager diff --stat && echo && git --no-pager diff CHANGELOG.md Cargo.toml ios/Info.plist)
  git worktree remove --force "$tmp"
  echo
  echo "Dry run: nothing changed. Without --dry-run this commits 'Release $tag' and tags it."
  exit 0
fi

bump
changelog
git add Cargo.toml Cargo.lock ios/Info.plist CHANGELOG.md
git commit -q -m "Release $tag" -m "Version $version. Changelog section closed; see CHANGELOG.md."
git tag -a "$tag" -m "Ames Window $tag"
echo
echo "Committed and tagged $tag. To publish:"
echo "  git push origin main $tag"
echo "The Release workflow then builds the web bundle, SBOM, SVD, checksums and"
echo "attestation, creates the GitHub Release, and opens a PR filing the SVD."
