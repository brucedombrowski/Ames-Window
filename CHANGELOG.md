# Changelog

All notable changes to this project are recorded here.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/)
and the project follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- A browser tab icon, `web/icon.svg` (#14). The page's policy gains
  `img-src 'self'` for it and nothing else.
- The feedback report says how long the app spent on each frame, painting
  and presenting (#27).

### Changed
- `winit` is declared with `default-features = false` and
  `wayland-csd-adwaita-notitle` in place of `wayland-csd-adwaita`: the same
  window frame on Wayland, without a title, and without the font parser that
  drew it ([DM-2026-006](docs/DM-2026-006.md), #6). The every-target SBOM
  goes from 215 components to 211.
- In the browser the finished picture goes straight to the canvas instead of
  through `softbuffer`, which repacked every pixel twice inside WebAssembly.
  Measured in headless Chrome on an M2 Max at 1366x681: presenting went from
  7.6 ms a frame to 0.2 ms, and the app's whole frame from 8.6 ms to 1.1 ms.
  `softbuffer` is no longer part of the browser build; its SBOM lists 53
  components, down from 54.
- After the first release: the README links the latest release for download
  and describes the release procedure as it ran; REQ-4.5 (releases built by
  automation, with checksums and provenance) now carries evidence instead of
  an open status; the CM plan's release procedure is no longer "planned".

### Removed
- The `ignore` entry for RUSTSEC-2026-0192 in `deny.toml`. The policy now
  accepts no advisory.

### Fixed
- `scripts/release.sh` left a stale `[Unreleased]` link reference from the
  second release on; it now rebuilds the references. It prints how to
  recover if a step fails before the commit, and a dry run in which a step
  fails now fails, instead of reporting that nothing changed.
- `scripts/svd.sh` took any tag for a release, as its baseline and in its
  list of skipped releases; only `v*` tags count now (#37).

## [0.0.1] - 2026-11-11

### Added
- README: a link to the YouTube short that inspired the project (#30).
- A Windows program. `ames-window-windows-x64.zip` holds a single
  `ames-window.exe` that needs nothing installed, a README, its SBOM
  (`sbom-windows.cdx.json`, 42 components) and the licences. It is built and
  tested on Windows on every pull request (`.github/workflows/windows.yml`),
  checked to import no networking library (`scripts/windows-imports.sh`),
  scanned with ClamAV at release, and covered by `SHA256SUMS` and the
  provenance attestation. Not code-signed. CI also starts the program on
  Windows and fails if it does not stay up.
- `.gitattributes`: text files are checked out with LF endings everywhere,
  so a file packaged on Windows matches its committed bytes and checksum.
- `scripts/malware.sh` and REQ-1.10 (NIST SP 800-53 SI-3): ClamAV scans our
  source and every component's source on pushes to `main`, pull requests and
  weekly, and the built bundle before each deploy and each release (#23).
- A Content-Security-Policy in `web/index.html`: the browser refuses any
  load from or connection to another site, inline script and `eval`. The
  page's script moved to `web/app.js` so that inline script can be refused.
- `scripts/no-network.sh`: fails the build if the compiled module imports a
  network API by name, if the bundle's JavaScript or HTML mentions one or has
  any `fetch` but the loader's, or if the page lacks that policy. A tripwire,
  not a proof; the README says what it cannot see. New evidence for REQ-2.1
  (#23).
- GitHub Dependabot alerts enabled on the repository, a second advisory
  source beside RustSec (#23).
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
- `docs/REQ-2026-001.json`: requirements in the process
  framework's Phase 1 schema, BCP 14 keywords, each with evidence anchors.
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
- `ames-app`'s package description names the browser as well as iOS and the
  desktop; it appears as the root component of both SBOMs (#21).
- README: a "Where is..." table for the things a reviewer looks for (#26),
  and the application is described as deliberately trivial yet sufficiently
  complex to be a fair test (#29).
- The source grep for network code also catches `XMLHttpRequest`,
  `sendBeacon` and `EventSource`.
- References to a separately kept process framework no longer name or link
  it.
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
- The Software Version Description names Microsoft's C runtime among the
  things linked into the Windows program that no SBOM lists, as the README
  said it did.
- `scripts/release.sh` committed a release whose SBOMs and verification
  matrix were stale, because both change with the version and the changelog,
  and the release workflow would then have failed its own drift checks at the
  tag. The script now regenerates and commits them. It also takes only
  `v*` tags as previous releases and counts untracked files as an unclean
  tree. The workflow's last step, opening a pull request for the SVD, warns
  instead of failing when the repository does not let Actions open one.
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

[Unreleased]: https://github.com/brucedombrowski/Ames-Window/compare/v0.0.1...HEAD
[0.0.1]: https://github.com/brucedombrowski/Ames-Window/releases/tag/v0.0.1
