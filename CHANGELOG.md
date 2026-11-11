# Changelog

All notable changes to this project are recorded here.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/)
and the project follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
No version has been released yet; everything is under Unreleased.

## [Unreleased]

### Added
- `scripts/release.sh`: cuts a release in one command, setting the version
  in `Cargo.toml`, `Cargo.lock` and the iOS bundle, and stops before the
  push. `.github/workflows/release.yml`: rebuilds from the tag, re-verifies
  every generated document, publishes the web bundle, SBOM, SVD, `SHA256SUMS`
  and a signed provenance attestation. REQ-4.5 now has evidence (#10).
- `scripts/svd.sh --release vX.Y.Z`: release mode, committed as
  `docs/releases/SVD-vX.Y.Z.md`, identical before the release commit and at
  the tag; `--check` verifies it.
- `scripts/deliver.sh`: delivers a release to a site with a spanning SVD from
  that site's installed version, the skipped releases' SVDs, assets, checksums
  and a delivery record; `--installed` records the installation.
- `docs/RISK-2026-001.json` and its rendering `docs/RISK-2026-001.md`
  (`scripts/risk.sh`): nine risks in NIST SP 800-30 terms, each with controls
  traced to requirements, residual rating, owner and review trigger. Four are
  carried deliberately (RISK-1, 3, 7, 8) with the cost comparison written out;
  RISK-9 is carried into the first release so the second can show it closed.
- Outcome requirements REQ-0.1 to REQ-0.5 in `docs/REQ-2026-001.json`, in
  the form an acquirer writes them, each derived from the mechanisms that
  satisfy it; `scripts/ver.sh` evaluates derived requirements.
- `docs/facts.json` (`scripts/facts.sh`): every number documents quote,
  generated and drift-checked.
- `deployments.toml`: per-site delivered and installed versions;
  `scripts/svd.sh --site` reads the baseline from it. REQ-4.6 now has evidence.
- `.github/CODEOWNERS` naming reviewers for the policy files (#9).
- `docs/start-here.md`: reading order for the compliance documents.
- `docs/DM-2026-005.md` (dependencies not vendored; registry compromise
  accepted as RISK-3) and `docs/DM-2026-006.md` (Wayland decorations
  without a font parser).
- `rust-version = "1.87"` declared in the workspace; 1.86 fails to compile.
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
- `crates/ames-app/src/ios_scene.rs` and a `UIApplicationSceneManifest` in
  `ios/Info.plist`: the window is handed to a `UIWindowScene`, which iOS 13
  and later require before showing it.
- `scripts/ios.sh` accepts `PROVISIONING_PROFILE` to name the profile to
  embed, and `.gitignore` refuses signing material.
- `.github/workflows/ci.yml`: formatting, clippy and the tests on every pull
  request, plus a check that the wasm and Windows targets compile.

### Changed
- `winit` declared with `default-features = false` and
  `wayland-csd-adwaita-notitle` in place of `wayland-csd-adwaita`. Same
  window frame on Wayland, no title text, three components fewer (#6,
  DM-2026-006).
- Review may be done by an AI agent holding a named role, not only by a
  person, so no Standard or Normal change waits on the owner
  (`docs/DM-2026-007.md`, the CM plan, REQ-3.2).
- The browser build rasterises at most 1440 pixels on its longest edge and
  lets the browser scale the canvas up.
- `ames-app` is `#![deny(unsafe_code)]` rather than `forbid`, with the one
  exception confined to `ios_scene.rs`.
- `scripts/ios.sh` no longer rewrites the binary's SDK version with `vtool`;
  the scene manifest makes that workaround unnecessary.
- README rewritten from verified facts only: a Status table per platform,
  what `scripts/ios.sh` really does, exact requirements.
- The `Pages` workflow triggers on `main` only; a leftover development
  branch was removed from its trigger list.
- `docs/tour.md` exercise updated for the three-column window; it no longer
  claims a test fails when the column count changes, because none does.

### Removed
- The `ignore` entry for RUSTSEC-2026-0192 in `deny.toml`; the component
  is no longer in the tree.

### Fixed
- The window spun slowly in the browser on large or high-density displays
  (reported from Edge on a Windows laptop): a frame slower than the 100 ms
  step clamp advanced the scene by less time than had passed. The clamp is
  now 250 ms and frames are cheaper (see Changed).
- `scripts/ios.sh sim` no longer aborts when `open -a Simulator` finds no
  Simulator.app.
- `scripts/sbom.sh` and `scripts/release.sh` used GNU-only `sed -i` forms and
  failed on macOS.
- `scripts/sbom.sh` rewrites the absolute workspace path that
  `cargo-cyclonedx` puts in the workspace crates' `bom-ref`s to a fixed
  `path+file:///src/` prefix, so the SBOM is the same on every machine. The
  first `Supply chain` run on `main` failed on exactly this difference.

### Security
- RUSTSEC-2026-0192 (`ttf-parser` unmaintained, transitive via winit) was
  accepted with a recorded reason, then resolved by removing the component.
  RISK-2 closed; #6 closed.

[Unreleased]: https://github.com/brucedombrowski/Ames-Window/commits/main
