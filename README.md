# Ames Window

A pure‑Rust iOS app that performs the **Ames window illusion**: a flat
trapezoid, painted like a window in perspective, that spins full circle but
appears to swing back and forth. Thread a rod through it and the rod seems to
bend and pass through solid wood.

![Five frames of the window turning](docs/preview.png)

The small dial at the bottom is a bird's‑eye view. The thin cream bar with a
dot on its tall end is what the window is *really* doing. The thick blue bar is
what your eye insists it is doing.

## Layout

| Crate | What it is |
|---|---|
| `crates/ames-core` | The geometry. No dependencies, no pixels. Trapezoid, camera, rotation, projection, and the perceived‑angle model. Fully unit tested. |
| `crates/ames-app` | The application. `winit` for the window and touch, `tiny-skia` to rasterise, `softbuffer` to present. The same binary runs on iOS, macOS, Windows and Linux. |

There is no Swift, no Objective‑C, no Xcode project and no GPU code. The
Objective‑C runtime is reached through the `objc2` bindings that `winit` and
`softbuffer` already use.

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
