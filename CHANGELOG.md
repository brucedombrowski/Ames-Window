# Changelog

All notable changes to this project are recorded here.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/)
and the project follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
The notes on each GitHub Release are that version's section of this file.

## [Unreleased]

## [0.1.0] - 2026-11-11

First release: the Ames window illusion for the browser, with the compliance
process around it.

### Added
- `.github/workflows/release.yml`: on a `vX.Y.Z` tag, checks that the tag,
  `Cargo.toml` and this file agree, runs every check, builds the web bundle,
  writes the SVD, `SHA256SUMS` and a signed provenance attestation, publishes
  the GitHub Release and records the SVD under `docs/releases/` (#10).
- `deployments.toml`: delivered and installed version per recipient site;
  `scripts/svd.sh --site` reads the installed one as the SVD baseline (#10).
- `scripts/svd.sh --check`: every SVD in `docs/releases/` must reproduce
  from its tag; the `Supply chain` workflow runs it.
- `rust-version = "1.87"` in the workspace, so the minimum Rust is enforced
  rather than documented (#11).
- `docs/standards.md`: register of governing standards, all openly
  published, with what each is used for and which do not apply and why.
- `docs/REQ-2026-001.json`: requirements in the `systems-engineering`
  Phase 1 schema, BCP 14 keywords, each with evidence anchors.
- `docs/VER-2026-001.md`: verification matrix generated from those anchors
  by `scripts/ver.sh`; CI fails when it drifts from the code.
- `docs/DM-2026-001` to `004`: decision memoranda for the no-network charter,
  the SBOM format, generated-versus-committed documents, and Markdown over
  LaTeX for process artifacts.
- `docs/configuration-management.md`: the CM plan (NIST SP 800-53 CM-9) with
  change classes, roles, process, enforcement split and records.
- `scripts/svd.sh`: MIL-STD-498 Software Version Description generated from
  the repository; `--from` sets the recipient's installed version as the
  baseline and lists skipped releases.
- `deny.toml` charter enforcement: 21 networking crates banned and the
  network features of `tokio` and `web-sys` denied (#8).
- CI job `no-network` grepping our own source for socket and fetch code (#8).
- `sbom.cdx.json`: CycloneDX 1.6 SBOM, reproducible from `Cargo.lock`, with
  `scripts/sbom.sh` and a CI drift check (#7).
- `deny.toml`: RustSec advisories, licence allowlist, crates.io-only sources,
  no wildcard versions; `Supply chain` workflow on push, pull request and
  weekly (#7).
- `ames-app` marked `publish = false`.

### Changed
- `docs/configuration-management.md`: the change-class table gains a Meaning
  column and notes that the class names follow ITIL; the release procedure
  is no longer marked planned.
- `.github/workflows/pages.yml` no longer deploys from a retired branch (#11).

### Fixed
- `docs/tour.md`: the exercise now matches `columns: 3` in the code (#11).

### Security
- RUSTSEC-2026-0192 (`ttf-parser` unmaintained, transitive via winit)
  accepted with a recorded reason; tracked in #6.

[Unreleased]: https://github.com/brucedombrowski/Ames-Window/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/brucedombrowski/Ames-Window/releases/tag/v0.1.0
