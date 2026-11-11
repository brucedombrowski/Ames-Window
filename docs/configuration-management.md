# Configuration management plan

The plan required by NIST SP 800-53 Rev. 5 CM-9, shaped by NIST SP 800-128.
It says what is controlled, how a change is classified and who decides it,
which rules the tools enforce and which people own, and what records exist.
Requirements it satisfies are in [`REQ-2026-001.json`](REQ-2026-001.json);
the evidence is in [`VER-2026-001.md`](VER-2026-001.md).

## Scope and charter

Ames Window is an IT security compliance demonstration for an application.
The application is kept deliberately trivial so that the process is the
subject. One charter item is permanent and outside every change class: **the
application never has network capability** (DM-2026-001, issue #8).

This repository is public and open-licensed. It carries the method:
generic process, policy, templates and generators. Anything that identifies
real people, devices, network destinations or system boundaries, which for a
real federal deployment would be Controlled Unclassified Information, does not
belong here. A filled-in instance lives in a private repository or the
recipient's document system, generated from the same templates.

## Configuration items and baselines

| Item | Baseline | Controlled by |
|---|---|---|
| Source code | Every commit on `main` | Git history; review before merge |
| Third-party components | `Cargo.lock` | Pinned versions and SHA-256 checksums; `deny.toml` |
| Component inventory | `sbom.cdx.json` | Regenerated from the lockfile; CI fails on drift |
| Dependency policy | `deny.toml` | Reviewed like code; changes to it are Normal or Major |
| Requirements | `docs/REQ-2026-001.json` | Reviewed like code; a change is a Normal change |
| Decisions | `docs/DM-*.md` and GitHub issues | Append-only in practice; a reversal is a new memorandum |
| Verification | `docs/VER-2026-001.md` | Generated from REQ anchors; CI fails on drift |
| Release | Git tag `vX.Y.Z` and its GitHub Release; `docs/releases/SVD-vX.Y.Z.md` | `scripts/release.sh`, then the `Release` workflow |
| Deliveries and installed instances | `deployments.toml` | `scripts/deliver.sh`; each update a Standard change |

## Change classes

The control board approves this table once. Each change is classified by its
content in the proposal, and the reviewer confirms the class. Only Major
changes return to the board.

| Class | What falls in it | Who decides |
|---|---|---|
| **Standard** | Colours, wording, documentation, test additions, updating the deployments record. No change to behaviour, components or platforms. | Preapproved. One reviewer who is not the author, then merge. |
| **Normal** | A feature within the current scope. Updating a component to a newer version. A change to the requirements, the dependency policy or this plan that does not change a class definition. | Technical lead approves after review and all automated checks pass. |
| **Major** | A new platform. A new component licence type. Accepting a known weakness (an `ignore` entry in `deny.toml`). Any change to this table or to the charter. Cutting a release. | The control board, before work starts, presented with the risk and the rollback plan. |

Misclassifying a change is itself a finding, recorded as an issue.

| Role | Holder |
|---|---|
| Technical lead (Normal changes) | [name] |
| Control board (Major changes) | Generic Software Control Board (GSCB), or its delegate [name] for urgent items |
| Reviewer pool (Standard changes) | Any committer other than the author, named in `CODEOWNERS` (issue #9) |

## The change process

1. **Propose.** The change is written up as a pull request, separate from
   `main`, labelled with its class.
2. **Machine checks.** Unit tests, `cargo deny check`, `scripts/sbom.sh
   --check` and `scripts/ver.sh --check` run. All must pass.
3. **Human review.** A second person reads the change, confirms the class and
   approves. The author cannot approve their own work (AC-5).
4. **Merge.** The change joins `main`. History is permanent.
5. **Release.** A Major change: the release authority runs `scripts/release.sh`,
   reviews, and pushes the tag; automation builds and seals the package.

## What the tools enforce and what people own

| Control | Enforced by | Reference |
|---|---|---|
| Tests, dependency policy, inventory and verification checks on every change | CI (`pages.yml`, `security.yml`) | REQ-1.4, REQ-1.7, REQ-3.3 |
| Inventory matches the lockfile | `scripts/sbom.sh --check` | REQ-1.4 |
| Verification matrix matches the code | `scripts/ver.sh --check` | Phase 4 quality criteria |
| No network capability | `deny.toml` bans and feature denials; CI grep | REQ-2.1, REQ-2.2 |
| Second-person review; no force-push | Branch ruleset and `CODEOWNERS`, **not yet enabled** | REQ-3.2, REQ-3.4, issue #9 |
| Who may approve each class | This plan and the board's minutes | Procedural |
| When to cut a release | Release authority | Procedural |
| Whether to accept a known weakness | Board, recorded in `deny.toml` with reason and issue | REQ-1.8 |
| Incident response when an advisory is published | Procedural: search the SBOM, open an issue, classify the fix | REQ-1.7 |
| Record retention | GitHub history and issues; releases hold the delivered SBOM and SVD | Procedural |

## Records

| Record | Where |
|---|---|
| Every change | Git history on `main`, with the pull request and review |
| Every decision | `docs/DM-*.md`, linked to a GitHub issue |
| Every accepted risk | `deny.toml` `ignore` entry with reason and issue |
| Every delivery | GitHub Release: executable, SBOM, SVD, `SHA256SUMS`, attestation |
| Changelog | `CHANGELOG.md`, Keep a Changelog format |

## Release, delivery and installation

Three events, three records. A release is not a delivery, and a delivery is
not an installation.

### Release (Major change)

1. The release authority approves the version number.
2. On `main` with a clean tree: `scripts/release.sh vX.Y.Z`. It sets the
   version, runs every check, regenerates the SBOM, verification matrix, risk
   rendering and facts, moves the changelog's Unreleased items under the new
   heading, generates `docs/releases/SVD-vX.Y.Z.md` in release mode, commits
   "Release vX.Y.Z" and creates an annotated tag. It does not push.
3. The release authority reviews the commit and pushes the branch and tag.
4. The `Release` workflow rebuilds from the tag, re-verifies every generated
   document against it, builds the web bundle, writes `SHA256SUMS`, signs a
   provenance attestation and publishes the GitHub Release with the bundle,
   SBOM, SVD and checksums.

### Delivery (Normal change)

`scripts/deliver.sh --to <site> --version vX.Y.Z`. Reads the site's installed
version from `deployments.toml`, generates the spanning SVD from that baseline
at the release tag (or uses the release's own SVD for a first delivery),
collects the per-release SVDs the site skipped, adds the release assets,
writes `SHA256SUMS` and `DELIVERY.md`, and marks the site as delivered.
Commit `deployments.toml`.

### Installation (Standard change)

When the recipient confirms: `scripts/deliver.sh --installed <site> vX.Y.Z`,
then commit `deployments.toml`. The site's baseline for the next delivery is
now this version.

## Standards

See [`standards.md`](standards.md). This plan draws on NIST SP 800-53 Rev. 5
(CM-2, CM-3, CM-5, CM-7, CM-8, CM-9, AC-5, SA-10, SA-11), NIST SP 800-128 and
MIL-STD-498.
