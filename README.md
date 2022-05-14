# modor

[![Crates.io](https://img.shields.io/crates/v/modor.svg)](https://crates.io/crates/modor)
[![Docs.rs](https://img.shields.io/docsrs/modor)](https://docs.rs/crate/modor)
[![License](https://img.shields.io/crates/l/modor)](https://github.com/modor-engine/modor)
[![CI](https://github.com/modor-engine/modor/actions/workflows/ci.yml/badge.svg)](https://github.com/modor-engine/modor/actions/workflows/ci.yml)
[![Coverage with grcov](https://img.shields.io/codecov/c/gh/modor-engine/modor)](https://app.codecov.io/gh/modor-engine/modor)
[![Lines of code](https://tokei.rs/b1/github/modor-engine/modor?category=code)](https://github.com/modor-engine/modor)
[![Unsafe usage](https://img.shields.io/badge/unsafe%20usage-0-green.svg)](https://github.com/modor-engine/modor/search?q=path%3Acrates%2Fmodor+extension%3Ars+unsafe)

Modor is a modular and kind of object-oriented game engine. It is based on
the [entity-component-system](https://en.wikipedia.org/wiki/Entity_component_system) pattern, but provides an API that
represents entities like strongly typed objects.

It also makes extensive use of compile-time checks. For example:

- system parameters are checked to avoid mutability issues at runtime, e.g. if the same component type is mutably
  queried twice by the same system
- system execution order is checked to avoid dependency cycles
- the engine API is designed to avoid runtime panics as much as possible

## Supported platforms

- Windows
- Linux
- macOS
- Web

## Usage

The minimum supported version of Rust is 1.60.

To include this library in your project, just add the following dependency in your `Cargo.toml` file:

```toml
modor = "0.1"
```

You can also include the modules you want, like:

- [physics](crates/modor_physics/README.md)
- [graphics](crates/modor_graphics/README.md)

## Examples

Examples are located in `examples/modor_examples/examples`.

You can use the following command to run them:

- Desktop: `cargo run --example <name>`
- Web: `cargo run-wasm --example <name>`

## License

Licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as
defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
