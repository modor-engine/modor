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

// TODO: make static asserts to check auto traits Sync and Send (to avoid future breaking changes)
