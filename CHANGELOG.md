# Changelog

All notable changes to this project are recorded here.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/)
and the project follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
No version has been released yet; everything is under Unreleased.

## [Unreleased]

### Added
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

### Security
- RUSTSEC-2026-0192 (`ttf-parser` unmaintained, transitive via winit)
  accepted with a recorded reason; tracked in #6.

[Unreleased]: https://github.com/brucedombrowski/Ames-Window/commits/main
