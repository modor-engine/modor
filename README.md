# modor

[![Crates.io](https://img.shields.io/crates/v/modor.svg)](https://crates.io/crates/modor)
[![Docs.rs](https://img.shields.io/docsrs/modor)](https://docs.rs/crate/modor)
[![License](https://img.shields.io/crates/l/modor)](https://github.com/modor-engine/modor)
[![CI](https://github.com/modor-engine/modor/actions/workflows/ci.yml/badge.svg)](https://github.com/modor-engine/modor/actions/workflows/ci.yml)
[![Coverage with grcov](https://img.shields.io/codecov/c/gh/modor-engine/modor)](https://app.codecov.io/gh/modor-engine/modor)
[![Lines of code](https://tokei.rs/b1/github/modor-engine/modor?category=code)](https://github.com/modor-engine/modor)
[![Unsafe usage](https://img.shields.io/badge/unsafe%20usage-1-green.svg)](https://github.com/modor-engine/modor/search?q=path%3Acrates+extension%3Ars+unsafe)

Modor is a *mo*dular and *d*ata-*or*iented game engine, based on the following principles:

- *Modularity*: the [entity component system](https://en.wikipedia.org/wiki/Entity_component_system) pattern makes it
  very easy to:
    - Extend functionalities of the engine in reusable modules.
    - Split a project into multiple independent crates.
    - Reduce coupling between parts of an application.
- *Simplicity*:
    - Everything is stored in an entity, even resources, settings or loaded modules.
    - Systems are always linked to component types to facilitate system import and limit their side effects.
    - The ability to define a component as system dependency makes system ordering easy and maintainable.
- *Compile-time checking*: compile-time checks are used extensively to avoid as many errors as possible
  during runtime:
    - System parameters are checked to avoid mutability issues at runtime, e.g. if the same component type is mutably
      queried twice by the same system.
    - System execution order is checked to avoid dependency cycles.
    - The API is designed to avoid runtime panics as much as possible.

## Supported platforms

- Windows
- Linux
- Android
- Web

Modor could also work on some other platforms, like macOS, but they have not been tested.

## Usage

The minimum supported version of Rust is
defined [in this file](https://github.com/modor-engine/modor/blob/main/Cargo.toml).

To include this library in your project, just add the following dependency in your `Cargo.toml` file:

```toml
modor = "0.1"
```

You can also include the modules you want, like:

- [physics](crates/modor_physics/README.md)
- [graphics](crates/modor_graphics/README.md)
- [input](crates/modor_input/README.md)

## Examples

You can use one of the following commands to run an example:

- Desktop: `cargo run --example <name> --release`
- Android: `cargo apk run --manifest-path=examples/Cargo.toml --example android_<name> --release`
  (requires [cargo-apk](https://crates.io/crates/cargo-apk))
- Web: `cargo run-wasm --example <name> --release`

For example: `cargo run --example rendering_2d --release`

## License

Licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as
defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
