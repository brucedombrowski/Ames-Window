#!/usr/bin/env bash
# Cut a release: one command, then one human push.
#
#   scripts/release.sh v0.1.0
#
# Cutting a release is a Major change (docs/configuration-management.md), so
# this script prepares everything and stops short of publishing. It:
#
#   1. refuses a dirty tree, a wrong branch, or an existing tag;
#   2. sets the workspace version in Cargo.toml and Cargo.lock;
#   3. runs every check: tests, dependency policy, charter grep;
#   4. regenerates the SBOM, verification matrix, risk rendering and facts;
#   5. moves CHANGELOG.md's Unreleased items under the new version heading;
#   6. generates docs/releases/SVD-vX.Y.Z.md in release mode;
#   7. commits "Release vX.Y.Z" and creates an annotated tag.
#
# It does NOT push. The release authority reviews the commit and runs the
# push it prints. Pushing the tag starts .github/workflows/release.yml, which
# rebuilds, re-verifies every generated file at the tag, and publishes the
# GitHub Release with the web bundle, SBOM, SVD, SHA256SUMS and attestation.
#
# Requires: git, cargo, jq, cargo-deny, cargo-cyclonedx (pinned version).
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
branch_ok="${2:-}"
[[ "$tag" =~ ^v[0-9]+\.[0-9]+\.[0-9]+$ ]] || { echo "usage: scripts/release.sh vX.Y.Z [--allow-branch]" >&2; exit 1; }
version="${tag#v}"
today="$(date -u +%Y-%m-%d)"

# 1. Preconditions.
if ! git diff --quiet || ! git diff --cached --quiet; then
  echo "Working tree is not clean. Commit or stash first." >&2; exit 1
fi
branch="$(git rev-parse --abbrev-ref HEAD)"
if [[ "$branch" != "main" && "$branch_ok" != "--allow-branch" ]]; then
  echo "Releases are cut from main (on '$branch'). Pass --allow-branch to override for a rehearsal." >&2; exit 1
fi
if git rev-parse -q --verify "refs/tags/$tag" >/dev/null; then
  echo "Tag $tag already exists." >&2; exit 1
fi
if ! grep -q '^## \[Unreleased\]' CHANGELOG.md; then
  echo "CHANGELOG.md has no Unreleased section." >&2; exit 1
fi

# 2. Version.
sed -i "0,/^version = \".*\"$/s//version = \"$version\"/" Cargo.toml
cargo update --workspace --offline -q

# 3. Checks.
echo "== tests";            cargo test --workspace -q 2>&1 | grep -E 'test result|error'
echo "== dependency policy"; cargo deny check 2>&1 | tail -1
echo "== charter";          if grep -rnE 'std::net|TcpStream|TcpListener|UdpSocket|\.fetch\(|XmlHttpRequest|WebSocket' crates/; then echo "network code found" >&2; exit 1; fi

# 4. Generated files that depend on the version or the lockfile.
scripts/sbom.sh
scripts/ver.sh
scripts/risk.sh
scripts/facts.sh >/dev/null

# 5. Changelog: Unreleased becomes the version; a fresh Unreleased is opened.
python3 - "$version" "$today" <<'PY'
import re, sys
version, today = sys.argv[1], sys.argv[2]
p = "CHANGELOG.md"; s = open(p).read()
s = s.replace("## [Unreleased]\n", f"## [Unreleased]\n\n## [{version}] - {today}\n", 1)
s = re.sub(r"\n\[Unreleased\]: .*\n?$",
           f"\n[Unreleased]: https://github.com/brucedombrowski/Ames-Window/compare/v{version}...main\n"
           f"[{version}]: https://github.com/brucedombrowski/Ames-Window/releases/tag/v{version}\n", s)
s = s.replace("No version has been released yet; everything is under Unreleased.\n", "")
open(p, "w").write(s)
PY

# 6. Matrix and facts again (the changelog edit moves anchor lines), then the
#    release SVD last, since its inventory checksums every other file.
scripts/ver.sh >/dev/null
scripts/facts.sh >/dev/null
scripts/svd.sh --release "$tag"
scripts/ver.sh --check >/dev/null
scripts/facts.sh --check >/dev/null
scripts/risk.sh --check >/dev/null
scripts/sbom.sh --check >/dev/null

# 7. Commit and tag.
git add -A
git commit -q -m "Release $tag" -m "Software Version Description: docs/releases/SVD-$tag.md
Changelog: CHANGELOG.md section [$version]
SBOM: sbom.cdx.json ($(jq '.components | length' sbom.cdx.json) components)"
git tag -a "$tag" -m "Ames Window $tag" -m "See docs/releases/SVD-$tag.md"
scripts/svd.sh --release "$tag" --check

cat <<EOF

Release $tag is committed and tagged locally. Nothing has been published.

Review:   git show --stat HEAD && cat docs/releases/SVD-$tag.md
Publish:  git push origin $branch && git push origin $tag
Undo:     git tag -d $tag && git reset --hard HEAD^

Pushing the tag runs the release workflow, which publishes the GitHub Release.
EOF
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
  sed -i "0,/^version = \"$current\"$/s//version = \"$version\"/" Cargo.toml
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
