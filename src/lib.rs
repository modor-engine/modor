//! Modor is modular and kind of object-oriented game engine.
//!
//! This game engine is based on the
//! [entity-component-system](https://en.wikipedia.org/wiki/Entity_component_system) pattern, but
//! proposes an API that considers entities like strongly typed objects, and provides tools similar
//! to the object-oriented paradigm:
//! - data are represented by components
//! - logic are represented with systems only run on the entity type
//! - data and logic inheritance is possible between entity types
//!
//! # Examples
//!
//! ```rust
//! use modor::*;
//!
//! Application::new()
//!     .with_group(build_entity_group)
//!     .update();
//!
//! fn build_entity_group(builder: &mut GroupBuilder<'_>) {
//!     builder
//!         .with_entity::<Character>((Position(45., 65.), CharacterType::Main))
//!         .with_entity::<Character>((Position(98., 12.), CharacterType::Enemy))
//!         .with_entity::<Character>((Position(14., 23.), CharacterType::Enemy));
//! }
//!
//! #[derive(Debug)]
//! struct Position(f32, f32);
//!
//! enum CharacterType {
//!     Main,
//!     Neutral,
//!     Enemy,
//! }
//!
//! struct Enemy;
//!
//! struct Character {
//!     ammunition: u32,
//! }
//!
//! impl EntityMainComponent for Character {
//!     type Data = (Position, CharacterType);
//!
//!     fn build(builder: &mut EntityBuilder<'_, Self>, (position, type_): Self::Data) -> Built {
//!         if let CharacterType::Enemy = type_ {
//!             builder.with(Enemy);
//!         }
//!         builder
//!             .with(position)
//!             .with_self(Self { ammunition: 10 })
//!     }
//!
//!     fn on_update(runner: &mut EntityRunner<'_, Self>) {
//!         runner.run(entity_system!(Self::fire_when_enemy));
//!     }
//! }
//!
//! impl Character {
//!     fn fire_when_enemy(&mut self, position: &Position, _: &Enemy) {
//!         if self.ammunition > 0 {
//!             self.ammunition -= 1;
//!             println!("Enemy at {:?} has fired", position);
//!         }
//!     }
//! }
//! ```

#![cfg_attr(test, allow(clippy::unwrap_used))]

#[cfg(test)]
#[macro_use]
extern crate static_assertions;

#[macro_use]
mod internal;

mod external;

pub use external::application::*;
pub use external::entities::*;
pub use external::groups::*;
pub use external::systems::building::*;
pub use external::systems::checks::component_params::*;
pub use external::systems::checks::param_compatibility::*;
pub use external::systems::checks::query_component_params::*;
pub use external::systems::checks::*;
pub use external::systems::definition::*;
pub use external::systems::params::*;
pub use external::systems::resources::*;

pub(crate) use external::systems::iterators::*;

#[cfg(doctest)]
doc_comment::doctest!("../README.md");

// TODO: check the below rules are respected
// - use one letter convention for closure params
// - don't put "s" (plural) before "Facade" and "System" type suffixes
// - name of test modules should match the type names, and be on form "<typename>_tests"
// - put "s" if multiple params in macros
// - docstrings starts with infinitive verb
// - choose the lowest possible visibility before "mod", "use", "struct", "enum" and "fn"
// - all pub types are outside the "internal" module
// - try to include special cases in macros instead of manually writing it
// - https://rust-lang.github.io/api-guidelines/checklist.html
// - statically assert public types to check auto traits Sync and Send (to avoid future breaking changes)
// - statically assert public "mutable" types does not implement Clone
// - avoid line breaks for a single instruction when not method chaining
// - put maximum of logic in "internal" module
// - README, documentation and CHANGELOG are up-to-date
// - don't put assert!(x.is_some()); in tests as unwrap already test the value exists
