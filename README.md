# Ames Window

**Try it in the browser:** https://brucedombrowski.github.io/Ames-Window/

A pure‑Rust app that performs the **Ames window illusion**: a flat trapezoid,
painted like a six‑pane window in perspective, that spins full circle but
appears to swing back and forth. A blue pen through its centre seems to bend
and pass through solid wood.

It is written for iOS, but the same code also builds for the desktop and for
the browser. See [Status](#status) for what has actually been run where.

![Four frames of the window turning](docs/preview.png)

Tap the screen (or click it) to reveal a small dial: a bird's‑eye view. The
thin pale bar with a dot on its tall end is what the window is *really* doing.
The thick blue bar is what your eye insists it is doing. Tap again to hide it
and let the illusion back in.

New to Rust? Start with [docs/tour.md](docs/tour.md), a file‑by‑file reading
order that introduces one idea at a time.

## Layout

| Path | What it is |
|---|---|
| `crates/ames-core` | The geometry. No dependencies, no pixels. Trapezoid, camera, rotation, projection, and the perceived‑angle model. Fully unit tested. |
| `crates/ames-app` | The application. `winit` for the window and touch, `tiny-skia` to rasterise, `softbuffer` to present. The same binary runs on iOS, macOS, Windows, Linux and the browser. |

There is no Swift, no Objective‑C, no Xcode project and no GPU code. The
Objective‑C runtime is reached through the `objc2` bindings that `winit` and
`softbuffer` already use. On Wayland the window frame is drawn without a title
string, which keeps a font parser out of the dependency tree (DM-2026-006).
| `crates/ames-core` | The geometry. No dependencies, no pixels. Trapezoid, painted reveals, pen, camera, rotation, projection, and the perceived‑angle model. Every module has unit tests. |
| `crates/ames-app` | The application, binary name `ames-window`. `winit` for the window and input, `tiny-skia` to rasterise on the CPU, `softbuffer` to present. One source tree for every platform; the only platform‑specific code is the entry point and the clock on wasm. |
| `crates/ames-app/examples/snapshot.rs` | Renders frames to PNG without opening a window. |
| `web/index.html`, `scripts/web.sh` | The browser shell and the build script that produces `web/dist`. |
| `ios/Info.plist`, `scripts/ios.sh` | The iOS bundle metadata and the build‑sign‑install script. |
| `.github/workflows/pages.yml` | CI: tests, builds the wasm, deploys to GitHub Pages. |
| `.github/workflows/release.yml` | CI: on a `v*` tag, builds and seals the release. See [Releasing](#releasing). |
| `.github/workflows/security.yml` | CI: SBOM and verification‑matrix drift checks, `cargo deny`, and a grep for socket code. Runs on pushes to `main`, pull requests, weekly, and by hand. |
| `deny.toml`, `sbom.cdx.json` | Dependency policy and the committed CycloneDX bill of materials. See [Supply chain](#supply-chain). |
| `scripts/sbom.sh`, `scripts/svd.sh`, `scripts/ver.sh` | Regenerate the SBOM, write a Software Version Description, regenerate the verification matrix. |
| `scripts/release.sh` | Bump the version, close the changelog, commit and tag. |
| `CHANGELOG.md` | Keep a Changelog format. |
| `docs/` | Preview images, the tour, and the compliance documents listed below. |

There is no Swift, no Objective‑C, no Xcode project and no GPU code. Both
crates are `#![forbid(unsafe_code)]`. On Apple platforms the Objective‑C
runtime is reached through the `objc2` bindings that `winit` and `softbuffer`
depend on.

The app has no network capability of any kind and never will. See
[Supply chain](#supply-chain).

## Requirements

- Rust **1.87 or newer**, declared as `rust-version` in `Cargo.toml`. The
  workspace uses edition 2024 and `u128::is_multiple_of`; 1.86 fails to
  compile.
- `python3`, used by both scripts (`web.sh` to serve locally, `ios.sh` to read
  simulator and profile data).
- For the web: the `wasm32-unknown-unknown` target and `wasm-bindgen-cli`
  **0.2.130** exactly, matching the pinned `wasm-bindgen` crate.
- For iOS: a Mac with full Xcode (the script calls `vtool`, `simctl` and
  `devicectl`) and the `aarch64-apple-ios` and `aarch64-apple-ios-sim` targets.

## Splash screen

The app opens with `Hello, world!`, lettered in a five‑by‑seven dot font
defined in `font.rs`, cursor blinking. It holds for 1.8 s, then dissolves into
the window over 0.7 s.

<img src="docs/splash.png" width="195" alt="Hello, world! splash screen">

## Controls

Touch and the left mouse button are handled the same way.

| Input | Effect |
|---|---|
| Tap / click | Show or hide the dial that reveals the true rotation |
| Drag left or right | Turn the window by hand (spinning pauses while you hold it) |
| `Space` | Show or hide the dial |
| `X` | Toggle the pen |
| `R` | Reverse the spin |

The keys need a keyboard, so in practice they are desktop and browser only.

## Build

```sh
cargo test --workspace                 # 23 core tests, 9 app tests, 1 doctest
cargo run -p ames-app                  # desktop window; needs a display
cargo run -p ames-app --example snapshot -- out
```

The snapshot example writes six PNGs into `out/`: `ames-splash.png`, plus
`ames-000.png`, `ames-030.png`, `ames-060.png`, `ames-120.png` and
`ames-150.png` at those rotation angles, with the dial shown.

### Web

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.130 --locked
scripts/web.sh            # builds web/dist
scripts/web.sh serve      # builds, then serves http://localhost:8000
```

The `Pages` workflow runs `cargo test --workspace`, builds the wasm, and
deploys `web/dist` to GitHub Pages. It triggers on pushes to `main` and can be
started by hand from the Actions tab. Deploys from branches other than `main`
have failed at the deploy job; from `main` the deploy succeeded on
2026‑11‑11, so the app is at:

https://brucedombrowski.github.io/Ames-Window/

Known gaps in the web build are tracked in issue #4: the canvas renders at CSS
resolution rather than device pixels, the wasm is about 1 MB unoptimised, and
touch on a real phone browser has not been confirmed.

### iOS

```sh
rustup target add aarch64-apple-ios aarch64-apple-ios-sim
scripts/ios.sh sim        # build, bundle, install and launch in the simulator
scripts/ios.sh device     # build, sign, install and launch on a plugged‑in iPhone
```

What the script does:

1. `cargo build --release` for the chosen target.
2. Copies the binary and `ios/Info.plist` into `target/ios/<kind>/Ames Window.app`.
3. Rewrites the binary's recorded SDK to iOS 26 with `vtool`. iOS 27 refuses to
   run apps linked against its own SDK that do not adopt the UIScene lifecycle,
   which `winit` 0.30 does not. This is a workaround until `winit` does.
4. Simulator: boots an iPhone simulator if none is running, installs, launches.
5. Device: finds a valid provisioning profile covering
   `com.brucedombrowski.ames-window`, embeds it, signs with its entitlements
   (using `CODESIGN_IDENTITY` or the first "Apple Development" identity), then
   installs and launches with `devicectl`.

A device needs a provisioning profile for that bundle id. Xcode mints one the
first time you run any project with that bundle id and your team on the phone;
after that the script finds it on its own.

## Status

What has been verified, as of 2026‑11‑11 (details in issues #2 and #4):

| Target | State |
|---|---|
| Linux | `cargo test --workspace` passes. The windowed app has not been run (no display in CI). |
| Browser | Built in CI and deployed to Pages from `main`. Checked earlier in headless Chromium at phone size: splash, fade, spin, no console errors. Not yet tried on a real phone browser. |
| macOS | `cargo test --workspace` passed on a Mac. The windowed app has not been reported run. |
| iOS simulator | Before the `vtool` workaround the app installed, launched, then crashed at once under iOS 27. The workaround has not yet been confirmed on a simulator. |
| iPhone | Install previously failed for want of a provisioning profile. Not yet run. |
| Windows | Never built. |

## Compliance documentation

The process follows the [systems-engineering](https://github.com/brucedombrowski/systems-engineering)
framework and cites only openly published standards.

| Document | What it is |
|---|---|
| [`docs/start-here.md`](docs/start-here.md) | Reading order for everything below. Start here. |
| [`docs/standards.md`](docs/standards.md) | Governing standards register: NIST, FIPS, NTIA, ECMA, MIL-STD-498, BCP 14. What applies, what does not, and why. |
| [`docs/REQ-2026-001.json`](docs/REQ-2026-001.json) | Requirements, BCP 14 keywords, each citing a standard and section, with evidence anchors. |
| [`docs/VER-2026-001.md`](docs/VER-2026-001.md) | Verification matrix, generated by `scripts/ver.sh` from those anchors. CI fails if it drifts from the code. |
| [`docs/configuration-management.md`](docs/configuration-management.md) | CM plan: baselines, change classes, roles, process, what tools enforce versus what people own. |
| [`docs/RISK-2026-001.md`](docs/RISK-2026-001.md) | Risk register (NIST SP 800-30): likelihood, impact, controls, residual, owner. Rendered by `scripts/risk.sh` from the JSON register. |
| `docs/DM-2026-*.md` | Decision memoranda: options considered, decision, rationale with standard references. |
| [`docs/facts.json`](docs/facts.json) | Every number quoted anywhere, generated by `scripts/facts.sh`. CI fails if it drifts. |
| [`deployments.toml`](deployments.toml) | What has been delivered to and installed at each recipient. A release is not a delivery. |
| [`CHANGELOG.md`](CHANGELOG.md) | Keep a Changelog format, Semantic Versioning. |

## Supply chain

This repository is an IT security compliance demonstration for an
application. The application is deliberately trivial so that the process
around it is the subject. One rule follows from that and is permanent: **the
application never has network capability.** No telemetry, analytics, crash
reporting or update checks, inbound or outbound. Checked on 2026‑11‑11: no
networking crate is in `Cargo.lock` and the source uses no sockets. The
dependency policy rejects any component that could provide it, the
`Supply chain` workflow greps our own source for socket code, and every
Software Version Description states the claim. See issue #8.

`sbom.cdx.json` is a [CycloneDX](https://cyclonedx.org) 1.6 Software Bill of
Materials listing every crate the binary depends on, for every target, with
versions, licences and the dependency graph. It is generated from `Cargo.lock`
by `scripts/sbom.sh` and is byte-for-byte reproducible, so it lives in git and
the `Supply chain` workflow fails if it ever drifts from the lockfile. The
script rewrites the absolute path `cargo-cyclonedx` records for the
workspace's own crates to a fixed `path+file:///src/` prefix; without that
the file differed between machines, which is why the workflow's first run on
`main` failed.

`deny.toml` is the policy: `cargo deny check` fails on known vulnerabilities,
licences outside the permissive allowlist, wildcard versions, or crates from
anywhere other than crates.io. One advisory is explicitly ignored with a
reason in the file: RUSTSEC‑2026‑0192, `ttf-parser` unmaintained, reached only
through `winit` (issue #6). The same workflow runs the check on every push to
`main` and weekly, so new advisories surface without a code change. The
workflow's latest result is on the Actions tab; it was first merged to `main`
on 2026‑11‑11.

`scripts/svd.sh` writes a Software Version Description to `target/svd/`,
following MIL-STD-498 (DI-IPSC-81442) section by section: materials released,
every source file and third-party component with checksums, changes since the
last release, installation steps and known problems. It is generated entirely
from the repository at the current commit, so it is never written by hand and
never out of date. CI attaches it beside the SBOM on every run.

```sh
cargo install cargo-cyclonedx --version 0.5.10 --locked
cargo install cargo-deny --locked
scripts/sbom.sh           # regenerate after changing dependencies
cargo deny check          # vulnerabilities, licences, bans, sources
scripts/svd.sh            # Software Version Description for this commit
scripts/svd.sh --from v1.0.0   # ...for a recipient still running v1.0.0
scripts/svd.sh --site example-lab   # ...baseline looked up in deployments.toml
scripts/ver.sh && scripts/risk.sh && scripts/facts.sh   # regenerate the checked documents
```

### Release and delivery

A release is not a delivery. `scripts/release.sh vX.Y.Z` prepares a release
on `main`: version bump, every check, every generated document, changelog,
the release SVD in `docs/releases/`, a commit and an annotated tag. It does
not push; the release authority reviews and pushes, and the `Release`
workflow publishes the GitHub Release with the web bundle, SBOM, SVD,
`SHA256SUMS` and a signed provenance attestation.

`scripts/deliver.sh --to <site> --version vX.Y.Z` hands a release to a
recipient: a Software Version Description cut from that site's installed
version, the SVDs of releases it skipped, the assets, checksums and a
delivery record. `--installed` records the installation afterwards. Both
update `deployments.toml`.

An SVD is written for whoever receives the software. If a site is on 1.0 and
is being given 1.4, its "changes installed" section must cover 1.0 to 1.4,
including the 1.1 through 1.3 releases that site never installed. `--from`
takes the tag or commit the recipient has; the skipped releases are listed.

## Releasing

No version has been released yet. The procedure, from
[`docs/configuration-management.md`](docs/configuration-management.md):

1. The release authority approves the version (a Major change).
2. On a clean `main`: `scripts/release.sh vX.Y.Z` bumps `Cargo.toml`,
   `Cargo.lock` and `ios/Info.plist`, turns the changelog's Unreleased
   section into the version's section, commits and tags. Add `--dry-run` to
   see the diff first.
3. `git push origin main vX.Y.Z`.
4. The `Release` workflow tests, checks the SBOM and verification matrix,
   builds `ames-window-web.zip`, generates the Software Version Description
   at the tagged commit, writes `SHA256SUMS`, signs a build‑provenance
   attestation, and creates the GitHub Release carrying all of them. It then
   opens a pull request that files the SVD under `docs/releases/`.
5. After a delivery or installation, update `deployments.toml` (issue #10).

The workflow has not run yet, because there is no tag; the dry run of the
script has.

## Why it works

The brain assumes the taller edge of the trapezoid is nearer, because that is
what a real rectangle would look like in perspective. While the tall edge is in
fact nearer, perception and reality agree. Once the short edge swings nearer,
the assumption is wrong, so the brain mirrors the motion rather than abandon
it, and the window appears to reverse. `ames_core::perceived_angle` is that
rule as a single `if`. The red strips on the frame are the painted thickness
of a real window, not shadows; they are what sells the depth.

## License

Licensed under either of the [MIT license](LICENSE-MIT) or the
[Apache License, Version 2.0](LICENSE-APACHE), at your option. This is the
convention across the Rust ecosystem, so the code can be used anywhere other
Rust crates can.
