# modor

[![Crates.io](https://img.shields.io/crates/v/modor.svg)](https://crates.io/crates/modor)
[![Docs.rs](https://img.shields.io/docsrs/modor)](https://docs.rs/crate/modor)
[![License](https://img.shields.io/crates/l/modor)](https://github.com/modor-engine/modor)
[![CI](https://github.com/modor-engine/modor/actions/workflows/ci.yml/badge.svg)](https://github.com/modor-engine/modor/actions/workflows/ci.yml)
[![Coverage with grcov](https://img.shields.io/codecov/c/gh/modor-engine/modor)](https://app.codecov.io/gh/modor-engine/modor)
[![Mutation tested with mutagen](https://img.shields.io/badge/mutation%20tested-mutagen-blue.svg)](https://github.com/modor-engine/modor/actions/workflows/ci.yml)
[![Lines of code](https://tokei.rs/b1/github/modor-engine/modor?category=code)](https://github.com/modor-engine/modor)
[![Safe Rust](https://img.shields.io/badge/safe%20Rust-%E2%9C%94%EF%B8%8F-green.svg)](https://github.com/modor-engine/modor/search?q=unsafe)

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

Not tested but should also work on:

- macOS

## Usage

The minimum supported version of Rust is 1.57.

To include this library in your project, just add the following dependency in your `Cargo.toml` file:

```toml
modor = "0.1"
```

You can also include the modules you want, like:
- [physics](crates/modor_physics/README.md)

## Example

 ```rust
#[macro_use]
extern crate modor;

use modor::*;

fn main() {
    App::new()
        .with_entity(Character::build(Position(45., 65.), CharacterType::Main))
        .with_entity(Character::build(Position(98., 12.), CharacterType::Enemy))
        .with_entity(Character::build(Position(14., 23.), CharacterType::Enemy))
        .update();
}

#[derive(Debug)]
struct Position(f32, f32);

enum CharacterType {
    Main,
    Neutral,
    Enemy,
}

struct Enemy;

struct Character {
    ammunition: u32,
}

#[entity]
impl Character {
    fn build(position: Position, type_: CharacterType) -> impl Built<Self> {
        EntityBuilder::new(Self { ammunition: 10 })
            .with_option(matches!(type_, CharacterType::Enemy).then(|| Enemy))
            .with(position)
    }

    #[run]
    fn fire_when_enemy(&mut self, position: &Position, _: &Enemy) {
        if self.ammunition > 0 {
            self.ammunition -= 1;
            println!("Enemy at {:?} has fired", position);
        }
    }
}
```

## License

Licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as
defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
