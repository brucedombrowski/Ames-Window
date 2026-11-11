#!/usr/bin/env bash
# Cut a release: one command, then one human push.
#
#   scripts/release.sh v0.1.0
#
# Cutting a release is a Major change (docs/configuration-management.md), so
# this script prepares everything and stops short of publishing. It:
#
#   1. refuses a dirty tree, a wrong branch, or an existing tag;
#   2. sets the version in Cargo.toml, Cargo.lock and ios/Info.plist;
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
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$root"

tag="${1:-}"
branch_ok="${2:-}"
[[ "$tag" =~ ^v[0-9]+\.[0-9]+\.[0-9]+$ ]] || { echo "usage: scripts/release.sh vX.Y.Z [--allow-branch]" >&2; exit 1; }
version="${tag#v}"
today="$(date -u +%Y-%m-%d)"

# 1. Preconditions.
# `git status --porcelain` rather than `git diff`: an untracked file is not a
# difference, but step 7's `git add -A` would put it in the release commit.
if [[ -n "$(git status --porcelain)" ]]; then
  echo "Working tree is not clean (untracked files count). Commit, stash or remove first." >&2; exit 1
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
if ! sed -n '/^## \[Unreleased\]/,/^## \[[0-9]/p' CHANGELOG.md | grep -q '^- '; then
  echo "The Unreleased section of CHANGELOG.md is empty; nothing to release." >&2; exit 1
fi
prev_tag="$(git describe --tags --abbrev=0 --match 'v[0-9]*' 2>/dev/null || true)"

# From here on the script changes files, and at the end commits and tags. If
# any step fails, say how to get back to where we started.
start="$(git rev-parse --short HEAD)"
undo() {
  echo >&2
  echo "release.sh failed; nothing was pushed. To undo what it changed:" >&2
  echo "  git tag -d $tag 2>/dev/null; git reset --hard $start && git clean -fd docs/releases" >&2
}
trap undo ERR

# 2. Version.
# First match only: the workspace version. (perl, not GNU-only `sed -i 0,/re/`.)
VERSION="$version" perl -0pi -e 's/^version = ".*"$/version = "$ENV{VERSION}"/m' Cargo.toml
cargo update --workspace --offline -q
python3 - "$version" <<'PY'
import re, sys
p = "ios/Info.plist"; s = open(p).read()
s, n = re.subn(r"(<key>CFBundleShortVersionString</key>\s*<string>)[^<]*", r"\g<1>" + sys.argv[1], s)
assert n == 1, "CFBundleShortVersionString not found in ios/Info.plist"
open(p, "w").write(s)
PY

# 3. Checks.
echo "== tests";            cargo test --workspace -q 2>&1 | grep -E 'test result|error'
echo "== dependency policy"; cargo deny check 2>&1 | tail -1
echo "== charter";          if grep -rnE 'std::net|TcpStream|TcpListener|UdpSocket|\.fetch\(|XmlHttpRequest|WebSocket' crates/ web/; then echo "network code found" >&2; exit 1; fi

# 4. Generated files that depend on the version or the lockfile.
scripts/sbom.sh
scripts/ver.sh
scripts/risk.sh
scripts/facts.sh >/dev/null

# 5. Changelog: Unreleased becomes the version; a fresh Unreleased is opened.
python3 - "$version" "$today" "$prev_tag" <<'PY'
import re, sys
version, today, prev = sys.argv[1:4]
repo = "https://github.com/brucedombrowski/Ames-Window"
p = "CHANGELOG.md"; s = open(p).read()
s = s.replace("## [Unreleased]\n", f"## [Unreleased]\n\n## [{version}] - {today}\n", 1)
s = s.replace("No version has been released yet; everything is under Unreleased.\n", "")
# Link references at the bottom: Unreleased now compares from this version,
# and this version gets its own line above the older ones.
older = re.findall(r"^\[\d[^\]]*\]: .*$", s, flags=re.M)
s = re.sub(r"^\[[^\]]+\]: .*\n?", "", s, flags=re.M).rstrip("\n") + "\n\n"
s += f"[Unreleased]: {repo}/compare/v{version}...main\n"
s += (f"[{version}]: {repo}/compare/{prev}...v{version}\n" if prev
      else f"[{version}]: {repo}/releases/tag/v{version}\n")
s += "".join(line + "\n" for line in older)
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
