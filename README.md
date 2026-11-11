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
| `crates/ames-core` | The geometry. No dependencies, no pixels. Trapezoid, painted reveals, pen, camera, rotation, projection, and the perceived‑angle model. Every module has unit tests. |
| `crates/ames-app` | The application, binary name `ames-window`. `winit` for the window and input, `tiny-skia` to rasterise on the CPU, `softbuffer` to present. One source tree for every platform; the only platform‑specific code is the entry point and the clock on wasm. |
| `crates/ames-app/examples/snapshot.rs` | Renders frames to PNG without opening a window. |
| `web/index.html`, `scripts/web.sh` | The browser shell and the build script that produces `web/dist`. |
| `ios/Info.plist`, `scripts/ios.sh` | The iOS bundle metadata and the build‑sign‑install script. |
| `.github/workflows/pages.yml` | CI: tests, builds the wasm, deploys to GitHub Pages. |
| `docs/` | Preview images and the tour. |

There is no Swift, no Objective‑C, no Xcode project and no GPU code. Both
crates are `#![forbid(unsafe_code)]`. On Apple platforms the Objective‑C
runtime is reached through the `objc2` bindings that `winit` and `softbuffer`
depend on.

The app has no network capability of any kind and never will: no networking
crate is in the dependency tree and the source uses no sockets. That is a
charter item, recorded in issue #8.

## Requirements

- Rust **1.87 or newer**. The workspace uses edition 2024 and
  `u128::is_multiple_of`; 1.86 fails to compile.
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
deploys `web/dist` to GitHub Pages. It triggers on pushes to `main` (and on a
leftover development branch named in the workflow file), and can be started by
hand from the Actions tab. Deploys from branches other than `main` have failed
at the deploy job; from `main` the deploy succeeded on 2026‑11‑11, so the app
is at:

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
