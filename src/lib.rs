//! Modor is a modular and kind of object-oriented game engine.
//!
//! This game engine is based on the
//! [entity-component-system](https://en.wikipedia.org/wiki/Entity_component_system) pattern, but
//! provides an API that represents entities like strongly typed objects, and provides tools similar
//! to the object-oriented paradigm:
//! - data are represented by components
//! - logic is put in systems that are only run for the entity type where they are defined
//! - data and logic inheritance is possible between entity types
//!
//! # Examples
//!
//! ```rust
//! use modor::*;
//!
//! App::new()
//!     .with_entity::<Character>((Position(45., 65.), CharacterType::Main))
//!     .with_entity::<Character>((Position(98., 12.), CharacterType::Enemy))
//!     .with_entity::<Character>((Position(14., 23.), CharacterType::Enemy))
//!     .update();
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
//!     fn build(builder: EntityBuilder<'_, Self>, (position, type_): Self::Data) -> Built {
//!         builder
//!             .with_if(Enemy, matches!(type_, CharacterType::Enemy))
//!             .with(position)
//!             .with_self(Self { ammunition: 10 })
//!     }
//!
//!     fn on_update(runner: &mut EntityRunner<'_, Self>) {
//!         runner.run(system!(Self::fire_when_enemy));
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

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::wildcard_imports))]

#[macro_use]
extern crate non_empty_vec;
#[cfg(test)]
#[macro_use]
extern crate static_assertions;
#[cfg(doctest)]
#[macro_use]
extern crate doc_comment;

#[macro_use]
mod utils;
mod app;
mod entities;
mod storages;
mod system_checks;
mod system_params;
mod systems;

pub mod testing;

pub use app::*;
pub use entities::*;
pub use system_checks::*;
pub use system_params::entity::*;
pub use system_params::queries::*;
pub use system_params::world::*;
pub use system_params::*;
pub use systems::*;

#[cfg(doctest)]
doc_comment!(include_str!("../README.md"));
