# Ames Window

A pure‑Rust iOS app that performs the **Ames window illusion**: a flat
trapezoid, painted like a window in perspective, that spins full circle but
appears to swing back and forth. Thread a rod through it and the rod seems to
bend and pass through solid wood.

![Five frames of the window turning](docs/preview.png)

The small dial at the bottom is a bird's‑eye view. The thin cream bar with a
dot on its tall end is what the window is *really* doing. The thick blue bar is
what your eye insists it is doing.

New to Rust? Start with [docs/tour.md](docs/tour.md), a file‑by‑file reading
order that introduces one idea at a time.

## Layout

| Crate | What it is |
|---|---|
| `crates/ames-core` | The geometry. No dependencies, no pixels. Trapezoid, camera, rotation, projection, and the perceived‑angle model. Fully unit tested. |
| `crates/ames-app` | The application. `winit` for the window and touch, `tiny-skia` to rasterise, `softbuffer` to present. The same binary runs on iOS, macOS, Windows, Linux and the browser. |

There is no Swift, no Objective‑C, no Xcode project and no GPU code. The
Objective‑C runtime is reached through the `objc2` bindings that `winit` and
`softbuffer` already use.

## Splash screen

The app opens the way every Rust program opens: `Hello, world!`, lettered in a
five‑by‑seven dot font defined in `font.rs`, cursor blinking. After a moment it
dissolves into the window.

<img src="docs/splash.png" width="195" alt="Hello, world! splash screen">

## Controls

| Gesture | Effect |
|---|---|
| Tap | Toggle the rod |
| Drag left or right | Turn the window by hand (spinning pauses while you hold it) |
| `Space` (desktop) | Toggle the rod |
| `R` (desktop) | Reverse the spin |

## Build

```sh
cargo test                                  # core + app unit tests
cargo run -p ames-app                       # desktop preview
cargo run -p ames-app --example snapshot -- out   # PNG frames, no window needed
```

### Web

The same code compiles to WebAssembly and runs in any modern browser. The
`Pages` workflow builds it on every push and publishes it to GitHub Pages at
`https://brucedombrowski.github.io/Ames-Window/`.

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.130 --locked
scripts/web.sh serve      # then open http://localhost:8000
```

### iOS

On a Mac with Xcode installed:

```sh
rustup target add aarch64-apple-ios aarch64-apple-ios-sim
scripts/ios.sh sim        # builds, bundles and launches in the simulator
scripts/ios.sh device     # needs CODESIGN_IDENTITY, see the script header
```

The script compiles the binary with Cargo, copies it next to `ios/Info.plist`
inside an `.app` folder, and hands that to `simctl`. That is the whole pipeline.

## Supply chain

`sbom.cdx.json` is a [CycloneDX](https://cyclonedx.org) 1.6 Software Bill of
Materials listing every crate the binary depends on, for every target, with
versions, licences and the dependency graph. It is generated from `Cargo.lock`
by `scripts/sbom.sh` and is byte-for-byte reproducible, so it lives in git and
the `Supply chain` workflow fails if it ever drifts from the lockfile.

`deny.toml` is the policy: `cargo deny check` fails on known vulnerabilities,
licences outside the permissive allowlist, wildcard versions, or crates from
anywhere other than crates.io. The same workflow runs it on every push and
weekly, so new advisories surface without a code change.

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

## Why it works

The brain assumes the taller edge of the trapezoid is nearer, because that is
what a real rectangle would look like in perspective. While the tall edge is in
fact nearer, perception and reality agree. Once the short edge swings nearer,
the assumption is wrong, so the brain mirrors the motion rather than abandon
it, and the window appears to reverse. `ames_core::perceived_angle` is that
rule in six lines.

## License

Licensed under either of the [MIT license](LICENSE-MIT) or the
[Apache License, Version 2.0](LICENSE-APACHE), at your option. This is the
convention across the Rust ecosystem, so the code can be used anywhere other
Rust crates can.
