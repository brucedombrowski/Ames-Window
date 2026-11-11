# Ames Window

A pure‑Rust iOS app that performs the **Ames window illusion**: a flat
trapezoid, painted like a six‑pane window in perspective, that spins full
circle but appears to swing back and forth. Thread a pen through it and the pen
seems to bend and pass through solid wood.

![Five frames of the window turning](docs/preview.png)

Tap the screen to reveal a small dial: a bird's‑eye view. The thin cream bar
with a dot on its tall end is what the window is *really* doing. The thick blue
bar is what your eye insists it is doing. Tap again to hide it and let the
illusion back in.

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
| Tap | Show or hide the dial that reveals the true rotation |
| Drag left or right | Turn the window by hand (spinning pauses while you hold it) |
| `Space` (desktop) | Show or hide the dial |
| `X` (desktop) | Toggle the rod |
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
