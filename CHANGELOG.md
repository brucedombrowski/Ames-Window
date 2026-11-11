# Changelog

All notable changes to this project are recorded here.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/)
and the project follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
No version has been released yet; everything is under Unreleased.

## [Unreleased]

### Added
- `sbom-web.cdx.json`: the SBOM for the browser build, which is what a
  release ships: the 54 components Cargo resolves for that target. `sbom.cdx.json` stays as the inventory of
  every target, 215 components. Both are generated and drift-checked by
  `scripts/sbom.sh`, and both are attached to a release (#21, #26).
- `.github/workflows/release.yml`: on a `v*` tag, tests, checks the SBOM and
  verification matrix, builds `ames-window-web.zip`, generates the SVD at the
  tagged commit, writes `SHA256SUMS`, signs a build-provenance attestation,
  creates the GitHub Release with all of them, and opens a pull request
  filing the SVD under `docs/releases/` (#10).
- `scripts/release.sh vX.Y.Z`: bumps `Cargo.toml`, `Cargo.lock` and the iOS
  bundle version, closes the changelog section, commits and tags.
  `--dry-run` previews the diff.
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

- The web page shows its measured frame rate and a **Report feedback** link
  that opens a GitHub issue form pre-filled with the frame rate, sizes,
  browser and build commit. `scripts/web.sh` stamps the commit into the page.
- `crates/ames-app/src/budget.rs`: the browser build times its frames and
  lowers its resolution, down to 480 pixels on the longest edge, when they
  take longer than 11 ms.

### Changed
- The browser build is compiled with WebAssembly SIMD (`.cargo/config.toml`)
  and keeps the painted backdrop between frames. In headless Chrome on an
  M2 Max at 1920x1080 the page went from 30 to 60 frames a second.
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

### Fixed
- `ames_core::perceived_angle` modelled the wrong reversal. It was honest
  within a quarter turn of face-on and otherwise mirrored the window left to
  right, which neither keeps the picture the same nor keeps the tall edge in
  front. It now reverses depth exactly when the tall edge is the farther one,
  so the believed window swings through half a turn with its tall edge always
  toward the viewer. The dial marks the believed tall end with a dot.
- The top-down dial did not draw the pen, and drew the bar for what the eye
  believes in the pen's blue, so the pen appeared to swing free of the
  window. The dial now draws the pen through the true window at the angle
  it is fixed at, and the believed bar is amber. `docs/preview.png` is
  regenerated.
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
- RUSTSEC-2026-0192 (`ttf-parser` unmaintained, transitive via winit)
  accepted with a recorded reason; tracked in #6.

[Unreleased]: https://github.com/brucedombrowski/Ames-Window/commits/main
