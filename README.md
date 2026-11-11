# Ames Window

**Try it in the browser:** https://brucedombrowski.github.io/Ames-Window/

**The inspiration:** [this YouTube short](https://www.youtube.com/shorts/LdZxRlyM30o)
of the demonstration, which the app sets out to reproduce.

A pure‑Rust app that performs the **Ames window illusion**: a flat trapezoid,
painted like a six‑pane window in perspective, that spins full circle but
appears to swing back and forth. A blue pen through its centre seems to bend
and pass through solid wood.

It is written for iOS, but the same code also builds for the desktop and for
the browser. See [Status](#status) for what has actually been run where.

![Four frames of the window turning](docs/preview.png)

Tap the screen (or click it) to reveal a small dial: a bird's‑eye view. The
thin pale bar with a dot on its tall end is what the window is *really* doing,
and the blue line through it is the pen, fixed to the window and turning with
it. The thick amber bar behind them is what your eye insists the window is
doing; its dot, the believed tall end, stays on the viewer's side while the
pale dot goes all the way round. Tap again to hide the dial and let the
illusion back in.

New to Rust? Start with [docs/tour.md](docs/tour.md), a file‑by‑file reading
order that introduces one idea at a time.

## Layout

| Path | What it is |
|---|---|
| `crates/ames-core` | The geometry. No dependencies, no pixels. Trapezoid, painted reveals, pen, camera, rotation, projection, and the perceived‑angle model. Every module has unit tests. |
| `crates/ames-app` | The application, binary name `ames-window`. `winit` for the window and input, `tiny-skia` to rasterise on the CPU, `softbuffer` to present. One source tree for every platform; the only platform‑specific code is the entry point and the clock on wasm, and `ios_scene.rs` on iOS. |
| `crates/ames-app/examples/snapshot.rs` | Renders frames to PNG without opening a window. |
| `web/index.html`, `web/app.js`, `scripts/web.sh` | The browser shell with its Content-Security-Policy, the page's script, and the build script that produces `web/dist`. |
| `ios/Info.plist`, `scripts/ios.sh` | The iOS bundle metadata and the build‑sign‑install script. |
| `.github/workflows/ci.yml` | CI: formatting, clippy and tests on every pull request; builds the browser bundle and runs `scripts/no-network.sh` on it; checks that the Windows target compiles. |
| `.github/workflows/pages.yml` | CI: tests, builds the wasm, deploys to GitHub Pages. |
| `.github/workflows/release.yml` | CI: on a `v*` tag, builds and seals the release. See [Releasing](#releasing). |
| `.github/workflows/security.yml` | CI: SBOM and verification‑matrix drift checks, `cargo deny`, and a grep for socket code. Runs on pushes to `main`, pull requests, weekly, and by hand. |
| `deny.toml`, `sbom-web.cdx.json`, `sbom-windows.cdx.json`, `sbom.cdx.json` | Dependency policy, and the committed CycloneDX bills of materials for the browser build, the Windows program, and every target. See [Supply chain](#supply-chain). |
| `packaging/windows/README.txt`, `.github/workflows/windows.yml`, `scripts/windows-imports.sh` | The Windows package: its README, the workflow that builds and tests it on Windows, and the check that the program imports no networking library. |
| `scripts/no-network.sh`, `scripts/malware.sh` | Fail the build if the bundle names a network API; scan files for known malware with ClamAV. |
| `scripts/sbom.sh`, `scripts/svd.sh`, `scripts/ver.sh` | Regenerate the SBOM, write a Software Version Description, regenerate the verification matrix. |
| `scripts/release.sh` | Bump the version, close the changelog, commit and tag. |
| `CHANGELOG.md` | Keep a Changelog format. |
| `docs/` | Preview images, the tour, and the compliance documents listed below. |

There is no Swift, no Objective‑C, no Xcode project and no GPU code. `ames-core` is
`#![forbid(unsafe_code)]`; `ames-app` is `#![deny(unsafe_code)]` with one
exception, `ios_scene.rs`, which calls UIKit to put the window on screen. On
Apple platforms the Objective‑C runtime is reached through the `objc2`
bindings that `winit` and `softbuffer` depend on.

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
- For iOS: a Mac with full Xcode (the script calls `simctl`, `codesign`
  and `devicectl`) and the `aarch64-apple-ios` and `aarch64-apple-ios-sim` targets.

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
cargo test --workspace                 # 24 core tests, 23 app tests, 1 doctest
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

Every frame is rasterised on the CPU, so the browser build is tuned to stay
smooth on slow machines:

- It is compiled with 128‑bit WebAssembly SIMD (`.cargo/config.toml`), which
  current browsers support (Safari from 16.4). Measured in headless Chrome on an M2 Max at
  1920x1080, that alone took the page from 30 to 60 frames a second.
- The backdrop is painted once per size and copied in each frame.
- It draws at most 1440 pixels on the longest edge and lets the browser scale
  the canvas up. `budget.rs` times each frame, and when frames take longer
  than 11 ms it lowers that limit step by step, down to 480 pixels.

The page shows its measured frame rate in the bottom right corner beside a
**Report feedback** link. The link opens GitHub's new‑issue form filled in
with the frame rate, sizes, browser and build commit. It is an ordinary link
in `web/index.html`. Those measurements are in the link's address, so they go
to GitHub when the viewer follows it, and become public only if the viewer
submits the form. Until the link is followed the page sends nothing.

Known gaps in the web build are tracked in issue #4: the wasm is about 1 MB
unoptimised, and touch on a real phone browser has not been confirmed.

### Windows

A release carries `ames-window-windows-x64.zip`: one program,
`ames-window.exe`, with a README, its SBOM and the licences. Unzip and
double-click; there is nothing to install, and the C runtime is linked in so
no redistributable is needed. It is not code-signed, so Windows SmartScreen
warns the first time it runs; the package's README says what to do and how to
check the download against the release's checksums.

Every pull request also builds the package on a Windows machine, runs the
tests there, and starts the program to see that it stays up. That build can
be downloaded from the pull request's `CI` run as the
`ames-window-windows-x64` artifact, to try a change before it is released.
It is not a release: it has not been malware-scanned, and its README says so.

To build it yourself, on Windows with Rust and the Visual Studio build tools:

```sh
cargo build --release -p ames-app      # target\release\ames-window.exe
```

### iOS

```sh
rustup target add aarch64-apple-ios aarch64-apple-ios-sim
scripts/ios.sh sim        # build, bundle, install and launch in the simulator
scripts/ios.sh device     # build, sign, install and launch on a plugged‑in iPhone
```

What the script does:

1. `cargo build --release` for the chosen target.
2. Copies the binary and `ios/Info.plist` into `target/ios/<kind>/Ames Window.app`.
3. Simulator: boots an iPhone simulator if none is running, installs, launches.
4. Device: takes the provisioning profile named by `PROVISIONING_PROFILE`, or
   finds a valid one covering `com.brucedombrowski.ames-window`; embeds it,
   signs with its entitlements (using `CODESIGN_IDENTITY` or the first "Apple
   Development" identity), then installs and launches with `devicectl`.

`ios/Info.plist` declares a `UIApplicationSceneManifest`, and
`crates/ames-app/src/ios_scene.rs` puts the window on the scene it names.
Both are required on iOS 13 and later: without the manifest, iOS 26 and newer
stop the app at launch; without the scene, the window is never shown and the
screen stays black. See the header of `ios_scene.rs` for the detail. This
replaces an earlier workaround that rewrote the binary's recorded SDK version
with `vtool`.

A device needs a provisioning profile for that bundle id. Xcode mints one the
first time you run any project with that bundle id and your team on the phone;
after that the script finds it on its own. To choose one yourself, point
`PROVISIONING_PROFILE` at it. Either way the entitlements are read back out of
the profile, so the two cannot drift apart:

```sh
CODESIGN_IDENTITY="Apple Development: you (TEAMID)" \
PROVISIONING_PROFILE=~/Library/MobileDevice/Provisioning\ Profiles/your.mobileprovision \
  scripts/ios.sh device
```

Keep that profile outside the repository: it carries your team ID and the UDID
of every registered device. `.gitignore` refuses `*.mobileprovision` for you.

## Status

What has been verified, as of 2026‑11‑11 (details in issues #2 and #4):

| Target | State |
|---|---|
| Linux | `cargo test --workspace` passes. The windowed app has not been run (no display in CI). |
| Browser | Built in CI and deployed to Pages from `main`. Checked earlier in headless Chromium at phone size: splash, fade, spin, no console errors. Not yet tried on a real phone browser. |
| macOS | `cargo test --workspace` passed on a Mac. The windowed app has not been reported run. |
| iOS simulator | The app used to install, launch, then crash at once under iOS 27. It now declares a scene manifest and adopts the scene (`ios_scene.rs`); commit `64595f7` records it running on an iPhone 18 Pro simulator: splash, then the window and pen, animating. |
| iPhone | Install previously failed for want of a provisioning profile. Not yet run. |
| Windows | Built and tested on Windows in CI on every pull request (`windows.yml`), and packaged as a single `ames-window.exe` that needs nothing installed. Whether anyone has yet run the program on a Windows desktop is not recorded here; the browser build is what a Windows machine has been seen to run. |

## Compliance documentation

The process follows a systems engineering process framework maintained
separately, and cites only openly published standards.

| Document | What it is |
|---|---|
| [`docs/standards.md`](docs/standards.md) | Governing standards register: NIST, FIPS, NTIA, ECMA, MIL-STD-498, BCP 14. What applies, what does not, and why. |
| [`docs/REQ-2026-001.json`](docs/REQ-2026-001.json) | Requirements, BCP 14 keywords, each citing a standard and section, with evidence anchors. |
| [`docs/VER-2026-001.md`](docs/VER-2026-001.md) | Verification matrix, generated by `scripts/ver.sh` from those anchors. CI fails if it drifts from the code. |
| [`docs/configuration-management.md`](docs/configuration-management.md) | CM plan: baselines, change classes, roles, process, what tools enforce versus what people own. |
| `docs/DM-2026-*.md` | Decision memoranda: options considered, decision, rationale with standard references. |
| [`CHANGELOG.md`](CHANGELOG.md) | Keep a Changelog format, Semantic Versioning. |

## Supply chain

This repository is an IT security compliance demonstration for an
application. The application is deliberately trivial, yet sufficiently complex to be a
fair test (several platforms, a real dependency tree, a browser build, one
boundary of unsafe code), so that the process
around it is the subject. One rule follows from that and is permanent: **the
application never has network capability.** No telemetry, analytics, crash
reporting or update checks, inbound or outbound. Checked on 2026‑11‑11: no
networking crate is in `Cargo.lock` and the source uses no sockets. The
dependency policy rejects any component that could provide it, the
`Supply chain` workflow greps our own source for socket code, and every
Software Version Description states the claim. See issue #8.

The web page carries one outbound link, **Report feedback**, which opens a
pre‑filled GitHub issue form in a new tab, carrying the page's measurements
in its address. That is navigation the viewer chooses, not a capability of
the application: the page and the wasm module make no request of their own,
and the `Supply chain` workflow greps `web/` as well as `crates/` to keep it
so. The owner asked for the link; it is not telemetry, because nothing is
sent without the viewer acting.

There are three [CycloneDX](https://cyclonedx.org) 1.6 Software Bills of
Materials, each listing components with versions, licences, checksums and the
dependency graph:

| File | Covers | Components |
|---|---|---|
| [`sbom-web.cdx.json`](sbom-web.cdx.json) | The browser build, which is what a release ships. **This is the release SBOM.** See below for its exact scope. | 54 |
| [`sbom-windows.cdx.json`](sbom-windows.cdx.json) | The Windows program, which a release also ships, on the same terms: what Cargo resolves for that target. | 42 |
| [`sbom.cdx.json`](sbom.cdx.json) | Every target the source builds for at once: browser, iOS, macOS, Windows, Linux, Android. An audit view; no single build contains all of it. | 215 |

What the two build files are, exactly: everything Cargo resolves for that
target, which is a little more than what is delivered.

| | Browser | Windows |
|---|---|---|
| Components listed | 54 | 42 |
| Compiled for the target, so can be in the program | 40 | 33 |
| Tools that run only while building (procedural macros, build scripts) | 10 | 1 |
| Optional, not compiled at all | 4 | 8 |

Nothing compiled into either program is missing from its file. Some things
in or behind each are not Cargo components and so are in none of the three
files: the Rust standard library and compiler for both; for the browser, the
JavaScript glue generated by `wasm-bindgen-cli` (a build tool pinned to
0.2.130) and the page itself; for Windows, Microsoft's C runtime, which is
linked into the program from the build machine's Visual Studio tools. The
Software Version Description names them. Checksums are recorded for every
component but our own `ames-core`.

The every-target file is four times the browser one because `winit`, the
window and input library, carries a separate backend for each operating
system. All three are generated from `Cargo.lock` by `scripts/sbom.sh` and
are byte-for-byte reproducible, so they live in git and the `Supply chain`
workflow fails if any drifts from the lockfile. The
script rewrites the absolute path `cargo-cyclonedx` records for the
workspace's own crates to a fixed `path+file:///src/` prefix; without that
the file differed between machines, which is why the workflow's first run on
`main` failed.

Three questions a security review asks, and what answers each:

| Question | Checked by | When |
|---|---|---|
| Known vulnerabilities | `cargo deny check` against the RustSec advisory database; GitHub Dependabot alerts against the GitHub Advisory Database | Pushes to `main`, every pull request, weekly; Dependabot continuously |
| Malware | `scripts/malware.sh` (ClamAV, signatures refreshed each run) over our source, every component's source, the built bundle, and the Windows package | Source and components: pushes to `main`, every pull request, weekly. Bundle: before every deploy and every release |
| Network capability | Crate bans and feature denials in `deny.toml`; a grep of `crates/` and `web/`; `scripts/no-network.sh` on the built bundle; in the browser, the page's Content-Security-Policy; for the Windows program, `scripts/windows-imports.sh`, which fails if it imports a networking library | Pushes to `main` and every pull request; on the built bundle before every deploy and release; the policy, every time the page runs |

Pushes to branches other than `main` are checked when they become a pull
request.

What these do not show.

- A scan finds what is already known. A clean result means no match on that
  date, not that a file is benign or a component free of undisclosed flaws.
- `scripts/no-network.sh` is a tripwire, not a proof. It fails the build if
  the compiled module imports a network API by name, or if the JavaScript
  beside it mentions one or contains any `fetch` but the loader's. It cannot
  see a request made through functions that are not network APIs by name:
  an `<img>` given a `src`, an assignment to `location`, a function looked up
  at run time.
- Most of those are stopped where they would happen. `web/index.html`
  carries a Content-Security-Policy that allows the page to load from and
  connect to its own site only: no outside scripts, images, styles, frames or
  connections, no inline script, no `eval`. Checked in headless Chrome by an
  independent reviewer: some thirty routes, from `fetch` and WebSocket to an
  image `src`, a style `url()` and a worker, were each refused, and the app
  ran at full speed under the policy. The build reads the page and fails
  unless it opens with the doctype, `<html>` and `<head>` and this exact
  policy comes next, after nothing but `<meta>` and `<title>`. That refuses
  the ordinary ways a policy ends up ignored: commented out, loosened,
  moved, or pushed out of `<head>` by a stray character. It is a check of
  the page's text, not a browser, and does not prove the policy is applied;
  loading the built page in a browser in CI would, and is tracked in
  issue #33.
- "Its own site" is the whole host the page is served from, not only this
  project's path on it.
- Two things a policy does not stop. **Navigation:** the Report feedback
  link is the viewer's choice and is described above, but script that
  assigned `location` or opened a window would also get out, visibly.
  **WebRTC:** not covered. `connect-src` does not apply to peer
  connections, and Chrome ignores the `webrtc` directive meant for them: the
  reviewer got a STUN request out under the policy with and without it. A
  peer connection built by looking its constructor up at run time would
  therefore pass both the build check and the policy. The build check does
  catch the name `RTCPeerConnection` in the module's imports and in the glue.
- Only Chrome has been tested. Firefox, Safari and Edge are expected to
  accept the policy but have not been tried.
- A program with its own approved scanners runs them on delivery. The SBOM
  exists so that it can.

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
```

An SVD is written for whoever receives the software. If a site is on 1.0 and
is being given 1.4, its "changes installed" section must cover 1.0 to 1.4,
including the 1.1 through 1.3 releases that site never installed. `--from`
takes the tag or commit the recipient has; the skipped releases are listed.

## Where is...

| Looking for | It is here |
|---|---|
| The Windows program | `ames-window-windows-x64.zip` on a GitHub Release (none cut yet). Its SBOM is [`sbom-windows.cdx.json`](sbom-windows.cdx.json), and the same file, byte for byte, is inside the zip. An unscanned build of any pull request is the `ames-window-windows-x64` artifact of its `CI` run. |
| The release SBOM | [`sbom-web.cdx.json`](sbom-web.cdx.json): the browser build, which is what a release ships. Will be attached to each GitHub Release; none has been cut yet. |
| The SBOM for every platform | [`sbom.cdx.json`](sbom.cdx.json). An audit view; no single build contains all of it. |
| Vulnerability scan results | The `Supply chain` workflow on the [Actions tab](https://github.com/brucedombrowski/Ames-Window/actions/workflows/security.yml), job `cargo deny`. Policy in [`deny.toml`](deny.toml). GitHub's own alerts are under Security, Dependabot, visible to maintainers only. |
| Licence check results | The same `cargo deny` job; the allowlist is in [`deny.toml`](deny.toml). |
| The Software Version Description | Generated on every `Supply chain` run and downloadable from that run as the `sbom-and-svd` artifact (needs a signed-in GitHub account; artifacts expire). Will be attached to each GitHub Release. |
| Releases | The [Releases page](https://github.com/brucedombrowski/Ames-Window/releases). **No release has been cut yet.** When one is, it carries the web bundle, both SBOMs, the SVD and `SHA256SUMS`, and GitHub holds a signed provenance attestation tying them to a commit and a build, checked with `gh attestation verify`. |
| Requirements and their evidence | [`docs/REQ-2026-001.json`](docs/REQ-2026-001.json) and [`docs/VER-2026-001.md`](docs/VER-2026-001.md). |
| Decisions and why | `docs/DM-2026-*.md`. |
| Why the SBOM timestamp says 1970 | Deliberate: a fixed timestamp makes the file reproducible byte for byte, so CI can prove it matches `Cargo.lock`. When it was really generated is the commit date, and for a release the attestation ([`docs/DM-2026-002.md`](docs/DM-2026-002.md), Consequences). |
| How to report a problem with the web page | The **Report feedback** link in the page's bottom right corner. |

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
fact nearer, perception and reality agree. For the other half of each turn the
tall edge is behind, and a flat picture cannot say so: the same outline fits a
window turned the opposite way with its tall edge in front, and that is the
one the brain picks. So the believed window never completes a turn. Its tall
edge sweeps across in front and back again while the real one goes all the way
round. `ames_core::perceived_angle` is that rule as a single `if`. The red strips on the frame are the painted thickness
of a real window, not shadows; they are what sells the depth.

## License

Licensed under either of the [MIT license](LICENSE-MIT) or the
[Apache License, Version 2.0](LICENSE-APACHE), at your option. This is the
convention across the Rust ecosystem, so the code can be used anywhere other
Rust crates can.
