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
| Release | Git tag `vX.Y.Z` and its GitHub Release | Built by automation (issue #10) |
| Installed instances | `deployments.toml` (planned, issue #10) | Updated after each installation as a Standard change |

## Change classes

The control board approves this table once. Each change is classified by its
content in the proposal, and the reviewer confirms the class. Only Major
changes return to the board. Class names follow ITIL: Standard means
pre-approved, not ordinary.

| Class | Meaning | Examples | Who decides |
|---|---|---|---|
| **Standard** | Pre-approved and routine. Follows a known procedure. No change to behaviour, components or platforms. | Colours, wording, documentation, test additions, updating the deployments record. | One reviewer who is not the author, then merge. |
| **Normal** | Not pre-approved. Needs assessment and an approver. | A feature within the current scope. Updating a component to a newer version. A change to the requirements, the dependency policy or this plan that does not change a class definition. | Technical lead approves after review and all automated checks pass. |
| **Major** | High risk or high impact. | A new platform. A new component licence type. Accepting a known weakness (an `ignore` entry in `deny.toml`). Any change to this table or to the charter. Cutting a release. | The control board, before work starts, presented with the risk and the rollback plan. |

Misclassifying a change is itself a finding, recorded as an issue.

| Role | Holder |
|---|---|
| Technical lead (Normal changes) | [name], or an agent assigned the title Technical Lead |
| Control board (Major changes) | Generic Software Control Board (GSCB), or its delegate [name] for urgent items. In this repository, the owner. |
| Reviewer pool (Standard changes) | Any committer other than the author, named in `CODEOWNERS` (issue #9), or an agent assigned the title Reviewer |

A role may be held by a person or by an AI agent assigned that job title
(DM-2026-007). An agent's review counts when it is a separate session from
the author's, it starts from the diff, and it records its title, the class it
confirms, what it checked and its verdict as a comment on the pull request.
The owner observes and directs; no Standard or Normal change waits on them.
A Major change needs the owner's direction, and a recorded direction is
sufficient.

## The change process

1. **Propose.** The change is written up as a pull request, separate from
   `main`, labelled with its class.
2. **Machine checks.** Unit tests, `cargo deny check`, `scripts/sbom.sh
   --check` and `scripts/ver.sh --check` run. All must pass.
3. **Independent review.** A reviewer who is not the author, a person or an
   agent holding a role, reads the change, confirms the class and approves.
   The author cannot approve their own work (AC-5).
4. **Merge.** The change joins `main`. History is permanent.
5. **Release.** A Major change: the release authority tags a version, and
   automation builds and seals the release package (issue #10).

## What the tools enforce and what people own

| Control | Enforced by | Reference |
|---|---|---|
| Tests, dependency policy, inventory and verification checks on every change | CI (`ci.yml`, `pages.yml`, `security.yml`) | REQ-1.4, REQ-1.7, REQ-3.3 |
| Inventory matches the lockfile | `scripts/sbom.sh --check` | REQ-1.4 |
| Verification matrix matches the code | `scripts/ver.sh --check` | Phase 4 quality criteria |
| No network capability | `deny.toml` bans and feature denials; CI grep | REQ-2.1, REQ-2.2 |
| Independent review; no force-push | Branch ruleset and `CODEOWNERS`, **not yet enabled** | REQ-3.2, REQ-3.4, issue #9 |
| Who may approve each class | This plan, DM-2026-007 and the board's minutes | Procedural |
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

## Release procedure (planned, issue #10)

1. Release authority approves the version as a Major change.
2. Move `CHANGELOG.md` Unreleased items under the new version.
3. Tag `vX.Y.Z` on `main`.
4. The release workflow builds the web bundle, regenerates the SBOM, runs
   `scripts/svd.sh --from <recipient baseline>`, writes `SHA256SUMS`, signs a
   provenance attestation and attaches everything to the GitHub Release.
5. After installation at a recipient, update `deployments.toml` (Standard).

## Standards

See [`standards.md`](standards.md). This plan draws on NIST SP 800-53 Rev. 5
(CM-2, CM-3, CM-5, CM-7, CM-8, CM-9, AC-5, SA-10, SA-11), NIST SP 800-128 and
MIL-STD-498.
