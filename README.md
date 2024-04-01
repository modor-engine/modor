# modor

[![Crates.io](https://img.shields.io/crates/v/modor.svg)](https://crates.io/crates/modor)
[![Docs.rs](https://img.shields.io/docsrs/modor)](https://docs.rs/crate/modor)
[![License](https://img.shields.io/crates/l/modor)](https://github.com/modor-engine/modor)
[![CI](https://github.com/modor-engine/modor/actions/workflows/ci.yml/badge.svg)](https://github.com/modor-engine/modor/actions/workflows/ci.yml)
[![Coverage with grcov](https://img.shields.io/codecov/c/gh/modor-engine/modor)](https://app.codecov.io/gh/modor-engine/modor)
[![Lines of code](https://tokei.rs/b1/github/modor-engine/modor?category=code)](https://github.com/modor-engine/modor)

Modor is a *mod*ular and *o*bject-o*r*iented game engine.

It has been designed with the following principles in mind:

- *Modularity*: the engine makes it easy to extend functionalities in an integrated way and to limit
  coupling between the different parts of an application.
- *Compile-time checking*: the API is designed to avoid as many errors as possible during runtime.
- *Simplicity*: the emphasis is on simplifying the API while guaranteeing good performance for
  real-life use cases.

## ⚠️ Warning ⚠️

Before considering to use this game engine, please keep in mind that:

- It is developed by a single person in his spare time.
- Although this engine can already be used to develop 2D games, some important features might still
  be missing.
- This engine is code-oriented, so no editor is included.

## Supported platforms

- Windows
- Linux
- macOS (limited support because the maintainer doesn't have access to a physical device)
- Android
- Web

Modor may also work on some other platforms, but they have not been tested.

## Usage

The minimum supported version of Rust is
defined [in this file](https://github.com/modor-engine/modor/blob/main/Cargo.toml).

You can include some or all engine features in your project by adding the following
dependencies in your `Cargo.toml` file:

```toml
modor = "0.1"
```

## Examples

You can use one of the following commands to run an example:

- Desktop: `cargo run --example <name> --release`
- Android: `cargo apk run --manifest-path=examples/Cargo.toml --example <name>_android --release`
  (requires [cargo-apk](https://crates.io/crates/cargo-apk))
- Web: `cargo run-wasm --example <name> --release`

For example: `cargo run --example rendering_2d --release`

## Behind the scene

Here are the main libraries used to implement Modor:

- Graphics crate is backed by [winit](https://github.com/rust-windowing/winit)
  and [wgpu](https://wgpu.rs/).
- Physics crate is backed by [rapier](https://rapier.rs/).

## License

Licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE)
  or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the
work by you, as
defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or
conditions.
