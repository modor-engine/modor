#![cfg_attr(test, allow(clippy::unwrap_used))]

#[cfg(test)]
#[macro_use]
extern crate static_assertions;

#[macro_use]
mod internal;

mod application;
mod entities;
mod groups;
mod system_building;
mod system_checks;
mod system_iterators;
mod system_params;
mod system_resources;
mod systems;

pub use application::*;
pub use entities::*;
pub use groups::*;
pub use system_building::*;
pub use system_checks::*;
pub use system_iterators::*;
pub use system_params::*;
pub use system_resources::*;
pub use systems::*;

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
