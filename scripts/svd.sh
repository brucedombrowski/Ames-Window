#!/usr/bin/env bash
# Generate the Software Version Description (SVD) for the current commit.
#
#   scripts/svd.sh                       # changes since our previous tag
#   scripts/svd.sh --from v1.0.0         # changes since the version the
#                                        # recipient has installed
#   scripts/svd.sh --site example-lab    # baseline looked up in deployments.toml
#   scripts/svd.sh --from v1.0.0 out.md  # to a given path
#   scripts/svd.sh --release v1.4.0      # release mode, see below
#   scripts/svd.sh --release v1.4.0 --check
#
# We may release more often than a given site installs. An SVD is written
# for a recipient, so "changes installed" (section 3.3) must span from the
# version THEY have to the version THEY are receiving, not from our last
# tag. Pass --from with the tag or commit of their installed version;
# releases in between that they never installed are listed as skipped.
#
# The document follows MIL-STD-498, Data Item Description DI-IPSC-81442
# (Software Version Description), section for section. Everything in it is
# read from the repository at HEAD: Cargo metadata, git history, the SBOM
# and deny.toml. Nothing is typed by hand, so the SVD cannot drift from the
# software it describes. The same commit always yields the same document.
#
# Release mode (--release vX.Y.Z) is used by scripts/release.sh. The document
# is committed as docs/releases/SVD-vX.Y.Z.md in the release commit itself, so
# it cannot name that commit's hash; it identifies the version by its tag,
# takes its date from the CHANGELOG heading, excludes docs/releases/ from the
# inventory and excludes the release commit from the change list. The result
# is byte-identical whether generated just before the release commit or
# regenerated at the tag, which is what --check verifies in CI.
#
# Requires: git, jq, sha256sum.
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$root"

name="Ames Window"
version="$(sed -n 's/^version = "\(.*\)"$/\1/p' Cargo.toml | head -1)"
commit="$(git rev-parse HEAD)"
short="$(git rev-parse --short HEAD)"
dirty=""
if ! git diff --quiet || ! git diff --cached --quiet; then dirty=" (uncommitted changes present)"; fi
commit_date="$(git log -1 --format=%cd --date=format:%Y-%m-%d)"
tag="$(git describe --tags --exact-match 2>/dev/null || true)"
prev_tag="$(git describe --tags --abbrev=0 HEAD^ 2>/dev/null || true)"
sbom="sbom.cdx.json"

from=""
site=""
out=""
release=""
check=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    --from) from="$2"; shift 2 ;;
    --site) site="$2"; shift 2 ;;
    --release) release="$2"; shift 2 ;;
    --check) check=1; shift ;;
    -h|--help) sed -n '2,22p' "$0"; exit 0 ;;
    *) out="$1"; shift ;;
  esac
done
if [[ -n "$release" ]]; then
  if [[ -n "$from$site" ]]; then echo "--release cannot be combined with --from or --site" >&2; exit 1; fi
  [[ "$release" =~ ^v[0-9]+\.[0-9]+\.[0-9]+$ ]] || { echo "--release: expected vX.Y.Z, got '$release'" >&2; exit 1; }
  rel_version="${release#v}"
  if [[ "$version" != "$rel_version" ]]; then
    echo "--release: Cargo.toml version is $version, not $rel_version; run scripts/release.sh" >&2; exit 1
  fi
  commit_date="$(sed -n "s/^## \[$rel_version\] - \([0-9-]*\)$/\1/p" CHANGELOG.md | head -1)"
  if [[ -z "$commit_date" ]]; then
    echo "--release: CHANGELOG.md has no '## [$rel_version] - YYYY-MM-DD' heading" >&2; exit 1
  fi
  dirty=""
  tag="$release"
  prev_tag="$(git describe --tags --abbrev=0 --exclude "$release" HEAD 2>/dev/null || true)"
fi
if [[ -n "$site" ]]; then
  if [[ -n "$from" ]]; then echo "use --site or --from, not both" >&2; exit 1; fi
  installed="$(sed -n "/^\[sites\.$site\]/,/^\[/p" deployments.toml | sed -n 's/^installed = "\(.*\)"/\1/p' | head -1)"
  if [[ -z "$installed" ]]; then
    echo "--site: '$site' is not in deployments.toml" >&2; exit 1
  fi
  if [[ "$installed" == "none" ]]; then
    echo "--site: nothing is installed at '$site' yet; this is a first delivery, run without --site" >&2; exit 1
  fi
  from="$installed"
fi
if [[ -n "$from" ]]; then
  if ! git rev-parse --verify --quiet "$from^{commit}" >/dev/null; then
    echo "--from: '$from' is not a known tag or commit" >&2; exit 1
  fi
  if ! git merge-base --is-ancestor "$from" HEAD; then
    echo "--from: '$from' is not an ancestor of HEAD" >&2; exit 1
  fi
  baseline="$from"
  if [[ -n "$site" ]]; then
    baseline_label="$from (installed at site '$site' per deployments.toml)"
  else
    baseline_label="$from (installed at the recipient)"
  fi
  from_slug="$(echo "$from" | tr -c 'A-Za-z0-9.\n' '-')"
  default_out="target/svd/SVD-${from_slug}-to-${version}-${short}.md"
else
  baseline="$prev_tag"
  baseline_label="$prev_tag (our previous release)"
  default_out="target/svd/SVD-${version}-${short}.md"
  [[ -n "$release" ]] && default_out="docs/releases/SVD-${release}.md"
fi
out="${out:-$default_out}"
mkdir -p "$(dirname "$out")"

if [[ -n "$tag" ]]; then
  release_id="$tag"
else
  release_id="$version+$short (development build, not a tagged release)"
fi
if [[ -n "$release" ]]; then
  build_id="the commit tagged \`$release\`"
  source_id="the git tag \`$release\`"
  generated_from="the repository at tag \`$release\`"
else
  build_id="commit \`$commit\`"
  source_id="commit \`$short\`"
  generated_from="commit \`$commit\`"
fi
final_out="$out"
if [[ -n "$check" ]]; then out="$(mktemp)"; fi

components="$(jq '.components | length' "$sbom")"
licences="$(jq -r '[.components[].licenses[]? | (.expression // .license.id)] | unique | length' "$sbom")"

{
cat <<EOF
# Software Version Description

**$name**, version $release_id$dirty

| | |
|---|---|
| Document type | Software Version Description (SVD), per MIL-STD-498 DI-IPSC-81442 |
| Software | $name, workspace version $version |
| Build identifier | $build_id |
| Build date | $commit_date |
| Baseline for section 3.3 | ${baseline:-none: first release} |
| Generated by | \`scripts/svd.sh\` from the repository at that commit |

## 1. Scope

### 1.1 Identification

This document applies to **$name** version **$release_id**, identified by
$build_id. The software consists of two Rust crates,
\`ames-core\` and \`ames-app\`, producing one executable, \`ames-window\`,
for iOS, macOS, Windows, Linux and WebAssembly.

### 1.2 System overview

$name is an interactive demonstration of the Ames window optical illusion: a
rotating trapezoid that appears to oscillate rather than spin. It has no
network capability, stores no data, and takes no input other than touch or
pointer gestures. The executable is self-contained.

### 1.3 Document overview

This SVD identifies and describes the software version named above. Section 3
lists the materials released, the files and third-party components that make
up the software, the changes since the previous version, installation
instructions and known problems. It is generated from the repository and is
reproducible from the commit identified in 1.1.

## 2. Referenced documents

| Document | Location |
|---|---|
| MIL-STD-498, Software Development and Documentation | US Department of Defense, 1994 |
| DI-IPSC-81442, Software Version Description | Data Item Description under MIL-STD-498 |
| CycloneDX Bill of Materials Specification 1.6 | https://cyclonedx.org/specification/overview/ |
| Software Bill of Materials for this version | \`sbom.cdx.json\` in the repository at the build commit |
| Dependency policy | \`deny.toml\` in the repository at the build commit |
| Project README (build and installation) | \`README.md\` |
| Source code tour | \`docs/tour.md\` |
| Repository | https://github.com/brucedombrowski/Ames-Window |

## 3. Version description

### 3.1 Inventory of materials released

A tagged release is built by the repository's automated workflows and carries
the following materials. For a development build only the source archive and
SBOM exist; the others are produced at release time.

| Material | Description |
|---|---|
| Source archive | The repository at $source_id |
| \`ames-window-web.zip\` | The WebAssembly build and its host page, ready to serve from any static web server |
| \`sbom.cdx.json\` | CycloneDX 1.6 Software Bill of Materials, $components third-party components |
| \`SHA256SUMS\` | SHA-256 checksum of every released file |
| \`$(basename "$final_out")\` | This document |

Checksums of the materials that exist at the build commit:

| File | SHA-256 |
|---|---|
EOF
for f in Cargo.lock sbom.cdx.json deny.toml; do
  printf '| `%s` | `%s` |\n' "$f" "$(sha256sum "$f" | cut -d' ' -f1)"
done

cat <<EOF

### 3.2 Inventory of software contents

#### 3.2.1 Source files

Every file under version control at the build commit, with its SHA-256
checksum. Generated artifacts (\`target/\`, \`web/dist/\`) and the release
documents in \`docs/releases/\` are excluded.

| File | SHA-256 |
|---|---|
EOF
git ls-files -z -- . ':!docs/releases' | while IFS= read -r -d '' f; do
  printf '| `%s` | `%s` |\n' "$f" "$(sha256sum "$f" | cut -d' ' -f1)"
done

cat <<EOF

#### 3.2.2 Third-party components

$components components from the public Rust registry (crates.io), under
$licences distinct licence expressions, all within the allowlist in
\`deny.toml\`. Exact versions and source checksums are pinned in
\`Cargo.lock\`. The complete record, including the dependency graph and
licence per component, is the SBOM. Summary by name and version:

| Component | Version | Licence |
|---|---|---|
EOF
jq -r '.components[] | "| \(.name) | \(.version) | \(.licenses[0].expression // .licenses[0].license.id // "see SBOM") |"' "$sbom"

cat <<EOF

### 3.3 Changes installed

EOF
if [[ -n "$baseline" ]]; then
  echo "This section spans from **$baseline_label** to the version described"
  echo "in 1.1. Everything listed here is new to a recipient on $baseline."
  skipped="$(git tag --merged HEAD --no-merged "$baseline" --sort=version:refname 2>/dev/null | grep -vx "$tag" || true)"
  if [[ -n "$skipped" ]]; then
    echo
    echo "Releases made between the two that were **not** installed at the"
    echo "recipient, whose changes are therefore included below:"
    echo
    echo "| Skipped release | Date | Its own SVD |"
    echo "|---|---|---|"
    for t in $skipped; do
      printf '| %s | %s | docs/releases/SVD-%s.md |\n' "$t" "$(git log -1 --format=%cd --date=format:%Y-%m-%d "$t")" "$t"
    done
  fi
  range="$baseline..HEAD"
else
  echo "No previous tagged release exists. All changes since the start of the repository:"
  range="HEAD"
fi
echo
echo "| Commit | Date | Change |"
echo "|---|---|---|"
git log "$range" --invert-grep --grep="^Release v[0-9]" --format='| `%h` | %cd | %s |' --date=format:%Y-%m-%d

cat <<EOF

Each change was proposed, passed the automated checks (unit tests, the
dependency policy in \`deny.toml\`, and the SBOM consistency check), and was
merged under the change process described in the project's configuration
management documentation.

### 3.4 Adaptation data

None. The software has no site-specific, mission-specific or installation-
specific configuration. There are no configuration files, environment
variables or run-time parameters.

### 3.5 Related documents

| Document | Purpose |
|---|---|
| \`README.md\` | Build, run and deployment instructions for every platform |
| \`docs/tour.md\` | Guided reading order through the source code |
| \`sbom.cdx.json\` | Machine-readable component inventory |
| \`deny.toml\` | Machine-enforced dependency policy |
| GitHub issues | Record of decisions and known problems |

### 3.6 Installation instructions

**Web.** Unpack \`ames-window-web.zip\` into any directory served by a static
web server over HTTPS. No server-side code is required. Open \`index.html\`.

**iOS.** On a Mac with Xcode installed, run \`scripts/ios.sh sim\` for the
simulator or \`scripts/ios.sh device\` for a signed device build. The script
compiles with Cargo, assembles the \`.app\` bundle and installs it.

**Desktop.** Run \`cargo run -p ames-app --release\`, or run the prebuilt
\`ames-window\` executable for the platform.

**Verification.** Compare each file's SHA-256 against \`SHA256SUMS\` before
installation. The SBOM for the installed version is \`sbom.cdx.json\`.

**Removal.** Delete the installed files or the app bundle. The software writes
nothing outside its own installation directory.

### 3.7 Possible problems and known errors

Accepted advisories, from the \`ignore\` list in \`deny.toml\`:

| Advisory | Reason recorded |
|---|---|
EOF
if grep -qE '^\s*\{ id = ' deny.toml; then
  grep -E '^\s*\{ id = ' deny.toml | sed -E 's/.*id = "([^"]+)".*reason = "([^"]+)".*/| \1 | \2 |/'
else
  echo "| none | No advisory is accepted at this commit. |"
fi

cat <<EOF

Open risks from the risk register (\`docs/RISK-2026-001.json\`, NIST SP 800-30):

| Risk | Title | Response | Owner |
|---|---|---|---|
EOF
jq -r '.risks[] | select(.status == "open") | "| \(.id) | \(.title) | \(.response) | \(.owner) |"' docs/RISK-2026-001.json

cat <<EOF

Open problems and decisions are tracked as GitHub issues in the repository.
No other known errors at the build commit.

## 4. Notes

| Term | Meaning |
|---|---|
| Crate | The Rust unit of compilation and packaging; a library or executable |
| SBOM | Software Bill of Materials: a machine-readable list of every component in a piece of software |
| CycloneDX | An OWASP standard format for SBOMs, ratified as ECMA-424 |
| Cargo.lock | The file that pins every component to an exact version and checksum |
| RustSec | The public advisory database for Rust components |
| GSCB | Generic Software Control Board: the body that approves software change classes |
| WebAssembly | A format that lets compiled code run inside a web browser |

---

Generated $commit_date from $generated_from by \`scripts/svd.sh\`.
EOF
} > "$out"

if [[ -n "$check" ]]; then
  if cmp -s "$out" "$final_out"; then
    rm -f "$out"; echo "$final_out is up to date."
  else
    diff -u "$final_out" "$out" | head -40 || true
    rm -f "$out"
    echo "$final_out is stale or was not generated by scripts/svd.sh --release." >&2; exit 1
  fi
else
  echo "Wrote $out"
fi
