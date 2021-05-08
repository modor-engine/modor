#[macro_use]
mod utils;

mod application;
mod entities;
mod groups;
mod internal;
mod light_entities;
mod system_building;
mod system_checks;
mod system_iterators;
mod system_params;
mod system_resources;
mod systems;

pub use application::*;
pub use entities::*;
pub use groups::*;
pub use light_entities::*;
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
// - put "s" if multiple params in macros
// - avoid line breaks for a single instruction when not method chaining
// - docstrings starts with infinitive verb
// - put maximum of logic in "internal" module
// - choose carefully between "component_type" and "type" names
// - choose the lowest possible visibility before "mod", "use", "struct", "enum" and "fn"
// - all pub types are outside the "internal" module
// - statically assert public types to check auto traits Sync and Send (to avoid future breaking changes)
// - try to include special cases in macros instead of manually writing it
// - name of test modules should match the type names
