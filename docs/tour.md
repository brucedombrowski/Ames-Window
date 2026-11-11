# A first tour of the code

Read the files in this order. Each one introduces one or two ideas and leans on
nothing you have not seen yet. Every file is short.

## 1. `crates/ames-core/src/angle.rs` — a type that is just a number

```rust
pub struct Angle(f32);
```

`f32` is a 32‑bit decimal number. `Angle` wraps one so the compiler can stop
you from passing degrees where radians were expected. This is a *newtype*,
and it is the most Rust idea in the whole project: make wrong code fail to
compile.

Things to notice:

- `pub fn degrees(d: f32) -> Self` is a function that *returns* an `Angle`.
  `Self` means "this type". `-> ` reads "returns".
- `#[must_use]` warns if you call a function and ignore the result.
- `impl Add for Angle` teaches `+` how to add two angles.
- `#[cfg(test)] mod tests` at the bottom is the test suite, in the same file.

## 2. `crates/ames-core/src/math.rs` — structs and methods

`Vec3` is three numbers with names. `impl Vec3 { ... }` attaches functions to
it. A function whose first argument is `self` is a *method*, called with a dot:
`point.rotate_y(angle)`.

`#[derive(Clone, Copy, Debug, PartialEq)]` asks the compiler to write boring
code for you: copying, printing, comparing.

## 3. `crates/ames-core/src/camera.rs` — returning two things

```rust
pub fn project(&self, point: Vec3) -> (Vec2, f32)
```

Returns a pair: where the point lands on screen and how far away it is. The
caller unpacks it with `let (position, depth) = camera.project(p);`.

`&self` means "borrow me, don't take me". Rust tracks who owns every value;
borrowing is how you look at something without owning it.

## 4. `crates/ames-core/src/window.rs` — closures and iterators

```rust
(0..4).map(|i| { ... }).collect()
```

`0..4` is the numbers 0, 1, 2, 3. `.map(|i| ...)` runs a small inline function
(a *closure*, the `|i|` is its argument) on each one. `.collect()` gathers the
results into a `Vec`, Rust's growable list. You will see this shape everywhere
in Rust instead of `for` loops that push into an array.

## 5. `crates/ames-core/src/illusion.rs` — the whole illusion in one `if`

Six lines. `if` is an expression here: each branch *is* the return value, so
there is no `return` keyword.

## 6. `crates/ames-core/src/scene.rs` — `Option` and ownership

```rust
pub rod: Option<Rod>,
```

There is no `null` in Rust. A value that might be absent is an `Option`: either
`Some(rod)` or `None`, and the compiler makes you handle both. Look at
`toggle_rod` in `app.rs` later to see the `match`.

`render` builds the list of shapes to draw. Note how it is split: rod halves
behind the glass, then the window, then rod halves in front.

## 7. `crates/ames-app/src/gesture.rs` — enums with data

```rust
pub enum Gesture { Tap, Drag(f32) }
```

An enum variant can carry a value. `match gesture { Gesture::Tap => ..., Gesture::Drag(dx) => ... }`
pulls the value back out. This is how Rust avoids flag‑and‑field combinations
that can be inconsistent.

## 8. `crates/ames-app/src/paint.rs` — drawing

Pure drawing: a pixmap goes in, pixels come out. Because it touches no window
or input, the test at the bottom can render a frame and count coloured pixels.

## 9. `crates/ames-app/src/app.rs` — the event loop

`winit` calls our methods: `resumed` when there is a screen, `window_event`
for every touch, key and redraw. `let Some(x) = ... else { return; }` is an
early exit when something we need is missing.

## Things that will feel odd at first

| You see | It means |
|---|---|
| `let` | declare a variable. Immutable unless you write `let mut`. |
| `&x` / `&mut x` | borrow `x` read‑only / borrow it to change it |
| `x?` | if `x` is an error, return it to the caller right now |
| `::` | path separator, like `.` for modules: `Angle::degrees(90.0)` |
| `!` after a name | a macro, like `vec![...]` or `assert!(...)` |
| `'static` | a lifetime; here it just means "lives for the whole program" |
| `#[...]` | an attribute: instructions to the compiler, not code |

## Try changing something

1. In `window.rs`, change `columns: 2` to `columns: 4` in `Window::classic`.
2. Run `cargo test`. A test fails, because it knows how many mullions to expect.
   Read the failure, then read the test. That loop is how Rust is learned.
3. Run `cargo run -p ames-app --example snapshot -- out` and open the PNGs.

`cargo fmt` formats everything. `cargo clippy` is a stricter compiler that
explains itself; read its hints, they are good teaching.
